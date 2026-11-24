//! Sprite positions stay fractional for movement. The rendered position is the
//! snapped camera plus the snapped gap between the two logical positions.
//!
//! Rounding the sprite and the camera separately makes `round(player) -
//! round(camera)` jump by a texel whenever they straddle a boundary, even when
//! the real gap is steady.

use crate::prelude::*;

use super::snap::{SubpixelPos, grid_step, snap_axis};

/// Fractional position for a sprite rendered relative to the pixel-perfect camera.
///
/// Physics and collision keep writing [`Transform`] during [`Update`]. Before
/// the frame renders, the drawn position becomes the camera's snapped position
/// plus the snapped gap from the camera's logical position. The fraction is
/// stored here and put back before the next frame's gameplay, so a speed below
/// one texel per frame still accumulates.
#[derive(Component, Clone, Copy, Debug)]
pub struct SpriteSubpixel {
    logical: Vec2,
    on_grid: Option<Vec2>,
}

impl Default for SpriteSubpixel {
    fn default() -> Self {
        Self {
            logical: Vec2::ZERO,
            on_grid: None,
        }
    }
}

/// Puts the fractional position back before gameplay reads [`Transform`].
///
/// A transform that no longer matches the texel last written was moved while
/// it was snapped (a spawn, a teleport) and becomes the new position.
pub(super) fn restore_sprite_subpixel(mut sprites: Query<(&mut Transform, &mut SpriteSubpixel)>) {
    for (mut transform, mut sample) in &mut sprites {
        let shown = transform.translation.truncate();
        let logical = match sample.on_grid {
            Some(on_grid) if on_grid == shown => sample.logical,
            _ => shown,
        };
        sample.logical = logical;
        write_xy(&mut transform, logical);
    }
}

/// Places each sprite on the camera texel grid after the camera has snapped.
///
/// The step is one canvas texel, `1 / (res * zoom)`. The drawn position is
/// `snap(camera) + snap(sprite - camera)`, using both logical positions, so a
/// steady follow lag stays on one screen texel.
pub(super) fn snap_sprites_to_pixel_grid(
    config: Res<PixelPerfectCameraConfig>,
    cameras: Query<(Entity, &SubpixelPos, &CameraZoom), With<PixelPerfectCamera>>,
    mut sprites: Query<(
        &mut Transform,
        &mut SpriteSubpixel,
        Option<&PixelPerfectCameraFollow>,
    )>,
) {
    let cameras: Vec<(Entity, CameraSample)> = cameras
        .iter()
        .map(|(entity, sample, zoom)| {
            (
                entity,
                CameraSample {
                    logical: sample.logical(),
                    on_grid: sample.on_grid(),
                    zoom: zoom.0,
                },
            )
        })
        .collect();
    let res = config.render_scale();
    for (mut transform, mut sample, follow) in &mut sprites {
        let logical = transform.translation.truncate();
        let camera = camera_for(follow, &cameras);
        let step = grid_step(res, camera.map(|camera| camera.zoom).unwrap_or(1.0));
        let on_grid = drawn_on_grid(logical, camera, step);
        sample.logical = logical;
        sample.on_grid = Some(on_grid);
        write_xy(&mut transform, on_grid);
    }
}

struct CameraSample {
    logical: Vec2,
    on_grid: Option<Vec2>,
    zoom: f32,
}

fn camera_for<'a>(
    follow: Option<&PixelPerfectCameraFollow>,
    cameras: &'a [(Entity, CameraSample)],
) -> Option<&'a CameraSample> {
    if let Some(id) = follow.and_then(|follow| follow.camera) {
        if let Some((_, sample)) = cameras.iter().find(|(entity, _)| *entity == id) {
            return Some(sample);
        }
        warn_once!("sprite subpixel follow target is not a pixel-perfect camera");
    }
    match cameras {
        [(_, sample)] => Some(sample),
        [] => None,
        _ => {
            warn_once!(
                "more than one pixel-perfect camera; sprites without a follow camera are snapped on their own"
            );
            None
        }
    }
}

/// `snap(camera) + snap(sprite - camera)`. Without a snapped camera, snap the sprite alone.
fn drawn_on_grid(logical: Vec2, camera: Option<&CameraSample>, step: f32) -> Vec2 {
    let Some(camera) = camera else {
        return snap_vec(logical, step);
    };
    let Some(camera_drawn) = camera.on_grid else {
        return snap_vec(logical, step);
    };
    camera_drawn + snap_vec(logical - camera.logical, step)
}

fn snap_vec(value: Vec2, step: f32) -> Vec2 {
    Vec2::new(snap_axis(value.x, step), snap_axis(value.y, step))
}

