use crate::prelude::*;

/// Provides details about the game dimensions and camera bounds, as well
/// as ways to set them and read from them
#[derive(Resource, Clone, Default)]
pub struct PixelPerfectCameraConfig {
    pub game_width: f32,
    pub game_height: f32,
}

impl PixelPerfectCameraConfig {
    #[inline]
    pub fn height_half(&self) -> f32 {
        self.game_height / 2.
    }

    #[inline]
    pub fn width_half(&self) -> f32 {
        self.game_width / 2.
    }

    #[inline]
    pub fn viewport_height(&self) -> f32 {
        self.game_height
    }

    #[inline]
    pub fn viewport_dimensions_half(&self) -> Vec2 {
        Vec2::new(self.viewport_width_half(), self.viewport_height_half())
    }

    #[inline]
    pub fn viewport_width(&self) -> f32 {
        self.game_width
    }

    #[inline]
    pub fn viewport_width_half(&self) -> f32 {
        self.width_half()
    }

    #[inline]
    pub fn viewport_height_half(&self) -> f32 {
        self.height_half()
    }

    /// Top-left corner of the viewport in world space (Bevy Y-up).
    #[inline]
    pub fn viewport_top_left(&self) -> Vec2 {
        Vec2::new(-self.viewport_width_half(), self.viewport_height_half())
    }
}
