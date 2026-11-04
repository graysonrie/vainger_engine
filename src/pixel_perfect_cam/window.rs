//! Integer game resolutions and helpers for Bevy's primary [`WindowPlugin`].
//!
//! Pass one of the constants as `game_res` to
//! [`crate::app::AppExt::setup_for_default_2d_pixel_game`], or as `size` to
//! [`WindowPluginExt::with_resolution_of`].

use std::num::NonZero;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};

/// 384×216. Antonblast's internal resolution.
#[allow(dead_code)]
pub const ANTONBLAST: UVec2 = UVec2::new(384, 216);

/// 384×216. UFO 50's internal resolution. Same size as [`ANTONBLAST`].
#[allow(dead_code)]
pub const UFO_50: UVec2 = UVec2::new(384, 216);

/// 160×288. UFO 50's portrait resolution.
#[allow(dead_code)]
pub const UFO_50_MOBILE: UVec2 = UVec2::new(160, 288);

/// 960×540.
#[allow(dead_code)]
pub const PIZZA_TOWER: UVec2 = UVec2::new(960, 540);

/// 256×224. NES resolution.
#[allow(dead_code)]
pub const NES: UVec2 = UVec2::new(256, 224);

/// Fluent helpers for configuring Bevy's primary `WindowPlugin`.
pub trait WindowPluginExt {
    /// OS window size, in physical pixels, is `size * window_scale`.
    ///
    /// `size` is the game resolution (see [`UFO_50`], [`NES`]). `window_scale`
    /// of `1` makes the window the same size as the canvas. Use an integer
    /// scale so each game pixel covers the same number of screen pixels.
    ///
    /// ### NOTE:
    /// This only sizes the OS window. The low-res canvas comes from
    /// [`super::PixelPerfectCameraPlugin::of_size`], and the two are not tied
    /// together after startup. A scale that does not match the canvas leaves
    /// letterboxing, or a window smaller than the canvas, which the camera
    /// plugin then forbids via the window's minimum size.
    fn with_resolution_of(self, size: UVec2, window_scale: u32) -> Self;
    /// Place the window's top-left at `pos`, in desktop pixels.
    ///
    /// [`crate::app::AppExt::setup_for_default_2d_pixel_game`] uses `(2, 2)`
    /// so the window does not sit under a menu bar.
    fn at_position(self, pos: IVec2) -> Self;
    /// Swap chain present mode. `Immediate` disables vsync.
    fn with_present_mode(self, mode: PresentMode) -> Self;
    /// Hint for how many frames the GPU may queue (`1` = lowest latency vsync).
    ///
    /// wgpu defaults to `2` when this is unset, which with `PresentMode::Fifo`
    /// is the usual "a few frames behind" feel. Drivers clamp the value to
    /// what they support. `frames` must be non-zero; `0` clears the hint
    /// (`NonZero::new(0)` is [`None`]).
    fn with_desired_maximum_frame_latency(self, frames: u32) -> Self;
    /// Whether the window takes keyboard focus when it opens.
    ///
    /// Dev builds of [`crate::app::AppExt::setup_for_default_2d_pixel_game`]
    /// pass `false` so a hot reload does not steal focus from the editor.
    fn with_focused(self, focused: bool) -> Self;
}

impl WindowPluginExt for WindowPlugin {
    fn with_present_mode(mut self, mode: PresentMode) -> Self {
        let window = self
            .primary_window
            .as_mut()
            .expect("Primary window should exist");

        window.present_mode = mode;

        self
    }

    /// Hint for how many frames the GPU may queue (`1` = lowest latency vsync).
    ///
    /// wgpu defaults to `2` when this is unset, which with `PresentMode::Fifo` is the usual
    /// “a few frames behind” feel. Drivers clamp the value to what they support.
    fn with_desired_maximum_frame_latency(mut self, frames: u32) -> Self {
        let window = self
            .primary_window
            .as_mut()
            .expect("Primary window should exist");

        window.desired_maximum_frame_latency = NonZero::new(frames);

        self
    }

    fn with_resolution_of(mut self, size: UVec2, window_scale: u32) -> Self {
        let window = self
            .primary_window
            .as_mut()
            .expect("Primary window should exist");
        window.resolution = WindowResolution::new(size.x * window_scale, size.y * window_scale);

        self
    }

    fn at_position(mut self, pos: IVec2) -> Self {
        self.primary_window
            .as_mut()
            .expect("Primary window should exist")
            .position = WindowPosition::new(pos);
        self
    }

    fn with_focused(mut self, focused: bool) -> Self {
        self.primary_window
            .as_mut()
            .expect("Primary window should exist")
            .focused = focused;
        self
    }
}