fn write_xy(transform: &mut Transform, pos: Vec2) {
    if transform.translation.x != pos.x {
        transform.translation.x = pos.x;
    }
    if transform.translation.y != pos.y {
        transform.translation.y = pos.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pixel_perfect_cam::snap::SubpixelPos;

    #[test]
    fn subpixel_steps_stay_on_the_camera_texel_until_the_next_one() {
        let mut app = test_app();
        app.add_systems(
            Update,
            (
                restore_sprite_subpixel,
                nudge_by_quarter,
                snap_sprites_to_pixel_grid,
            )
                .chain(),
        );

        let sprite = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 2.0),
                SpriteSubpixel::default(),
            ))
            .id();

        app.update();
        assert_drawn(app.world(), sprite, 0.5, 0.25);

        app.update();
        assert_drawn(app.world(), sprite, 0.5, 0.5);

        app.update();
        assert_drawn(app.world(), sprite, 1.0, 0.75);

        app.update();
        assert_drawn(app.world(), sprite, 1.0, 1.0);
        let z = app.world().get::<Transform>(sprite).unwrap().translation.z;
        assert_eq!(z, 2.0);
    }

    #[test]
    fn a_position_written_over_the_snapped_sprite_becomes_the_new_position() {
        let mut app = test_app();
        app.add_systems(PreUpdate, restore_sprite_subpixel)
            .add_systems(PostUpdate, snap_sprites_to_pixel_grid);

        let sprite = app
            .world_mut()
            .spawn((
                Transform::from_xyz(3.25, 4.0, 0.0),
                SpriteSubpixel::default(),
            ))
            .id();

        app.update();
        assert_drawn(app.world(), sprite, 3.5, 3.25);

        app.world_mut()
            .get_mut::<Transform>(sprite)
            .unwrap()
            .translation
            .x = 8.75;

        app.update();
        assert_drawn(app.world(), sprite, 9.0, 8.75);
    }

    #[test]
    fn moving_left_keeps_the_drawn_sprite_until_the_texel_is_crossed() {
        let mut app = test_app();
        app.add_systems(
            Update,
            (
                restore_sprite_subpixel,
                nudge_left,
                snap_sprites_to_pixel_grid,
            )
                .chain(),
        );

        let sprite = app
            .world_mut()
            .spawn((
                Transform::from_xyz(5.0, 0.0, 0.0),
                SpriteSubpixel::default(),
            ))
            .id();

        app.update();
        assert_drawn(app.world(), sprite, 5.0, 4.75);
    }

    #[test]
    fn a_caught_up_camera_and_sprite_land_on_the_same_texel() {
        let mut app = test_app();
        app.add_systems(
            PostUpdate,
            (
                crate::pixel_perfect_cam::snap::snap_camera_to_render_grid,
                snap_sprites_to_pixel_grid,
            )
                .chain(),
        );

        let camera = app
            .world_mut()
            .spawn((
                PixelPerfectCamera,
                CameraZoom(1.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                SubpixelPos::default(),
            ))
            .id();
        let sprite = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), SpriteSubpixel::default()))
            .id();

        for x in [0.0, 0.2, 0.25, 0.5, 0.74, 0.75, 0.9, 10.25] {
            app.world_mut().get_mut::<Transform>(camera).unwrap().translation.x = x;
            app.world_mut().get_mut::<Transform>(sprite).unwrap().translation.x = x;
            app.update();
            let camera_x = app.world().get::<Transform>(camera).unwrap().translation.x;
            let sprite_x = app.world().get::<Transform>(sprite).unwrap().translation.x;
            assert_eq!(sprite_x, camera_x, "fractional position {x}");
        }
    }

    #[test]
    fn a_constant_gap_stays_on_one_screen_pixel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PixelPerfectCameraConfig {
                game_width: 64.0,
                game_height: 64.0,
                res: 1,
            })
            .add_systems(
                PostUpdate,
                (
                    crate::pixel_perfect_cam::snap::snap_camera_to_render_grid,
                    snap_sprites_to_pixel_grid,
                )
                    .chain(),
            );

        let camera = app
            .world_mut()
            .spawn((
                PixelPerfectCamera,
                CameraZoom(1.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                SubpixelPos::default(),
            ))
            .id();
        let sprite = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), SpriteSubpixel::default()))
            .id();

        for gap in [0.25, -0.25, 0.75, -0.75] {
            let expected = snap_axis(gap, 1.0);
            for camera_x in [10.0, 10.25, 10.5, 10.75, 11.0] {
                app.world_mut()
                    .get_mut::<Transform>(camera)
                    .unwrap()
                    .translation
                    .x = camera_x;
                app.world_mut()
                    .get_mut::<Transform>(sprite)
                    .unwrap()
                    .translation
                    .x = camera_x + gap;
                app.update();
                let drawn_camera = app.world().get::<Transform>(camera).unwrap().translation.x;
                let drawn_sprite = app.world().get::<Transform>(sprite).unwrap().translation.x;
                assert_eq!(
                    drawn_sprite - drawn_camera,
                    expected,
                    "gap {gap} at camera {camera_x}"
                );
            }
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).insert_resource(PixelPerfectCameraConfig {
            game_width: 64.0,
            game_height: 64.0,
            res: 2,
        });
        app
    }

    fn nudge_by_quarter(mut sprites: Query<&mut Transform, With<SpriteSubpixel>>) {
        for mut transform in &mut sprites {
            transform.translation.x += 0.25;
        }
    }

    fn nudge_left(mut sprites: Query<&mut Transform, With<SpriteSubpixel>>) {
        for mut transform in &mut sprites {
            transform.translation.x -= 0.25;
        }
    }

    fn assert_drawn(world: &World, sprite: Entity, drawn: f32, logical: f32) {
        let shown = world.get::<Transform>(sprite).unwrap().translation.x;
        let sample = world.get::<SpriteSubpixel>(sprite).unwrap();
        assert_eq!(shown, drawn, "drawn pixel");
        assert_eq!(sample.logical.x, logical, "subpixel position");
    }
}
