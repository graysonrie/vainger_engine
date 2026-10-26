use bevy_egui::EguiPlugin;

use crate::{movement::MovementPlugin, prelude::*, ui::UiPlugin};

pub trait AppExt {
    fn setup_for_default_2d_pixel_game(
        self,
        game_res: UVec2,
    ) -> Self;


}

impl AppExt for App {
    /// Will add the PixelPerfectCameraPlugin and set up DefaultPlugins
    fn setup_for_default_2d_pixel_game(
        mut self,
        game_res: UVec2,
    ) -> Self {
        // Dev builds stay unfocused so cargo-watch reloads don't steal editor focus (macOS).
        let window_plugin = WindowPlugin::default()
            .at_position(IVec2::new(2, 2))
            .with_resolution_of(game_res, 2)
            .with_desired_maximum_frame_latency(1)
            .with_present_mode(bevy::window::PresentMode::Immediate)
            .with_focused(!cfg!(debug_assertions));

        // ! This is now handled by the pixel perfect cam itself
        // self.configure_sets(
        //     Update,
        //     PixelPerfectCameraControlsSet.run_if(in_game_condition),
        // )
        self.add_plugins(
            DefaultPlugins
                .set(window_plugin)
                .set(ImagePlugin::default_nearest())
                // Gilrs listens to all HID devices on macOS; keyboard input (WASD) triggers
                // spurious "Failed to find device" warnings when no gamepad is connected.
                .disable::<GilrsPlugin>(),
        )
        .add_plugins(PixelPerfectCameraPlugin::of_size(game_res))
        .add_plugins(PrimitiveAssetsPlugin)
        .add_plugins(MovementPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(EguiPlugin::default())
        .add_plugins(AsepriteUltraPlugin);

        self
    }

}
