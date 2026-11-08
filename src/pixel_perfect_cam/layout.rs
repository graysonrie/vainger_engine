//! Packs every [`PixelPerfectCamera`](super::PixelPerfectCamera) into the base canvas.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{RenderTarget, ScalingMode};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::prelude::*;

use super::{
    CameraZoom, GAME_LAYERS, GameCanvas, OUTER_LAYERS, OuterCamera, PixelPerfectCamCreationSet,
};

/// First render layer used for a screen-fade overlay.
///
/// Layer 0 is the world ([`GAME_LAYERS`]). Layer 1 is the upscaled canvas
/// ([`OUTER_LAYERS`]). Each screen's fade quad uses `2 + slot`.
const FADE_LAYER_START: usize = 2;

/// Spawn order of a [`super::PixelPerfectCamera`].
///
/// Lower values are placed earlier in the grid (left, then down). Removing a
/// camera does not renumber the ones that remain, so a later camera never
/// jumps ahead of an earlier one.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScreenIndex(pub u32);

/// This camera's cell on the base canvas, and the texture it renders into.
///
/// `rect.min` is the top-left of the cell. Y grows downward, so row 0 is the
/// top of the base canvas.
#[derive(Component, Clone, Debug)]
pub struct ScreenSlot {
    /// Pixel rect inside the base resolution.
    pub rect: URect,
    /// Low-resolution render target for this cell.
    pub image: Handle<Image>,
    pub(crate) canvas: Entity,
}

impl ScreenSlot {
    /// World size this camera can see. Cell pixels divided by `zoom`.
    pub fn visible_world_size(&self, zoom: f32) -> Vec2 {
        let zoom = effective_zoom(zoom);
        Vec2::new(
            self.rect.width() as f32 / zoom,
            self.rect.height() as f32 / zoom,
        )
    }
}

/// `zoom` when it is positive, otherwise `1.0`.
pub(crate) fn effective_zoom(zoom: f32) -> f32 {
    if zoom > 0.0 {
        zoom
    } else {
        warn_once!("CameraZoom must be greater than 0; using 1.0");
        1.0
    }
}

#[derive(Clone, Copy)]
struct ScreenOrderEntry {
    camera: Entity,
    index: u32,
}

#[derive(Resource, Default)]
struct ScreenOrder {
    next: u32,
    entries: Vec<ScreenOrderEntry>,
}

impl ScreenOrder {
    fn assign(&mut self, entity: Entity) {
        if self.entries.iter().any(|entry| entry.camera == entity) {
            return;
        }
        let index = self.next;
        self.next = self.next.saturating_add(1);
        self.entries.push(ScreenOrderEntry {
            camera: entity,
            index,
        });
    }

    fn forget(&mut self, entity: Entity) {
        self.entries.retain(|entry| entry.camera != entity);
    }
}

struct PlannedFade {
    camera: Entity,
    visible: Vec2,
    layers: RenderLayers,
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<ScreenOrder>()
        .add_observer(note_camera)
        .add_observer(forget_camera)
        .add_systems(
            PostUpdate,
            layout_cameras.in_set(PixelPerfectCamCreationSet),
        );
}

fn note_camera(event: On<Add, super::PixelPerfectCamera>, mut order: ResMut<ScreenOrder>) {
    order.assign(event.entity);
}

fn forget_camera(
    event: On<Remove, super::PixelPerfectCamera>,
    mut order: ResMut<ScreenOrder>,
    slots: Query<&ScreenSlot>,
    mut commands: Commands,
) {
    order.forget(event.entity);
    if let Ok(slot) = slots.get(event.entity) {
        commands.entity(slot.canvas).try_despawn();
    }
}

