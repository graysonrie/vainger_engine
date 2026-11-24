//! NES sprite placement. Movement keeps a fractional position, and the
//! rendered [`Transform`] is the whole pixel that contains it.

use crate::prelude::*;

/// Fractional position for an entity whose sprite must stay on whole pixels.
///
/// Physics and collision keep writing [`Transform`] during [`Update`]. Before
/// the frame renders, that position is floored onto a pixel and the fraction
/// is stored here. The next frame puts the fraction back before gameplay runs,
/// so a speed below one pixel per frame still accumulates.
///
/// The drawn pixel is `floor` of the position: the sprite stays on pixel `n`
/// while the position is in `[n, n + 1)`, then steps to the next pixel. That
/// is the NES accumulator. Rounding would draw the sprite a pixel early.
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
/// A transform that no longer matches the pixel last written was moved while
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

/// Floors the position onto a pixel after gameplay, including camera follow.
pub(super) fn snap_sprites_to_pixel_grid(
    mut sprites: Query<(&mut Transform, &mut SpriteSubpixel)>,
) {
    for (mut transform, mut sample) in &mut sprites {
        let logical = transform.translation.truncate();
        let on_grid = Vec2::new(logical.x.floor(), logical.y.floor());
        sample.logical = logical;
        sample.on_grid = Some(on_grid);
        write_xy(&mut transform, on_grid);
    }
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

    #[test]
    fn subpixel_steps_do_not_move_the_drawn_sprite_until_a_whole_pixel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_systems(
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
        assert_drawn(app.world(), sprite, 0.0, 0.25);

        app.update();
        assert_drawn(app.world(), sprite, 0.0, 0.5);

        app.update();
        assert_drawn(app.world(), sprite, 0.0, 0.75);

        app.update();
        assert_drawn(app.world(), sprite, 1.0, 1.0);
        let z = app.world().get::<Transform>(sprite).unwrap().translation.z;
        assert_eq!(z, 2.0);
    }

    #[test]
    fn a_position_written_over_the_snapped_sprite_becomes_the_new_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(PreUpdate, restore_sprite_subpixel)
            .add_systems(PostUpdate, snap_sprites_to_pixel_grid);

        let sprite = app
            .world_mut()
            .spawn((
                Transform::from_xyz(3.25, 4.0, 0.0),
                SpriteSubpixel::default(),
            ))
            .id();

        app.update();
        assert_drawn(app.world(), sprite, 3.0, 3.25);

        app.world_mut()
            .get_mut::<Transform>(sprite)
            .unwrap()
            .translation
            .x = 8.75;

        app.update();
        assert_drawn(app.world(), sprite, 8.0, 8.75);
    }

    #[test]
    fn moving_left_steps_the_drawn_sprite_when_the_pixel_is_crossed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_systems(
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
        assert_drawn(app.world(), sprite, 4.0, 4.75);
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
