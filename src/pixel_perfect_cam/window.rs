use std::num::NonZero;

use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution},
};

// Common integer game resolutions for pixel-perfect window setup:

#[allow(dead_code)]
pub const UFO_50: UVec2 = UVec2::new(384, 216);

#[allow(dead_code)]
pub const UFO_50_MOBILE: UVec2 = UVec2::new(160, 288);

#[allow(dead_code)]
pub const PIZZA_TOWER: UVec2 = UVec2::new(960, 540);

#[allow(dead_code)]
pub const NES: UVec2 = UVec2::new(256, 224);

/// Fluent helpers for configuring Bevy's primary `WindowPlugin`.
pub trait WindowPluginExt {
    fn with_resolution_of(self, size: UVec2, window_scale: u32) -> Self;
    fn at_position(self, pos: IVec2) -> Self;
    fn with_present_mode(self, mode: PresentMode) -> Self;
    fn with_desired_maximum_frame_latency(self, frames: u32) -> Self;
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

    /// Tip: get a size from the window size constants and then pass that in for `size`.
    /// For `window_scale` just use something like 1.
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