fn layout_cameras(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    config: Res<PixelPerfectCameraConfig>,
    mut order: ResMut<ScreenOrder>,
    mut cameras: Query<
        (
            Option<&ScreenIndex>,
            &CameraZoom,
            Option<&mut ScreenSlot>,
            Option<&mut Projection>,
            Option<&mut RenderLayers>,
            Has<IsDefaultUiCamera>,
        ),
        (
            With<super::PixelPerfectCamera>,
            Without<GameCanvas>,
            Without<super::screen_transition::ScreenFadeOverlay>,
        ),
    >,
    mut canvases: Query<
        (&mut Transform, &mut Sprite),
        (
            With<GameCanvas>,
            Without<super::PixelPerfectCamera>,
            Without<super::screen_transition::ScreenFadeOverlay>,
        ),
    >,
    mut overlays: Query<
        (&ChildOf, &mut Sprite, &mut RenderLayers),
        (
            With<super::screen_transition::ScreenFadeOverlay>,
            Without<GameCanvas>,
            Without<super::PixelPerfectCamera>,
        ),
    >,
    mut outer: Query<&mut Projection, (With<OuterCamera>, Without<super::PixelPerfectCamera>)>,
) {
    sync_outer_letterbox(&config, &mut outer);

    order.entries.retain(|entry| cameras.contains(entry.camera));
    let mut ordered = order.entries.clone();
    ordered.sort_by_key(|entry| entry.index);

    let width = positive_pixels(config.game_width);
    let height = positive_pixels(config.game_height);
    if width == 0 || height == 0 {
        return;
    }

    let rects = pack_grid(width, height, ordered.len());
    let mut planned = Vec::with_capacity(ordered.len());

    for (slot_index, (entry, rect)) in ordered.iter().copied().zip(rects).enumerate() {
        let ScreenOrderEntry {
            camera: entity,
            index: screen_index,
        } = entry;
        let Ok((index, zoom, slot, projection, render_layers, has_ui)) = cameras.get_mut(entity)
        else {
            continue;
        };

        if index.map(|index| index.0) != Some(screen_index) {
            commands.entity(entity).insert(ScreenIndex(screen_index));
        }

        let visible = Vec2::new(
            rect.width() as f32 / effective_zoom(zoom.0),
            rect.height() as f32 / effective_zoom(zoom.0),
        );
        let center = cell_center(rect, config.game_width, config.game_height);
        let layers = camera_layers(slot_index);

        if let Some(mut slot) = slot {
            resize_slot(&mut images, &mut slot, rect);
            place_canvas(&mut canvases, slot.canvas, center, rect);
            if let Some(mut projection) = projection {
                apply_visible_projection(&mut projection, visible);
            }
            if render_layers.as_deref() != Some(&layers) {
                commands.entity(entity).insert(layers);
            }
        } else {
            let image = images.add(canvas_image(rect.size()));
            let canvas = commands
                .spawn((
                    Sprite {
                        image: image.clone(),
                        custom_size: Some(Vec2::new(rect.width() as f32, rect.height() as f32)),
                        ..default()
                    },
                    Transform::from_translation(center.extend(0.0)),
                    GameCanvas,
                    OUTER_LAYERS.clone(),
                ))
                .id();
            commands.entity(entity).insert((
                ScreenSlot {
                    rect,
                    image: image.clone(),
                    canvas,
                },
                Camera2d,
                inner_camera(),
                RenderTarget::Image(image.into()),
                projection_for(visible),
                Msaa::Off,
                layers,
            ));
        }

        if slot_index == 0 && !has_ui {
            commands.entity(entity).insert(IsDefaultUiCamera);
        } else if slot_index != 0 && has_ui {
            commands.entity(entity).remove::<IsDefaultUiCamera>();
        }

        planned.push(PlannedFade {
            camera: entity,
            visible,
            layers: fade_layers(slot_index),
        });
    }

    for (child_of, mut sprite, mut layers) in &mut overlays {
        let Some(planned) = planned.iter().find(|fade| fade.camera == child_of.0) else {
            continue;
        };
        if sprite.custom_size != Some(planned.visible) {
            sprite.custom_size = Some(planned.visible);
        }
        if layers.as_ref() != &planned.layers {
            *layers = planned.layers.clone();
        }
    }
}

fn sync_outer_letterbox(
    config: &PixelPerfectCameraConfig,
    outer: &mut Query<&mut Projection, (With<OuterCamera>, Without<super::PixelPerfectCamera>)>,
) {
    let mode = ScalingMode::AutoMin {
        min_width: config.game_width,
        min_height: config.game_height,
    };
    for mut projection in outer {
        let Projection::Orthographic(ortho) = projection.as_mut() else {
            continue;
        };
        if !matches!(
            ortho.scaling_mode,
            ScalingMode::AutoMin { min_width, min_height }
                if min_width == config.game_width && min_height == config.game_height
        ) {
            ortho.scaling_mode = mode;
        }
    }
}

