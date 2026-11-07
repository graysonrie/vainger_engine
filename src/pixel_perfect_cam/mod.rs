//! Low-resolution game camera, upscaled into the window with nearest-neighbor sampling.
//!
//! [`PixelPerfectCameraPlugin`] spawns two cameras:
//!
//! - [`PixelPerfectCamera`] renders the world and Bevy UI into a canvas the size
//!   of the game resolution ([`GAME_LAYERS`], layer 0). Sprites with no render
//!   layer land here, which is what you want.
//! - [`OuterCamera`] draws that canvas into the window ([`OUTER_LAYERS`], layer 1).
//!
//! [`EditorUiCamera`] is a third camera, for egui only. It is the primary egui
//! context. Do not spawn another one, and do not put gameplay sprites on it.
//!
//! ### NOTE:
//! World sprites placed on [`OUTER_LAYERS`] skip the low-res canvas and are
//! drawn in window space, so they will not scale with the game pixels.
//!
//! ### NOTE:
//! The canvas is a fixed texture created at startup from
//! [`PixelPerfectCameraConfig`]. Changing `game_width` or `game_height` later
//! does not resize it.
//!
//! ### NOTE:
//! Upscale is nearest-neighbor, but it is only an even pixel grid when the
//! window size is an integer multiple of the game resolution.
//! [`crate::app::AppExt::setup_for_default_2d_pixel_game`] opens the window at
//! 1×. A window that is not a multiple of the canvas (including after a
//! resize) maps some game pixels to more screen pixels than others.

pub mod bounds;
mod config;
pub mod follow;
mod panner;
pub mod pointer;
pub mod screen_transition;
#[cfg(target_arch = "wasm32")]
mod web_color;

use crate::pixel_perfect_cam::bounds::CameraBounds;
use crate::prelude::*;
pub use config::*;
pub mod window;
pub use bounds::CameraBoundsInfo;
pub use follow::PixelPerfectCameraFollow;
#[allow(unused)]
pub use panner::CameraPanner2dBundle;

/// Camera control stages, chained in [`PostUpdate`]: [`Follow`](Self::Follow),
/// then [`Bounds`](Self::Bounds).
///
/// Move the followed entity in [`Update`] (or earlier in [`PostUpdate`]) so the
/// camera sees the position on the same frame. Follow does nothing useful
/// unless exactly one [`PixelPerfectCameraFollow`] exists; see that type.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PixelPerfectCameraControlsSet {
    /// Smooths [`PixelPerfectCamera`] toward [`PixelPerfectCameraFollow`].
    Follow,
    /// Clamps the camera to [`CameraBounds`], if that component is present.
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
#[derive(Component, Default)]
#[require(Transform)]
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
///
/// ### NOTE:
/// A system in [`PreUpdate`] sets `physical_rect` back to [`None`] every frame.
/// Write it every frame from [`Update`] or later, or the assignment from last
/// frame is gone before the cameras sync.
#[derive(Resource, Default, Clone, Copy)]
pub struct GameViewSlot {
    /// Physical pixels of the primary window. `None` uses the full window.
    pub physical_rect: Option<URect>,
}

/// Marker for the sprite that displays the low-res canvas in the high-res world.
#[derive(Component)]
pub struct GameCanvas;

/// Handle to the NES-resolution render target (hook for future palette post-process).
#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct GameCanvasImage(pub Handle<Image>);

/// Spawns the low-res canvas, the inner and outer cameras, and the follow / bounds systems.
///
/// [`Self::of_size`] is the usual constructor. The plugin also installs
/// panning ([`CameraPanner2dBundle`]), [`bounds::CameraBoundsPlugin`], and
/// [`follow::PixelPerfectCameraFollowPlugin`].
/// [`crate::app::AppExt::setup_for_default_2d_pixel_game`] adds this plugin.
pub struct PixelPerfectCameraPlugin {
    /// Canvas size copied into [`PixelPerfectCameraConfig`] at startup.
    pub config: PixelPerfectCameraConfig,
}

