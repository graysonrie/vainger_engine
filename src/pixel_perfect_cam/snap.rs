//! Keeps fractional positions for the camera and for world sprites, and copies
//! grid-snapped positions into [`Transform`] for rendering.
//!
//! Sprites are restored in [`PreUpdate`] so gameplay reads the logical position.
//! They are snapped in [`PostUpdate`](bevy::prelude::PostUpdate) after the camera
//! and parallax, using the same lattice as [`snap_camera_to_render_grid`].

use std::collections::HashMap;

use crate::prelude::*;

use super::layout::effective_zoom;
use super::screen_transition::ScreenFadeOverlay;

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

fn grid_step(res: u32, zoom: f32) -> f32 {
    1.0 / (res.max(1) as f32 * effective_zoom(zoom))
}

fn snap_axis(value: f32, step: f32) -> f32 {
    (value / step).round() * step
}

/// Logical sprite translation, and the grid translation last written for rendering.
///
/// The stored vectors are the entity's local translation. Child sprites keep their
/// own fraction; the snap shifts that local translation so the quad lands on the
/// world grid.
#[derive(Component, Clone, Copy, Default)]
pub(super) struct SpriteGridPos {
    logical: Vec2,
    on_grid: Option<Vec2>,
}

/// Puts each sprite's fractional translation back before gameplay reads it.
pub(super) fn restore_sprite_positions(mut sprites: Query<(&mut Transform, &mut SpriteGridPos)>) {
    for (mut transform, mut sample) in &mut sprites {
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

/// Snaps world-sprite quads onto the camera render grid.
///
/// The gameplay translation is stored on [`SpriteGridPos`]. [`GameCanvas`] and the
/// screen-fade overlay stay where they are: they are full-screen quads, not world sprites.
///
/// When cameras disagree on zoom, the snap uses the coarsest step (the smallest
/// zoom). A coarser step still lands on a finer one when the zooms are integer
/// multiples, which is the splitscreen case of zoom 1 and 2.
pub(super) fn snap_sprites_to_render_grid(
    config: Res<PixelPerfectCameraConfig>,
    cameras: Query<&CameraZoom, With<super::PixelPerfectCamera>>,
    parents: Query<&ChildOf>,
    mut sets: ParamSet<(
        Query<&Transform>,
        Query<
            (
                Entity,
                &Sprite,
                &Anchor,
                &mut Transform,
                Option<&mut SpriteGridPos>,
            ),
            (Without<super::GameCanvas>, Without<ScreenFadeOverlay>),
        >,
    )>,
    images: Option<Res<Assets<Image>>>,
    layouts: Option<Res<Assets<TextureAtlasLayout>>>,
    mut commands: Commands,
) {
    let Some(step) = coarsest_step(config.render_scale(), &cameras) else {
        return;
    };

    let ids: Vec<(u32, Entity)> = {
        let sprites = sets.p1();
        sprites
            .iter()
            .map(|(entity, ..)| (ancestor_depth(entity, &parents), entity))
            .collect()
    };
    let mut ordered = ids;
    ordered.sort_by_key(|(depth, entity)| (*depth, *entity));

    let mut locals = HashMap::new();
    {
        let transforms = sets.p0();
        for (_, entity) in &ordered {
            let mut current = Some(*entity);
            let mut guard = 0;
            while let Some(id) = current {
                if locals.contains_key(&id) {
                    break;
                }
                if let Ok(transform) = transforms.get(id) {
                    locals.insert(id, *transform);
                }
                current = parents.get(id).ok().map(|child_of| child_of.parent());
                guard += 1;
                if guard == 64 {
                    break;
                }
            }
        }
    }

    for (_, entity) in ordered {
        let world = chain_world(entity, &locals, &parents);
        let parent_world = match parents.get(entity) {
            Ok(child_of) => chain_world(child_of.parent(), &locals, &parents),
            Err(_) => Transform::IDENTITY,
        };

        let mut sprites = sets.p1();
        let Ok((_, sprite, anchor, mut transform, sample)) = sprites.get_mut(entity) else {
            continue;
        };
        let Some(size) = drawn_quad_size(sprite, images.as_deref(), layouts.as_deref()) else {
            continue;
        };
        if !size.x.is_finite() || !size.y.is_finite() {
            continue;
        }

        let logical = transform.translation.truncate();
        let snapped_world = snapped_world_origin(
            world.translation,
            world.rotation,
            world.scale,
            anchor.as_vec(),
            size,
            step,
        );
        let local_point = parent_world
            .compute_affine()
            .inverse()
            .transform_point3(snapped_world);
        if !local_point.is_finite() {
            continue;
        }

        transform.translation = local_point;
        let on_grid = local_point.truncate();
        if let Some(stored) = locals.get_mut(&entity) {
            stored.translation = local_point;
        }
        match sample {
            Some(mut sample) => {
                sample.logical = logical;
                sample.on_grid = Some(on_grid);
            }
            None => {
                commands.entity(entity).insert(SpriteGridPos {
                    logical,
                    on_grid: Some(on_grid),
                });
            }
        }
    }
}

/// Smallest positive zoom wins, so the grid step is the largest texel in world units.
fn coarsest_step(
    res: u32,
    zooms: &Query<&CameraZoom, With<super::PixelPerfectCamera>>,
) -> Option<f32> {
    let zoom = zooms
        .iter()
        .map(|zoom| effective_zoom(zoom.0))
        .reduce(f32::min)?;
    Some(grid_step(res, zoom))
}

fn ancestor_depth(entity: Entity, parents: &Query<&ChildOf>) -> u32 {
    let mut depth = 0;
    let mut current = entity;
    while let Ok(child_of) = parents.get(current) {
        depth += 1;
        current = child_of.parent();
        if depth == 64 {
            break;
        }
    }
    depth
}

fn chain_world(
    entity: Entity,
    locals: &HashMap<Entity, Transform>,
    parents: &Query<&ChildOf>,
) -> Transform {
    let mut chain = Vec::new();
    let mut current = Some(entity);
    let mut guard = 0;
    while let Some(id) = current {
        chain.push(id);
        current = parents.get(id).ok().map(|child_of| child_of.parent());
        guard += 1;
        if guard == 64 {
            break;
        }
    }
    let mut world = Transform::IDENTITY;
    for id in chain.iter().rev() {
        if let Some(local) = locals.get(id) {
            world = world.mul_transform(*local);
        }
    }
    world
}

/// World translation that puts one quad corner on the render grid.
///
/// The corner is the sprite mesh's `(0, 0)` vertex. Bevy places it at
/// `-size * (anchor + 0.5)` in local space, which is the minimum corner when the
/// sprite is axis-aligned. An integer frame size is a whole number of `1/res`
/// steps, so the other three corners land on the grid as well.
fn snapped_world_origin(
    origin: Vec3,
    rotation: Quat,
    scale: Vec3,
    anchor: Vec2,
    size: Vec2,
    step: f32,
) -> Vec3 {
    let corner_local = (-size * (anchor + Vec2::splat(0.5))).extend(0.0) * scale;
    let corner = (origin + rotation * corner_local).truncate();
    let snapped = Vec2::new(snap_axis(corner.x, step), snap_axis(corner.y, step));
    origin + (snapped - corner).extend(0.0)
}

/// Drawn quad size, matching Bevy's sprite extract: `custom_size`, otherwise the
/// atlas frame or image rect.
fn drawn_quad_size(
    sprite: &Sprite,
    images: Option<&Assets<Image>>,
    layouts: Option<&Assets<TextureAtlasLayout>>,
) -> Option<Vec2> {
    if let Some(size) = sprite.custom_size {
        return Some(size);
    }
    if let Some(atlas) = &sprite.texture_atlas {
        let rect = atlas.texture_rect(layouts?)?;
        if let Some(sprite_rect) = sprite.rect {
            return Some(sprite_rect.size());
        }
        return Some(rect.size().as_vec2());
    }
    if let Some(rect) = sprite.rect {
        return Some(rect.size());
    }
    images?.get(&sprite.image).map(|image| image.size_f32())
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

    #[test]
    fn sprite_corner_snaps_onto_the_texel_grid() {
        let origin = snapped_world_origin(
            Vec3::new(10.4, 3.2, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec2::ZERO,
            Vec2::splat(16.0),
            0.5,
        );
        assert!((origin.x - 10.5).abs() < 1e-4, "x {}", origin.x);
        assert!((origin.y - 3.0).abs() < 1e-4, "y {}", origin.y);
    }

    #[test]
    fn half_pixel_anchor_still_lands_on_the_grid() {
        // Opaque-center anchors from `frame_hitbox` are often `n + 0.5` pixels.
        let anchor = Vec2::new(0.5 / 16.0, 0.0);
        let origin = snapped_world_origin(
            Vec3::new(10.4, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            anchor,
            Vec2::splat(16.0),
            0.5,
        );
        let corner = origin.truncate() - Vec2::splat(16.0) * (anchor + Vec2::splat(0.5));
        assert!((origin.x - 10.5).abs() < 1e-4, "x {}", origin.x);
        assert!((corner.x - 2.0).abs() < 1e-4, "corner {}", corner.x);
    }

    #[test]
    fn sprite_motion_keeps_the_logical_position() {
        let mut app = App::new();
        app.insert_resource(PixelPerfectCameraConfig {
            game_width: 64.0,
            game_height: 64.0,
            res: 2,
        })
        .add_plugins(MinimalPlugins)
        .add_systems(PreUpdate, restore_sprite_positions)
        .add_systems(Update, nudge_sprite)
        .add_systems(PostUpdate, snap_sprites_to_render_grid);

        app.world_mut().spawn((
            super::super::PixelPerfectCamera,
            CameraZoom(1.0),
            Transform::default(),
        ));
        let sprite = app
            .world_mut()
            .spawn((
                Sprite::sized(Vec2::splat(16.0)),
                Anchor::CENTER,
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        for _ in 0..16 {
            app.update();
        }

        let sample = app.world().get::<SpriteGridPos>(sprite).unwrap();
        assert!(
            (sample.logical.x - 1.6).abs() < 1e-4,
            "logical {}",
            sample.logical.x
        );
        let shown = app.world().get::<Transform>(sprite).unwrap().translation;
        let expected = snapped_world_origin(
            Vec3::new(sample.logical.x, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec2::ZERO,
            Vec2::splat(16.0),
            0.5,
        );
        assert!((shown.x - expected.x).abs() < 1e-4, "shown {}", shown.x);
    }

    #[test]
    fn child_sprite_snaps_in_world_space() {
        let mut app = App::new();
        app.insert_resource(PixelPerfectCameraConfig {
            game_width: 64.0,
            game_height: 64.0,
            res: 2,
        })
        .add_plugins(MinimalPlugins)
        .add_systems(PostUpdate, snap_sprites_to_render_grid);

        app.world_mut().spawn((
            super::super::PixelPerfectCamera,
            CameraZoom(1.0),
            Transform::default(),
        ));
        let parent = app
            .world_mut()
            .spawn(Transform::from_xyz(10.25, 0.0, 0.0))
            .id();
        let child = app
            .world_mut()
            .spawn((
                Sprite::sized(Vec2::splat(8.0)),
                Anchor::CENTER,
                Transform::from_xyz(0.2, 0.0, 0.0),
                ChildOf(parent),
            ))
            .id();

        app.update();

        let child_tf = app.world().get::<Transform>(child).unwrap();
        let parent_tf = app.world().get::<Transform>(parent).unwrap();
        let world_x = parent_tf.translation.x + child_tf.translation.x;
        let expected = snapped_world_origin(
            Vec3::new(10.45, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec2::ZERO,
            Vec2::splat(8.0),
            0.5,
        );
        assert!(
            (world_x - expected.x).abs() < 1e-4,
            "world {world_x}, expected {}",
            expected.x
        );
    }

    #[test]
    fn canvas_sprite_is_not_snapped() {
        let mut app = App::new();
        app.insert_resource(PixelPerfectCameraConfig {
            game_width: 64.0,
            game_height: 64.0,
            res: 2,
        })
        .add_plugins(MinimalPlugins)
        .add_systems(PostUpdate, snap_sprites_to_render_grid);

        app.world_mut().spawn((
            super::super::PixelPerfectCamera,
            CameraZoom(1.0),
            Transform::default(),
        ));
        let canvas = app
            .world_mut()
            .spawn((
                Sprite::sized(Vec2::splat(64.0)),
                Transform::from_xyz(10.4, 0.0, 0.0),
                super::super::GameCanvas,
            ))
            .id();

        app.update();

        let shown = app.world().get::<Transform>(canvas).unwrap().translation.x;
        assert!((shown - 10.4).abs() < 1e-4, "shown {shown}");
        assert!(app.world().get::<SpriteGridPos>(canvas).is_none());
    }

    #[test]
    fn coarser_camera_zoom_sets_the_step() {
        let mut app = App::new();
        app.insert_resource(PixelPerfectCameraConfig {
            game_width: 64.0,
            game_height: 64.0,
            res: 2,
        })
        .add_plugins(MinimalPlugins)
        .add_systems(PostUpdate, snap_sprites_to_render_grid);

        app.world_mut().spawn((
            super::super::PixelPerfectCamera,
            CameraZoom(1.0),
            Transform::default(),
        ));
        app.world_mut().spawn((
            super::super::PixelPerfectCamera,
            CameraZoom(2.0),
            Transform::default(),
        ));
        let sprite = app
            .world_mut()
            .spawn((
                Sprite::sized(Vec2::splat(2.0)),
                Anchor::CENTER,
                Transform::from_xyz(0.2, 0.0, 0.0),
            ))
            .id();

        app.update();

        let shown = app.world().get::<Transform>(sprite).unwrap().translation.x;
        let expected = snapped_world_origin(
            Vec3::new(0.2, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec2::ZERO,
            Vec2::splat(2.0),
            0.5,
        );
        assert!((shown - expected.x).abs() < 1e-4, "shown {shown}");
    }

    fn nudge_sprite(mut sprites: Query<&mut Transform, With<Sprite>>) {
        for mut transform in &mut sprites {
            transform.translation.x += 0.1;
        }
    }
}