fn resize_slot(images: &mut Assets<Image>, slot: &mut ScreenSlot, rect: URect) {
    if slot.rect == rect {
        return;
    }
    if let Some(mut image) = images.get_mut(&slot.image) {
        image.resize(Extent3d {
            width: rect.width().max(1),
            height: rect.height().max(1),
            depth_or_array_layers: 1,
        });
    }
    slot.rect = rect;
}

fn place_canvas(
    canvases: &mut Query<
        (&mut Transform, &mut Sprite),
        (
            With<GameCanvas>,
            Without<super::PixelPerfectCamera>,
            Without<super::screen_transition::ScreenFadeOverlay>,
        ),
    >,
    canvas: Entity,
    center: Vec2,
    rect: URect,
) {
    let Ok((mut transform, mut sprite)) = canvases.get_mut(canvas) else {
        return;
    };
    let translation = center.extend(0.0);
    if transform.translation != translation {
        transform.translation = translation;
    }
    let size = Vec2::new(rect.width() as f32, rect.height() as f32);
    if sprite.custom_size != Some(size) {
        sprite.custom_size = Some(size);
    }
}

fn apply_visible_projection(projection: &mut Projection, visible: Vec2) {
    if let Projection::Orthographic(ortho) = projection
        && matches!(
            ortho.scaling_mode,
            ScalingMode::Fixed { width, height } if width == visible.x && height == visible.y
        )
    {
        return;
    }
    *projection = projection_for(visible);
}

fn projection_for(visible: Vec2) -> Projection {
    Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: visible.x,
            height: visible.y,
        },
        ..OrthographicProjection::default_2d()
    })
}

fn inner_camera() -> Camera {
    Camera {
        order: -1,
        clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
        ..default()
    }
}

fn camera_layers(slot_index: usize) -> RenderLayers {
    GAME_LAYERS.clone().with(FADE_LAYER_START + slot_index)
}

fn fade_layers(slot_index: usize) -> RenderLayers {
    RenderLayers::none().with(FADE_LAYER_START + slot_index)
}

fn canvas_image(size: UVec2) -> Image {
    let canvas_size = Extent3d {
        width: size.x.max(1),
        height: size.y.max(1),
        depth_or_array_layers: 1,
    };
    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("game_canvas"),
            size: canvas_size,
            dimension: TextureDimension::D2,
            // RGBA sRGB, not BGRA. WebGL has no BGRA sRGB format, so a BGRA target
            // takes a different gamma path in the browser than on desktop.
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    canvas.resize(canvas_size);
    canvas
}

fn positive_pixels(value: f32) -> u32 {
    if value <= 0.0 { 0 } else { value as u32 }
}

/// Center of `rect` in outer-camera space. The base canvas is centered on the origin, Y-up.
fn cell_center(rect: URect, game_width: f32, game_height: f32) -> Vec2 {
    let width = rect.width() as f32;
    let height = rect.height() as f32;
    Vec2::new(
        rect.min.x as f32 + width * 0.5 - game_width * 0.5,
        game_height * 0.5 - (rect.min.y as f32 + height * 0.5),
    )
}

/// Integer cells covering `width` by `height`, row-major, top to bottom.
///
/// `columns = ceil(sqrt(n))` and `rows = ceil(n / columns)`. One camera is the
/// full frame. Two are left and right. Three is a 2×2 grid with the
/// bottom-right cell empty. Remainder pixels go to the earliest cells.
fn pack_grid(width: u32, height: u32, n: usize) -> Vec<URect> {
    if n == 0 {
        return Vec::new();
    }
    let columns = (n as f32).sqrt().ceil() as usize;
    let rows = n.div_ceil(columns);
    let column_widths = split_sizes(width, columns);
    let row_heights = split_sizes(height, rows);
    let mut rects = Vec::with_capacity(n);
    for index in 0..n {
        let column = index % columns;
        let row = index / columns;
        let x = column_widths[..column].iter().sum();
        let y = row_heights[..row].iter().sum();
        rects.push(URect::new(
            x,
            y,
            x + column_widths[column],
            y + row_heights[row],
        ));
    }
    rects
}

