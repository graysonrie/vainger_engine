pub mod bounds;
mod config;
pub mod follow;
mod layout;
mod panner;
pub mod pointer;
pub mod screen_transition;
#[cfg(target_arch = "wasm32")]
mod web_color;

use crate::pixel_perfect_cam::bounds::CameraBounds;
use crate::prelude::*;
pub use config::*;
use thiserror::Error;
pub mod window;
pub use bounds::CameraBoundsInfo;
pub use follow::PixelPerfectCameraFollow;
pub use layout::{ScreenIndex, ScreenSlot};
#[allow(unused)]
pub use panner::CameraPanner2dBundle;

/// These systems run in the Update event
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PixelPerfectCameraControlsSet {
    Follow,
    Bounds,
}
use bevy::camera::visibility::RenderLayers;
use bevy::render::render_resource::BlendState;
use bevy::{
    camera::{CameraOutputMode, CameraUpdateSystems, ScalingMode, Viewport},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{EguiGlobalSettings, EguiPostUpdateSet, PrimaryEguiContext};

#[allow(unused_imports)]
pub use window::{NES, PIZZA_TOWER, UFO_50, UFO_50_MOBILE, WindowPluginExt};

/// Low-resolution camera that renders the world into one cell of the base canvas.
///
/// Spawn one per screen. The layout pass packs them into a grid and gives each
/// its own render target. [`CameraZoom`] is required and defaults to `1.0`.
#[derive(Component, Default)]
#[require(Transform, CameraZoom)]
pub struct PixelPerfectCamera;

/// How much world this camera sees, relative to its cell.
///
/// `1.0` maps one world pixel to one canvas pixel. `2.0` shows half of the cell
/// and draws each world pixel as a 2×2 block. The value is applied directly, so
/// a fraction does not land on whole pixels and the picture shimmers.
///
/// ### NOTE:
/// Values `<= 0` are treated as `1.0`.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct CameraZoom(pub f32);

impl Default for CameraZoom {
    fn default() -> Self {
        Self(1.0)
    }
}

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

        app.configure_sets(
            PostUpdate,
            (
                PixelPerfectCamCreationSet,
                PixelPerfectCameraControlsSet::Follow,
                PixelPerfectCameraControlsSet::Bounds,
            )
                .chain()
                .before(CameraUpdateSystems),
        );
        layout::build(app);
        app.add_systems(Startup, (setup, configure_window_constraints));
        #[cfg(target_arch = "wasm32")]
        app.add_systems(Update, web_color::tag_canvas_color_space);
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
    mut egui_global_settings: ResMut<EguiGlobalSettings>,
    config: Res<PixelPerfectCameraConfig>,
) {
    // Do not auto-bind egui to the first camera; EditorUiCamera hosts the primary context.
    egui_global_settings.auto_create_primary_context = false;
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

/// Why a [`PixelPerfectCameraQuery`] lookup failed.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CameraLookupError {
    /// No [`PixelPerfectCamera`] is spawned.
    #[error("no pixel-perfect camera is spawned")]
    Missing,
    /// `only` was used while more than one camera exists.
    #[error("expected exactly one pixel-perfect camera, found {found}")]
    NotExactlyOne {
        /// How many cameras were spawned.
        found: usize,
    },
    /// `camera` is not a [`PixelPerfectCamera`].
    #[error("pixel-perfect camera {entity} does not exist")]
    Unknown {
        /// The entity that was requested.
        entity: Entity,
    },
    /// The camera exists, but this frame's layout pass has not given it a cell yet.
    #[error("pixel-perfect camera {entity} has no screen cell yet")]
    SlotNotReady {
        /// The camera that is still waiting for layout.
        entity: Entity,
    },
}

#[derive(SystemParam)]
pub struct PixelPerfectCameraQuery<'w, 's> {
    commands: Commands<'w, 's>,
    cameras: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Transform,
            &'static CameraZoom,
            Option<&'static ScreenSlot>,
        ),
        With<PixelPerfectCamera>,
    >,
}

struct CameraParts<'a> {
    transform: &'a Transform,
    zoom: &'a CameraZoom,
    slot: Option<&'a ScreenSlot>,
}

struct CameraPartsMut<'a> {
    transform: Mut<'a, Transform>,
}

#[allow(unused)]
impl PixelPerfectCameraQuery<'_, '_> {
    /// The only camera. Fails when there are zero or several.
    pub fn only(&self) -> Result<Entity, CameraLookupError> {
        let mut iter = self.cameras.iter();
        let Some((entity, _, _, _)) = iter.next() else {
            return Err(CameraLookupError::Missing);
        };
        if iter.next().is_some() {
            let found = 2 + iter.count();
            return Err(CameraLookupError::NotExactlyOne { found });
        }
        Ok(entity)
    }

    pub fn get_x(&self) -> Result<f32, CameraLookupError> {
        self.get_x_of(self.only()?)
    }

    /// Camera center X for `camera`, in world pixels.
    pub fn get_x_of(&self, camera: Entity) -> Result<f32, CameraLookupError> {
        Ok(self.get_xy_of(camera)?.x)
    }
    pub fn get_y(&self) -> Result<f32, CameraLookupError> {
        self.get_y_of(self.only()?)
    }

    /// Camera center Y for `camera`, in world pixels.
    pub fn get_y_of(&self, camera: Entity) -> Result<f32, CameraLookupError> {
        Ok(self.get_xy_of(camera)?.y)
    }
    /// Returns the current center of what the camera is looking at
    pub fn get_xy(&self) -> Result<Vec2, CameraLookupError> {
        self.get_xy_of(self.only()?)
    }

