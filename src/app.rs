use bevy_egui::EguiPlugin;
use bevy_framepace::Limiter;

use crate::{
    aseprite::{
        AsepriteExtPlugin, animation::AsepriteAnimationPlugin, hitbox::AsepriteHitboxPlugin,
    },
    collision::pipeline::Collision2DPlugin,
    ldtk::int_grid_cells::IntGridCellsPlugin,
    movement::MovementPlugin,
    prelude::*,
    ui::UiPlugin,
};

/// Installs the engine's default 2D pixel-game plugins on [`App`].
pub trait AppExt {
    /// Window, nearest-neighbor sampling, LDtk, Aseprite animation, the
    /// pixel-perfect camera, movement, UI, egui, and a 60 FPS cap.
    ///
    /// `game_res` is the low-resolution canvas size in pixels, for example
    /// [`UFO_50`] or [`NES`]. The OS window opens at that same size
    /// (`window_scale` of 1). Pass a larger scale to
    /// [`WindowPluginExt::with_resolution_of`] yourself if you build the
    /// [`WindowPlugin`] by hand and want each game pixel to cover more than
    /// one screen pixel.
    ///
    /// ### NOTE:
    /// This does not add [`crate::assets::AssetHelpersPlugin`] (loading `.aseprite`
    /// files as LDtk tileset images). Add those when you need them.
    ///
    /// ### NOTE:
    /// The present mode is [`bevy::window::PresentMode::Immediate`]. Vsync is
    /// off; `bevy_framepace` is what holds the frame rate at 60. Dev builds
    /// open the window unfocused so a reload does not steal editor focus.
    fn setup_for_default_2d_pixel_game(self, game_res: UVec2, window_scale: u32) -> Self;
}

impl AppExt for App {
    fn setup_for_default_2d_pixel_game(mut self, game_res: UVec2, window_scale: u32) -> Self {
        // Dev builds stay unfocused so cargo-watch reloads don't steal editor focus (macOS).
        let window_plugin = WindowPlugin::default()
            .at_position(IVec2::new(2, 2))
            .with_resolution_of(game_res, window_scale)
            .with_present_mode(bevy::window::PresentMode::Immediate)
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
        .add_plugins(PixelPerfectCameraPlugin::of_size(game_res))
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
