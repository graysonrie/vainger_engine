use crate::prelude::*;

/// Low-resolution canvas size, in pixels.
///
/// [`super::PixelPerfectCameraPlugin`] inserts this from its config at startup.
/// The render target and both camera projections are created from these fields
/// once, in the startup system.
///
/// ### NOTE:
/// Changing `game_width` or `game_height` after startup does not resize the
/// canvas or the projections. Restart the app to change the game resolution.
#[derive(Resource, Clone, Default)]
pub struct PixelPerfectCameraConfig {
    /// Canvas width in pixels.
    pub game_width: f32,
    /// Canvas height in pixels.
    pub game_height: f32,
}

impl PixelPerfectCameraConfig {
    /// `game_height / 2`.
    #[inline]
    pub fn height_half(&self) -> f32 {
        self.game_height / 2.
    }

    /// `game_width / 2`.
    #[inline]
    pub fn width_half(&self) -> f32 {
        self.game_width / 2.
    }

    /// Canvas height in pixels. Same value as [`Self::game_height`].
    #[inline]
    pub fn viewport_height(&self) -> f32 {
        self.game_height
    }

    /// Half the canvas size, `(width / 2, height / 2)`, in pixels.
    #[inline]
    pub fn viewport_dimensions_half(&self) -> Vec2 {
        Vec2::new(self.viewport_width_half(), self.viewport_height_half())
    }

    #[inline]
    pub fn viewport_dimensions(&self) -> Vec2 {
        vec2(self.game_width, self.game_height)
    }

    /// Canvas width in pixels. Same value as [`Self::game_width`].
    #[inline]
    pub fn viewport_width(&self) -> f32 {
        self.game_width
    }

    /// Half the canvas width. Same as [`Self::width_half`].
    #[inline]
    pub fn viewport_width_half(&self) -> f32 {
        self.width_half()
    }

    /// Half the canvas height. Same as [`Self::height_half`].
    #[inline]
    pub fn viewport_height_half(&self) -> f32 {
        self.height_half()
    }

    /// Offset from the camera center to the top-left of the view, Y-up.
    ///
    /// This is `(-width / 2, height / 2)`, not a world position. It assumes the
    /// camera is looking at its own origin. Add the camera translation to get
    /// the corner in the world; [`super::PixelPerfectCameraQuery::viewport_top_left`]
    /// does that.
    #[inline]
    pub fn viewport_top_left(&self) -> Vec2 {
        Vec2::new(-self.viewport_width_half(), self.viewport_height_half())
    }
}