    /// World-space center of `camera`, in pixels. Y-up.
    pub fn get_xy_of(&self, camera: Entity) -> Result<Vec2, CameraLookupError> {
        let camera = self.camera_ref(camera)?;
        Ok(camera.transform.get_2d_pos())
    }
    pub fn set_xy(&mut self, pos: Vec2) -> Result<(), CameraLookupError> {
        self.set_xy_of(self.only()?, pos)
    }

    /// Move `camera`'s center to `pos`. Z is set to `0`.
    ///
    /// [`CameraBounds`] are applied later in [`PostUpdate`], so a position
    /// outside the bounds will be pulled back before the frame renders.
    pub fn set_xy_of(&mut self, camera: Entity, pos: Vec2) -> Result<(), CameraLookupError> {
        let mut camera = self.camera_mut(camera)?;
        camera.transform.translation = pos.extend(0.);
        Ok(())
    }
    /// Top-left corner of the current viewport in world space (Bevy Y-up).
    #[inline]
    pub fn viewport_top_left(&self) -> Result<Vec2, CameraLookupError> {
        self.viewport_top_left_of(self.only()?)
    }

    /// Top-left corner of `camera`'s visible view, in world space (Y-up).
    ///
    /// The visible size is the cell divided by [`CameraZoom`], not the base canvas.
    pub fn viewport_top_left_of(&self, camera: Entity) -> Result<Vec2, CameraLookupError> {
        let half = self.visible_half(camera)?;
        Ok(self.get_xy_of(camera)? + Vec2::new(-half.x, half.y))
    }
    /// Place the camera so `top_left` is the top-left of the viewport (Bevy Y-up).
    pub fn set_viewport_top_left(&mut self, top_left: Vec2) -> Result<(), CameraLookupError> {
        self.set_viewport_top_left_of(self.only()?, top_left)
    }

    /// Place `camera` so `top_left` is the top-left of its view (Y-up).
    pub fn set_viewport_top_left_of(
        &mut self,
        camera: Entity,
        top_left: Vec2,
    ) -> Result<(), CameraLookupError> {
        let half = self.visible_half(camera)?;
        self.set_xy_of(camera, top_left - Vec2::new(-half.x, half.y))
    }
    pub fn get_view_width(&self) -> Result<f32, CameraLookupError> {
        self.get_view_width_of(self.only()?)
    }

    /// Visible width of `camera`, in world pixels.
    pub fn get_view_width_of(&self, camera: Entity) -> Result<f32, CameraLookupError> {
        Ok(self.visible_size(camera)?.x)
    }
    pub fn get_view_height(&self) -> Result<f32, CameraLookupError> {
        self.get_view_height_of(self.only()?)
    }

    /// Visible height of `camera`, in world pixels.
    pub fn get_view_height_of(&self, camera: Entity) -> Result<f32, CameraLookupError> {
        Ok(self.visible_size(camera)?.y)
    }
    pub fn viewport_dimensions_half(&self) -> Result<Vec2, CameraLookupError> {
        self.viewport_dimensions_half_of(self.only()?)
    }

    /// Half of `camera`'s visible size, `(width / 2, height / 2)`.
    pub fn viewport_dimensions_half_of(&self, camera: Entity) -> Result<Vec2, CameraLookupError> {
        self.visible_half(camera)
    }

    pub fn set_bounds(
        &mut self,
        xmin: Option<f32>,
        xmax: Option<f32>,
        ymin: Option<f32>,
        ymax: Option<f32>,
    ) -> Result<(), CameraLookupError> {
        self.set_bounds_of(self.only()?, xmin, xmax, ymin, ymax)
    }

    /// Replace [`CameraBounds`] on `camera`. See [`Self::set_bounds`].
    pub fn set_bounds_of(
        &mut self,
        camera: Entity,
        xmin: Option<f32>,
        xmax: Option<f32>,
        ymin: Option<f32>,
        ymax: Option<f32>,
    ) -> Result<(), CameraLookupError> {
        self.camera_ref(camera)?;
        self.commands.entity(camera).insert(CameraBounds {
            xmin,
            xmax,
            ymin,
            ymax,
        });
        Ok(())
    }

    fn camera_ref(&self, camera: Entity) -> Result<CameraParts<'_>, CameraLookupError> {
        let (_, transform, zoom, slot) = self
            .cameras
            .get(camera)
            .map_err(|_| CameraLookupError::Unknown { entity: camera })?;
        Ok(CameraParts {
            transform,
            zoom,
            slot,
        })
    }

    fn camera_mut(&mut self, camera: Entity) -> Result<CameraPartsMut<'_>, CameraLookupError> {
        let (_, transform, _, _) = self
            .cameras
            .get_mut(camera)
            .map_err(|_| CameraLookupError::Unknown { entity: camera })?;
        Ok(CameraPartsMut { transform })
    }

    fn visible_size(&self, camera: Entity) -> Result<Vec2, CameraLookupError> {
        let camera_parts = self.camera_ref(camera)?;
        let slot = camera_parts
            .slot
            .ok_or(CameraLookupError::SlotNotReady { entity: camera })?;
        Ok(slot.visible_world_size(camera_parts.zoom.0))
    }

    fn visible_half(&self, camera: Entity) -> Result<Vec2, CameraLookupError> {
        Ok(self.visible_size(camera)? * 0.5)
    }
}
