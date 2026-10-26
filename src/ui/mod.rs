use bevy::prelude::*;

pub mod fonts;

pub use fonts::{BitmapText, BitmapTextPlugin};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BitmapTextPlugin);
    }
}
