use bevy::state::state::FreelyMutableState;

use crate::{movement::MovementPlugin, prelude::*, ui::UiPlugin};

pub trait AppExt {
    fn setup_for_default_2d_pixel_game<M>(
        self,
        game_res: UVec2,
        in_game_condition: impl SystemCondition<M>,
    ) -> Self;

    /// Ensure you have already registered a `LoadingState` for your game
    fn load_engine_assets_in_state<S: FreelyMutableState>(&mut self, loading_state: S)
    -> &mut Self;
}

impl AppExt for App {
    /// Will add the PixelPerfectCameraPlugin and set up DefaultPlugins
    fn setup_for_default_2d_pixel_game<M>(
        mut self,
        game_res: UVec2,
        in_game_condition: impl SystemCondition<M>,
    ) -> Self {
        // Dev builds stay unfocused so cargo-watch reloads don't steal editor focus (macOS).
        let window_plugin = WindowPlugin::default()
            .at_position(IVec2::new(2, 2))
            .with_resolution_of(game_res, 2)
            .with_desired_maximum_frame_latency(1)
            .with_present_mode(bevy::window::PresentMode::Fifo)
            .with_focused(!cfg!(debug_assertions));

        self.configure_sets(
            Update,
            PixelPerfectCameraControlsSet.run_if(in_game_condition),
        )
        .add_plugins(
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
        .add_plugins(AsepriteUltraPlugin);

        self
    }
    fn load_engine_assets_in_state<S: FreelyMutableState>(
        &mut self,
        loading_state: S,
    ) -> &mut Self {
        super::assets::load_engine_assets(self, loading_state);
        self
    }
}
