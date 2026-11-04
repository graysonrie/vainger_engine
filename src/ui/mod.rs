//! Bitmap-font UI drawn into the low-res game canvas.

use bevy::prelude::*;

pub mod fonts;

pub use fonts::{BitmapText, BitmapTextPlugin};

/// Installs [`BitmapTextPlugin`].
///
/// Already added by [`crate::app::AppExt::setup_for_default_2d_pixel_game`].
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(BitmapTextPlugin);
    }
}
