//! Facing and the glide tween. [`MovementPlugin`] is installed by
//! [`crate::app::AppExt::setup_for_default_2d_pixel_game`].

use crate::prelude::*;
pub mod glide;

/// Left or right. [`horizontal_direction_as_value`] turns it into a sign.
#[derive(Clone, Debug, Copy, Default)]
pub enum HorizontalDirection {
    /// Negative X.
    Left,
    /// Positive X. The default.
    #[default]
    Right,
}

/// `-1` for [`HorizontalDirection::Left`], `1` for [`HorizontalDirection::Right`].
pub fn horizontal_direction_as_value(direction: &HorizontalDirection) -> i32 {
    match direction {
        HorizontalDirection::Left => -1,
        HorizontalDirection::Right => 1,
    }
}

/// System set for movement systems. Glide runs here during [`Update`].
///
/// The set is not ordered against other sets. Camera follow runs later, in
/// [`PostUpdate`], so a glide (or anything else in [`Update`]) is visible to
/// the camera on the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

/// Installs [`glide::MovementGlidePlugin`].
///
/// Already added by [`crate::app::AppExt::setup_for_default_2d_pixel_game`].
/// Adding it again registers the glide system twice.
pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(glide::MovementGlidePlugin);
    }
}