impl PixelPerfectCameraPlugin {
    /// Canvas and camera sized to `size`, in game pixels.
    ///
    /// Presets live in [`window`]: [`NES`], [`UFO_50`], [`UFO_50_MOBILE`],
    /// [`PIZZA_TOWER`], [`window::ANTONBLAST`].
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

/// The set that pixel perfect cameras are finalized in
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PixelPerfectCamCreationSet;

impl Plugin for PixelPerfectCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone());
        app.init_resource::<GameViewSlot>();
        app.add_plugins(panner::CameraPanner2dPlugin);
        app.add_plugins(bounds::CameraBoundsPlugin);
        app.add_plugins(screen_transition::ScreenTransitionPlugin);
        app.add_plugins(follow::PixelPerfectCameraFollowPlugin);

        app.configure_sets(PostUpdate, {
            use PixelPerfectCameraControlsSet::*;
            (Follow, Bounds).chain()
        });
        app.add_systems(Startup, (setup, configure_window_constraints));
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, web_color::tag_canvas_color_space);
        app.add_systems(PreUpdate, reset_game_view_slot);
        app.add_systems(
            PostUpdate,
            finalize_pixel_perfect_camera
                .before(CameraUpdateSystems)
                .in_set(PixelPerfectCamCreationSet),
        );
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

    let canvas_handle = images.add(canvas);
    commands.insert_resource(GameCanvasImage(canvas_handle.clone()));

    // let inner_projection = Projection::from(OrthographicProjection {
    //     scaling_mode: ScalingMode::Fixed {
    //         width: config.game_width,
    //         height: config.game_height,
    //     },
    //     ..OrthographicProjection::default_2d()
    // });
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
    // commands.spawn((
    //     Camera2d,
    //     Camera {
    //         order: -1,
    //         clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
    //         ..default()
    //     },
    //     RenderTarget::Image(canvas_handle.into()),
    //     inner_projection,
    //     Msaa::Off,
    //     PixelPerfectCamera,
    //     IsDefaultUiCamera,
    //     GAME_LAYERS.clone(),
    //     Transform::from_translation(Vec3::ZERO),
    // ));
}

fn finalize_pixel_perfect_camera(
    mut commands: Commands,
    q: Query<Entity, Added<PixelPerfectCamera>>,
    config: Res<PixelPerfectCameraConfig>,
    game_canvas: Res<GameCanvasImage>,
) {
    for ent in q {
        let inner_projection = Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: config.game_width,
                height: config.game_height,
            },
            ..OrthographicProjection::default_2d()
        });

        commands.entity(ent).insert((
            Camera2d,
            Camera {
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.1, 0.1, 0.1)),
                ..default()
            },
            RenderTarget::Image(game_canvas.0.clone().into()),
            inner_projection,
            Msaa::Off,
            IsDefaultUiCamera,
            GAME_LAYERS.clone(),
        ));
    }
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

/// Read and write the [`PixelPerfectCamera`] transform from a system parameter.
///
/// There is exactly one inner camera. This param fails the system if that
/// camera is missing.
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

    /// Camera center X, in world pixels.
    pub fn get_x(&self) -> f32 {
        self.transform().translation.x
    }
    /// Camera center Y, in world pixels.
    pub fn get_y(&self) -> f32 {
        self.transform().translation.y
    }
    /// World-space center of the view, in pixels. Y-up.
    pub fn get_xy(&self) -> Vec2 {
        self.transform().get_2d_pos()
    }
    /// Move the camera center to `pos`. Z is set to `0`.
    ///
    /// [`CameraBounds`] are applied later in [`PostUpdate`], so a position
    /// outside the bounds will be pulled back before the frame renders.
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
    /// Canvas width in pixels. This is the config value, not the OS window.
    pub fn get_view_width(&self) -> f32 {
        self.config.game_width
    }
    /// Canvas height in pixels. This is the config value, not the OS window.
    pub fn get_view_height(&self) -> f32 {
        self.config.game_height
    }
    /// Half the canvas size, `(width / 2, height / 2)`.
    pub fn viewport_dimensions_half(&self) -> Vec2 {
        self.config.viewport_dimensions_half()
    }

    /// Replace [`CameraBounds`] on the inner camera.
    ///
    /// Each argument is an edge of the allowed view in world pixels, or [`None`]
    /// to leave that side open. These are viewport edges, not limits on the
    /// camera center: `xmin` is the leftmost pixel the view may show.
    ///
    /// ### NOTE:
    /// This inserts the component again on every call. A region smaller than
    /// the canvas on an axis pins the camera to the middle of that region.
    /// See [`CameraBounds`].
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
