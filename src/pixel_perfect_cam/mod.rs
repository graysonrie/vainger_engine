pub mod bounds;
mod config;
pub mod follow;
mod panner;
pub mod pointer;

use crate::pixel_perfect_cam::bounds::CameraBounds;
use crate::prelude::*;
pub use config::*;
pub mod window;
pub use bounds::CameraBoundsInfo;
pub use follow::PixelPerfectCameraFollow;
#[allow(unused)]
pub use panner::CameraPanner2dBundle;

/// These systems run in the Update event
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PixelPerfectCameraControlsSet {
    Follow,
    Bounds,
}

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::render::render_resource::{
    BlendState, Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::{
    camera::{CameraOutputMode, CameraUpdateSystems, ScalingMode, Viewport},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{EguiGlobalSettings, EguiPostUpdateSet, PrimaryEguiContext};

#[allow(unused_imports)]
pub use window::{NES, PIZZA_TOWER, UFO_50, UFO_50_MOBILE, WindowPluginExt};

/// Low-resolution camera that renders the game (and Bevy UI) into [`GameCanvasImage`].
///
/// This is the movable world camera: panning and stage-centering target it.
#[derive(Component)]
pub struct PixelPerfectCamera;

/// Render layers for world content and Bevy UI drawn into the low-res canvas.
pub const GAME_LAYERS: RenderLayers = RenderLayers::layer(0);

/// Render layers for the upscaled canvas sprite and the window camera.
pub const OUTER_LAYERS: RenderLayers = RenderLayers::layer(1);

/// Window camera that draws the upscaled canvas into [`GameViewSlot`].
#[derive(Component)]
pub struct OuterCamera;

/// Full-window camera can host editor egui outside the game viewport.
#[derive(Component)]
pub struct EditorUiCamera;

/// Cameras that draw to the primary window surface (not the low-res canvas).
///
/// Disabled while the surface is 0×0 (minimize / some resize frames) so wgpu
/// is not asked to draw into an invalid swap chain.
#[derive(Component)]
struct WindowSurfaceCamera;

/// Physical window rect the game canvas may occupy (leftover after editor panels).
///
/// `None` letterboxes into the full window.
#[derive(Resource, Default, Clone, Copy)]
pub struct GameViewSlot {
    pub physical_rect: Option<URect>,
}

/// Marker for the sprite that displays the low-res canvas in the high-res world.
#[derive(Component)]
pub struct GameCanvas;

/// Handle to the NES-resolution render target (hook for future palette post-process).
#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct GameCanvasImage(pub Handle<Image>);

pub struct PixelPerfectCameraPlugin {
    pub config: PixelPerfectCameraConfig,
}

impl PixelPerfectCameraPlugin {
    /// Tip: use `pixel_perfect_cam_2d::window_size_constants` to pick a screen size
    pub fn of_size(size: UVec2) -> Self {
        let game_width = size.x as f32;
        let game_height = size.y as f32;
        Self {
            config: PixelPerfectCameraConfig {
                game_width,
                game_height,
            },
        }
    }
}

impl Plugin for PixelPerfectCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.init_resource::<GameViewSlot>();
        app.add_plugins(panner::CameraPanner2dPlugin);
        app.add_plugins(bounds::CameraBoundsPlugin);
        app.add_plugins(follow::PixelPerfectCameraFollowPlugin);

        app.configure_sets(PostUpdate, {
            use PixelPerfectCameraControlsSet::*;
            ( Follow, Bounds).chain()
        });
        app.add_systems(Startup, (setup, configure_window_constraints));
        app.add_systems(PreUpdate, reset_game_view_slot);
        app.add_systems(
            PostUpdate,
            sync_window_cameras
                .before(CameraUpdateSystems)
                .after(EguiPostUpdateSet::EndPass),
        );
    }
}

/// Spawns the low-res render target, outer window camera, canvas sprite, and inner game camera.
fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    config: Res<PixelPerfectCameraConfig>,
) {
    // Do not auto-bind egui to the first camera; EditorUiCamera hosts the primary context.
    egui_global_settings.auto_create_primary_context = false;

    let width = config.game_width as u32;
    let height = config.game_height as u32;
    let canvas_size = Extent3d {
        width,
        height,
        ..default()
    };

    let mut canvas = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("game_canvas"),
            size: canvas_size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
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

    let canvas_handle = images.add(canvas);
    commands.insert_resource(GameCanvasImage(canvas_handle.clone()));

    let inner_projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::Fixed {
            width: config.game_width,
            height: config.game_height,
        },
        ..OrthographicProjection::default_2d()
    });
    // Letterbox in projection space so we do not need a custom GPU viewport
    // (those can exceed the swap chain for a frame and DeviceLost the renderer).
    let outer_projection = Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::AutoMin {
            min_width: config.game_width,
            min_height: config.game_height,
        },
        ..OrthographicProjection::default_2d()
    });

    // Full-window clear so editor chrome (and letterbox bars) are not clipped to the game view.
    commands.spawn((
        Camera2d,
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.08, 0.08, 0.1)),
            ..default()
        },
        Msaa::Off,
        RenderLayers::none(),
        WindowSurfaceCamera,
    ));

    // Outer camera: draws the upscaled canvas into the leftover game slot.
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        outer_projection,
        Msaa::Off,
        OuterCamera,
        OUTER_LAYERS.clone(),
        WindowSurfaceCamera,
    ));

    // Editor egui overlays the whole window and is not bound to the game viewport.
    commands.spawn((
        Camera2d,
        Camera {
            order: 2,
            output_mode: CameraOutputMode::Write {
                blend_state: Some(BlendState::ALPHA_BLENDING),
                clear_color: ClearColorConfig::None,
            },
            clear_color: ClearColorConfig::Custom(Color::NONE),
            ..default()
        },
        Msaa::Off,
        RenderLayers::none(),
        PrimaryEguiContext,
        EditorUiCamera,
        WindowSurfaceCamera,
    ));

    // The render target for the game canvas (what the inner world will draw into)
    commands.spawn((
        Sprite {
            image: canvas_handle.clone(),
            custom_size: Some(Vec2::new(config.game_width, config.game_height)),
            ..default()
        },
        GameCanvas,
        OUTER_LAYERS.clone(),
    ));

    // Inner camera: world + Bevy UI → NES-resolution canvas.
    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
            ..default()
        },
        RenderTarget::Image(canvas_handle.into()),
        inner_projection,
        Msaa::Off,
        PixelPerfectCamera,
        IsDefaultUiCamera,
        GAME_LAYERS.clone(),
        Transform::from_translation(Vec3::ZERO),
    ));
}