fn split_sizes(total: u32, parts: usize) -> Vec<u32> {
    let parts_u = parts as u32;
    let base = total / parts_u;
    let remainder = total % parts_u;
    (0..parts)
        .map(|index| base + u32::from((index as u32) < remainder))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Image>()
            .insert_resource(PixelPerfectCameraConfig {
                game_width: 320.0,
                game_height: 180.0,
            });
        build(&mut app);
        app
    }

    #[test]
    fn spawning_cameras_splits_targets_and_zoom_changes_the_view() {
        let mut app = app();
        let first = app.world_mut().spawn(PixelPerfectCamera).id();
        let second = app.world_mut().spawn(PixelPerfectCamera).id();
        app.update();

        let first_slot = app.world().get::<ScreenSlot>(first).unwrap().clone();
        let second_slot = app.world().get::<ScreenSlot>(second).unwrap().clone();
        assert_eq!(first_slot.rect, URect::new(0, 0, 160, 180));
        assert_eq!(second_slot.rect, URect::new(160, 0, 320, 180));
        assert_ne!(first_slot.canvas, second_slot.canvas);
        assert!(app.world().get::<IsDefaultUiCamera>(first).is_some());
        assert!(app.world().get::<IsDefaultUiCamera>(second).is_none());
        assert_fixed_view(&app, first, 160.0, 180.0);

        app.world_mut().entity_mut(first).insert(CameraZoom(2.0));
        app.update();
        assert_fixed_view(&app, first, 80.0, 90.0);

        let removed_canvas = second_slot.canvas;
        app.world_mut().entity_mut(second).despawn();
        app.update();
        let first_slot = app.world().get::<ScreenSlot>(first).unwrap();
        assert_eq!(first_slot.rect, URect::new(0, 0, 320, 180));
        assert!(app.world().get_entity(removed_canvas).is_err());
        assert_fixed_view(&app, first, 160.0, 90.0);
    }

    fn assert_fixed_view(app: &App, camera: Entity, width: f32, height: f32) {
        let Projection::Orthographic(ortho) = app.world().get::<Projection>(camera).unwrap() else {
            panic!("camera {camera} has no orthographic projection");
        };
        assert!(
            matches!(
                ortho.scaling_mode,
                ScalingMode::Fixed { width: view_width, height: view_height }
                    if view_width == width && view_height == height
            ),
            "expected {width}x{height}, got {:?}",
            ortho.scaling_mode
        );
    }

    #[test]
    fn one_camera_fills_the_base_canvas() {
        let cells = pack_grid(320, 180, 1);
        assert_eq!(cells, vec![URect::new(0, 0, 320, 180)]);
    }

    #[test]
    fn two_cameras_split_left_and_right() {
        let cells = pack_grid(320, 180, 2);
        assert_eq!(
            cells,
            vec![URect::new(0, 0, 160, 180), URect::new(160, 0, 320, 180)]
        );
    }

    #[test]
    fn three_cameras_leave_the_bottom_right_empty() {
        let cells = pack_grid(320, 180, 3);
        assert_eq!(
            cells,
            vec![
                URect::new(0, 0, 160, 90),
                URect::new(160, 0, 320, 90),
                URect::new(0, 90, 160, 180),
            ]
        );
    }

    #[test]
    fn four_cameras_fill_a_grid() {
        let cells = pack_grid(320, 180, 4);
        assert_eq!(cells.len(), 4);
        assert_eq!(cells[3], URect::new(160, 90, 320, 180));
        let area: u32 = cells.iter().map(|rect| rect.width() * rect.height()).sum();
        assert_eq!(area, 320 * 180);
    }

    #[test]
    fn remainder_pixels_go_to_the_earliest_cells() {
        let cells = pack_grid(321, 181, 2);
        assert_eq!(cells[0].width(), 161);
        assert_eq!(cells[1].width(), 160);
        assert_eq!(cells[0].height(), 181);
        assert_eq!(cells[0].width() + cells[1].width(), 321);
    }
}
