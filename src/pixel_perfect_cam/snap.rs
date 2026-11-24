//! Remembers the camera's follow position. That position is what renders.
//!
//! Rounding the camera onto a texel grid, and rounding sprites separately,
//! slides a stopped character a pixel into the level while the camera eases.

use crate::prelude::*;

/// Follow position, and the position last presented for rendering.
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

/// Records the follow position as the position that will be drawn.
///
/// The low-resolution target and nearest filtering define the pixel size.
/// The camera transform is left where follow and bounds put it, so the level
/// and the characters share one view.
pub(super) fn present_camera_at_follow_position(
    mut cameras: Query<(&Transform, &mut SubpixelPos), With<super::PixelPerfectCamera>>,
) {
    for (transform, mut sample) in &mut cameras {
        let logical = transform.translation.truncate();
        sample.logical = logical;
        sample.on_grid = Some(logical);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    present_camera_at_follow_position,
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
        assert!(
            (shown - sample.logical.x).abs() < 1e-4,
            "rendered {shown}, logical {}",
            sample.logical.x
        );
    }

    #[test]
    fn a_fractional_follow_position_is_rendered() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PixelPerfectCameraConfig {
                game_width: 64.0,
                game_height: 64.0,
                res: 1,
            })
            .add_systems(
                PostUpdate,
                (restore_subpixel_pos, nudge_an_eighth, present_camera_at_follow_position).chain(),
            );

        let camera = app
            .world_mut()
            .spawn((
                super::super::PixelPerfectCamera,
                CameraZoom(1.0),
                Transform::from_xyz(10.0, 0.0, 0.0),
                SubpixelPos::default(),
            ))
            .id();

        app.update();

        let shown = app.world().get::<Transform>(camera).unwrap().translation.x;
        assert!((shown - 10.125).abs() < 1e-4, "rendered {shown}");
    }

    fn nudge_an_eighth(mut cameras: Query<&mut Transform, With<super::super::PixelPerfectCamera>>) {
        for mut transform in &mut cameras {
            transform.translation.x += 0.125;
        }
    }
}