/// Ensures the window cannot shrink below the configured game resolution.
fn configure_window_constraints(
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    config: Res<PixelPerfectCameraConfig>,
) {
    window.resize_constraints = WindowResizeConstraints {
        min_width: config.game_width,
        min_height: config.game_height,
        ..window.resize_constraints
    }
}

/// Keeps window cameras off 0×0 surfaces and only applies a GPU viewport for [`GameViewSlot`].
///
/// Letterboxing uses [`ScalingMode::AutoMin`] on the outer camera, not a custom viewport.
/// Oversized viewports submitted after `camera_system` can exceed the swap chain and
/// `DeviceLost` the renderer.
fn sync_window_cameras(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cameras: Query<(&mut Camera, Option<&OuterCamera>), With<WindowSurfaceCamera>>,
    slot: Res<GameViewSlot>,
) {
    let win = UVec2::new(window.physical_width(), window.physical_height());

    for (mut camera, outer) in &mut cameras {
        let Some(surface) = safe_surface_size(win, camera.physical_target_size()) else {
            camera.is_active = false;
            if outer.is_some() {
                camera.viewport = None;
            }
            continue;
        };

        camera.is_active = true;
        if outer.is_some() {
            camera.viewport = slot
                .physical_rect
                .and_then(|rect| clamped_slot_viewport(rect, surface));
        }
    }
}

/// Conservative size that cannot exceed either the window or last known render target.
fn safe_surface_size(win: UVec2, target: Option<UVec2>) -> Option<UVec2> {
    let target = target.unwrap_or(win);
    let size = UVec2::new(win.x.min(target.x), win.y.min(target.y));
    (size.x > 0 && size.y > 0).then_some(size)
}

fn clamped_slot_viewport(slot: URect, surface: UVec2) -> Option<Viewport> {
    let origin = UVec2::new(slot.min.x.min(surface.x), slot.min.y.min(surface.y));
    let mut viewport = Viewport {
        physical_position: origin,
        physical_size: UVec2::new(
            slot.width().min(surface.x.saturating_sub(origin.x)),
            slot.height().min(surface.y.saturating_sub(origin.y)),
        ),
        depth: 0.0..1.0,
    };
    viewport.clamp_to_size(surface);
    (viewport.physical_size.x > 0 && viewport.physical_size.y > 0).then_some(viewport)
}

fn reset_game_view_slot(mut slot: ResMut<GameViewSlot>) {
    slot.physical_rect = None;
}

#[derive(SystemParam)]
pub struct PixelPerfectCameraQuery<'w, 's> {
    commands: Commands<'w, 's>,
    camera: Single<'w, 's, (Entity, &'static mut Transform), With<PixelPerfectCamera>>,
    config: Res<'w, PixelPerfectCameraConfig>,
}

#[allow(unused)]
impl PixelPerfectCameraQuery<'_, '_> {
    fn transform(&self) -> &Transform {
        &self.camera.1
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.camera.1
    }

    pub fn get_x(&self) -> f32 {
        self.transform().translation.x
    }
    pub fn get_y(&self) -> f32 {
        self.transform().translation.y
    }
    /// Returns the current center of what the camera is looking at
    pub fn get_xy(&self) -> Vec2 {
        self.transform().get_2d_pos()
    }
    pub fn set_xy(&mut self, pos: Vec2) {
        self.transform_mut().translation = pos.extend(0.);
    }
    /// Top-left corner of the current viewport in world space (Bevy Y-up).
    #[inline]
    pub fn viewport_top_left(&self) -> Vec2 {
        self.get_xy() + self.config.viewport_top_left()
    }
    /// Place the camera so `top_left` is the top-left of the viewport (Bevy Y-up).
    pub fn set_viewport_top_left(&mut self, top_left: Vec2) {
        self.set_xy(top_left - self.config.viewport_top_left());
    }
    pub fn get_view_width(&self) -> f32 {
        self.config.game_width
    }
    pub fn get_view_height(&self) -> f32 {
        self.config.game_height
    }
    pub fn viewport_dimensions_half(&self) -> Vec2 {
        self.config.viewport_dimensions_half()
    }

    pub fn set_bounds(
        &mut self,
        xmin: Option<f32>,
        xmax: Option<f32>,
        ymin: Option<f32>,
        ymax: Option<f32>,
    ) {
        self.commands.entity(self.camera.0).insert(CameraBounds {
            xmin,
            xmax,
            ymin,
            ymax,
        });
    }
}
