use bevy::window::WindowMode;
use bevy_egui::EguiPlugin;
use bevy_framepace::Limiter;

use crate::{
    aseprite::{
        AsepriteExtPlugin, animation::AsepriteAnimationPlugin, hitbox::AsepriteHitboxPlugin,
    },
    ldtk::int_grid_cells::IntGridCellsPlugin,
    movement::MovementPlugin,
    prelude::*,
    ui::UiPlugin,
};

pub struct GameResSettings {
    game_res: UVec2,
    window_scale: u32,
    window_mode: WindowMode,
}
impl GameResSettings {
    /// New game res with `Windowed` mode
    pub fn new(game_res: UVec2, window_scale: u32) -> Self {
        Self {
            game_res,
            window_scale,
            window_mode: WindowMode::Windowed,
        }
    }

    /// New game res with borderless fullscreen mode
    pub fn new_fullscreen(game_res: UVec2, window_scale: u32) -> Self {
        Self {
            game_res,
            window_scale,
            window_mode: WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        }
    }
}

pub trait AppExt {
    fn setup_for_default_2d_pixel_game(self, settings: GameResSettings) -> Self;
}

impl AppExt for App {
    /// Will add the PixelPerfectCameraPlugin and set up DefaultPlugins
    ///
    /// Sets up `bevy_framepace` to cap the FPS at 60
    fn setup_for_default_2d_pixel_game(mut self, settings: GameResSettings) -> Self {
        // Dev builds stay unfocused so cargo-watch reloads don't steal editor focus (macOS).
        let window_plugin = WindowPlugin::default()
            .at_position(IVec2::new(2, 2))
            .with_resolution_of(settings.game_res, settings.window_scale)
            .with_present_mode(bevy::window::PresentMode::Immediate)
            .with_window_mode(settings.window_mode )
            .with_focused(!cfg!(debug_assertions));

        self.add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(ImagePlugin::default_nearest())
                // Gilrs listens to all HID devices on macOS; keyboard input (WASD) triggers
                // spurious "Failed to find device" warnings when no gamepad is connected.
                .disable::<GilrsPlugin>(),
        )
        .add_plugins(LdtkPlugin)
        .add_plugins(AsepriteAnimationPlugin)
        .add_plugins(AsepriteExtPlugin)
        .add_plugins(PixelPerfectCameraPlugin::of_size(settings.game_res).with_res(1))
        .add_plugins(PrimitiveAssetsPlugin)
        .add_plugins(AsepriteHitboxPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(bevy_framepace::FramepacePlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(AsepriteUltraPlugin)
        .add_plugins(IntGridCellsPlugin)
        .add_systems(Startup, set_fps);

        self
    }
}

fn set_fps(mut settings: ResMut<bevy_framepace::FramepaceSettings>) {
    settings.limiter = Limiter::from_framerate(60.)
}
