use std::ops::DerefMut;

use crate::prelude::*;

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
    mut query: Single<(&mut Transform, &CameraBounds), With<PixelPerfectCamera>>,
    config: Res<PixelPerfectCameraConfig>,
) {
    let (camera_transform, camera_bounds) = query.deref_mut();
    let clamped = camera_bounds.clamp_center(
        camera_transform.get_2d_pos(),
        config.viewport_dimensions_half(),
    );
    camera_transform.translation.x = clamped.x;
    camera_transform.translation.y = clamped.y;
}

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

#[derive(SystemParam)]
pub struct CameraBoundsInfo<'w, 's> {
    bounds: Single<'w, 's, &'static CameraBounds, With<PixelPerfectCamera>>,
}

impl CameraBoundsInfo<'_, '_> {
    /// Will use `f32::MIN` or `f32::MAX` for unbounded points
    pub fn as_approximate_rect(&self) -> Rect {
        let bounds = *self.bounds;
        Rect::new(
            bounds.xmin.unwrap_or(f32::MIN),
            bounds.ymin.unwrap_or(f32::MIN),
            bounds.xmax.unwrap_or(f32::MAX),
            bounds.ymax.unwrap_or(f32::MAX),
        )
    }

    /// Same as [`Self::as_approximate_rect`], but unbounded axes are clamped so
    /// gizmos don't try to draw a rectangle of infinite size.
    pub fn as_debug_rect(&self) -> Rect {
        const EXTENT: f32 = 4096.0;
        let rect = self.as_approximate_rect();
        Rect {
            min: Vec2::new(rect.min.x.max(-EXTENT), rect.min.y.max(-EXTENT)),
            max: Vec2::new(rect.max.x.min(EXTENT), rect.max.y.min(EXTENT)),
        }
    }
}
