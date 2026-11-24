//! Keeps the camera's fractional follow position, and copies a grid-snapped
//! position into [`Transform`] for rendering.

use crate::prelude::*;

use super::layout::effective_zoom;

/// Unsnapped camera center, and the grid position last written to [`Transform`].
#[derive(Component, Clone, Copy)]
pub(crate) struct SubpixelPos {
    logical: Vec2,
    on_grid: Option<Vec2>,
}

impl Default for SubpixelPos {
    fn default() -> Self {
        Self {
            logical: Vec2::ZERO,
            on_grid: None,
        }
    }
}

impl SubpixelPos {
    /// `pos` was set explicitly, so the next restore must not put the old fraction back.
    pub(crate) fn set_exact(&mut self, pos: Vec2) {
        self.logical = pos;
        self.on_grid = Some(pos);
    }

    pub(super) fn logical(&self) -> Vec2 {
        self.logical
    }

    pub(super) fn on_grid(&self) -> Option<Vec2> {
        self.on_grid
    }
}

/// Puts the fractional center back before follow and bounds run.
///
/// A transform that no longer matches the grid position was moved by something
/// else (`set_xy`, a panner) and becomes the new center.
pub(super) fn restore_subpixel_pos(
    mut cameras: Query<(&mut Transform, &mut SubpixelPos), With<super::PixelPerfectCamera>>,
) {
    for (mut transform, mut sample) in &mut cameras {
        let shown = transform.translation.truncate();
        let logical = match sample.on_grid {
            Some(on_grid) if on_grid == shown => sample.logical,
            _ => {
                sample.logical = shown;
                shown
            }
        };
        transform.translation.x = logical.x;
        transform.translation.y = logical.y;
    }
}

/// Snaps the camera onto the render grid after follow and bounds.
///
/// One grid step is one target texel: `1 / (res * zoom)` world units.
pub(super) fn snap_camera_to_render_grid(
    config: Res<PixelPerfectCameraConfig>,
    mut cameras: Query<
        (&mut Transform, &mut SubpixelPos, &CameraZoom),
        With<super::PixelPerfectCamera>,
    >,
) {
    let res = config.render_scale();
    for (mut transform, mut sample, zoom) in &mut cameras {
        let logical = transform.translation.truncate();
        let step = grid_step(res, zoom.0);
        let on_grid = Vec2::new(snap_axis(logical.x, step), snap_axis(logical.y, step));
        sample.logical = logical;
        sample.on_grid = Some(on_grid);
        transform.translation.x = on_grid.x;
        transform.translation.y = on_grid.y;
    }
}

pub(super) fn grid_step(res: u32, zoom: f32) -> f32 {
    1.0 / (res.max(1) as f32 * effective_zoom(zoom))
}

pub(super) fn snap_axis(value: f32, step: f32) -> f32 {
    (value / step).round() * step
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_axis_rounds_to_the_step() {
        assert_eq!(snap_axis(10.25, 0.5), 10.5);
        assert_eq!(snap_axis(10.0, 0.5), 10.0);
        assert_eq!(snap_axis(-1.25, 0.5), -1.5);
    }

    #[test]
    fn tenth_steps_are_not_eaten_by_the_grid() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PixelPerfectCameraConfig {
                game_width: 64.0,
                game_height: 64.0,
                res: 2,
            })
            .add_systems(
                PostUpdate,
                (
                    restore_subpixel_pos,
                    nudge_an_eighth,
                    snap_camera_to_render_grid,
                )
                    .chain(),
            );

        let camera = app
            .world_mut()
            .spawn((
                super::super::PixelPerfectCamera,
                CameraZoom(1.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                SubpixelPos::default(),
            ))
            .id();

        for _ in 0..16 {
            app.update();
        }

        let sample = app.world().get::<SubpixelPos>(camera).unwrap();
        assert!(
            (sample.logical.x - 2.0).abs() < 1e-4,
            "logical {}",
            sample.logical.x
        );
        let shown = app.world().get::<Transform>(camera).unwrap().translation.x;
        assert!((shown - snap_axis(sample.logical.x, 0.5)).abs() < 1e-4);
    }

    fn nudge_an_eighth(mut cameras: Query<&mut Transform, With<super::super::PixelPerfectCamera>>) {
        for mut transform in &mut cameras {
            transform.translation.x += 0.125;
        }
    }
}
