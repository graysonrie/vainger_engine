//! Keeps each pixel-perfect camera inside its own optional world-space edges.

use crate::prelude::*;

/// Clamps each [`PixelPerfectCamera`] that has [`CameraBounds`] in
/// [`PixelPerfectCameraControlsSet::Bounds`] during [`PostUpdate`].
///
/// Added by [`PixelPerfectCameraPlugin`]. A camera with no [`CameraBounds`] is
/// left where it is. The margin is that camera's visible view: its cell size
/// divided by [`CameraZoom`].
pub struct CameraBoundsPlugin;

impl Plugin for CameraBoundsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            clamp_camera_bounds.in_set(PixelPerfectCameraControlsSet::Bounds),
        );
    }
}

fn clamp_camera_bounds(
    mut query: Query<
        (&mut Transform, &CameraBounds, &ScreenSlot, &CameraZoom),
        With<PixelPerfectCamera>,
    >,
) {
    for (mut camera_transform, camera_bounds, slot, zoom) in &mut query {
        let clamped = camera_bounds.clamp_center(
            camera_transform.get_2d_pos(),
            slot.visible_world_size(zoom.0) * 0.5,
        );
        camera_transform.translation.x = clamped.x;
        camera_transform.translation.y = clamped.y;
    }
}

/// World-space limits for the edges of one pixel-perfect camera's view.
///
/// Insert this on the [`PixelPerfectCamera`] entity, or call
/// [`PixelPerfectCameraQuery::set_bounds`]. Each field is how far that edge of
/// the view may go. [`None`] leaves that side open.
///
/// ### NOTE:
/// These are edges of the visible area, not limits on the camera center. The
/// center stays half the visible view away from a finite edge. The visible view
/// is the camera's cell divided by [`CameraZoom`]. If the allowed span on an
/// axis is smaller than that view, the camera is pinned to the midpoint of
/// that span instead of trying to satisfy both edges.
#[derive(Component, Clone, Copy, Debug)]
pub struct CameraBounds {
    /// Left edge of the allowed viewport. `None` = unbounded.
    pub xmin: Option<f32>,
    /// Right edge of the allowed viewport. `None` = unbounded.
    pub xmax: Option<f32>,
    /// Bottom edge of the allowed viewport. `None` = unbounded.
    pub ymin: Option<f32>,
    /// Top edge of the allowed viewport. `None` = unbounded.
    pub ymax: Option<f32>,
}

impl CameraBounds {
    /// Clamps a camera center so the viewport stays inside these edge bounds.
    pub fn clamp_center(&self, center: Vec2, viewport_half: Vec2) -> Vec2 {
        Vec2::new(
            Self::clamp_axis(center.x, self.xmin, self.xmax, viewport_half.x),
            Self::clamp_axis(center.y, self.ymin, self.ymax, viewport_half.y),
        )
    }

    fn clamp_axis(
        center: f32,
        min_edge: Option<f32>,
        max_edge: Option<f32>,
        viewport_half: f32,
    ) -> f32 {
        let min_center = min_edge.map(|edge| edge + viewport_half);
        let max_center = max_edge.map(|edge| edge - viewport_half);

        match (min_center, max_center) {
            (Some(min), Some(max)) if min > max => {
                let min_edge = min_edge.unwrap();
                let max_edge = max_edge.unwrap();
                (min_edge + max_edge) / 2.
            }
            (min_center, max_center) => {
                let mut center = center;
                if let Some(min) = min_center {
                    center = center.max(min);
                }
                if let Some(max) = max_center {
                    center = center.min(max);
                }
                center
            }
        }
    }
}

/// Reads [`CameraBounds`] from one [`PixelPerfectCamera`].
///
/// [`Self::as_approximate_rect`] turns open edges into `f32::MIN` / `f32::MAX`,
/// which will overflow a gizmo. Use [`Self::as_debug_rect`] for drawing.
#[derive(SystemParam)]
pub struct CameraBoundsInfo<'w, 's> {
    bounds: Query<'w, 's, &'static CameraBounds, With<PixelPerfectCamera>>,
}

impl CameraBoundsInfo<'_, '_> {
    /// Bounds on `camera`.
    pub fn of(&self, camera: Entity) -> Result<&CameraBounds, CameraLookupError> {
        self.bounds
            .get(camera)
            .map_err(|_| CameraLookupError::Unknown { entity: camera })
    }

    /// Bounds when exactly one camera has them.
    pub fn only(&self) -> Result<&CameraBounds, CameraLookupError> {
        let mut iter = self.bounds.iter();
        let Some(bounds) = iter.next() else {
            return Err(CameraLookupError::Missing);
        };
        if iter.next().is_some() {
            let found = 2 + iter.count();
            return Err(CameraLookupError::NotExactlyOne { found });
        }
        Ok(bounds)
    }

    /// Will use `f32::MIN` or `f32::MAX` for unbounded points.
    pub fn as_approximate_rect(&self, camera: Entity) -> Result<Rect, CameraLookupError> {
        let bounds = *self.of(camera)?;
        Ok(Rect::new(
            bounds.xmin.unwrap_or(f32::MIN),
            bounds.ymin.unwrap_or(f32::MIN),
            bounds.xmax.unwrap_or(f32::MAX),
            bounds.ymax.unwrap_or(f32::MAX),
        ))
    }

    /// Same as [`Self::as_approximate_rect`], but unbounded axes are clamped so
    /// gizmos don't try to draw a rectangle of infinite size.
    pub fn as_debug_rect(&self, camera: Entity) -> Result<Rect, CameraLookupError> {
        const EXTENT: f32 = 4096.0;
        let rect = self.as_approximate_rect(camera)?;
        Ok(Rect {
            min: Vec2::new(rect.min.x.max(-EXTENT), rect.min.y.max(-EXTENT)),
            max: Vec2::new(rect.max.x.min(EXTENT), rect.max.y.min(EXTENT)),
        })
    }
}
