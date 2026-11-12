use crate::prelude::*;
pub mod glide;

#[derive(Clone, Debug, Copy, Default)]
pub enum HorizontalDirection {
    Left,
    #[default]
    Right,
}
pub fn horizontal_direction_as_value(direction: &HorizontalDirection) -> i32 {
    match direction {
        HorizontalDirection::Left => -1,
        HorizontalDirection::Right => 1,
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct MovementSet;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(glide::MovementGlidePlugin);
    }
}
