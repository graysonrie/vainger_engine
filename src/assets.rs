use bevy::state::state::FreelyMutableState;

use crate::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct EngineArtHandles {
    #[asset(path = "engine/art/text/ufo_font_default_spritesheet.png")]
    pub default_ufo_font: Handle<Image>,
}

pub fn load_engine_assets<S: FreelyMutableState>(app: &mut App, loading_state: S) {
    app.configure_loading_state(
        LoadingStateConfig::new(loading_state)
            .load_collection::<EngineArtHandles>()
            .finally_init_resource::<crate::ui::EngineFonts>(),
    );
}
