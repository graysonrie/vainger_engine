//! Remembers the camera's follow position. That position is what renders.
//!
//! Rounding the camera onto a texel grid, and rounding sprites separately,
//! slides a stopped character a pixel into the level while the camera eases.

use crate::prelude::*;

/// Follow position, and the position last presented for rendering.
#[derive(Component, Clone, Copy)]
pub(crate) struct SubpixelPos {
    logical: Vec2,
    on_grid: Option<Vec2>,
}

impl Default for SubpixelPos {
    fn default() -> Self {
        Self {
            logical: Vec2::ZERO,
            on_grid: None,
        }
    }
}

impl SubpixelPos {
    /// `pos` was set explicitly, so the next restore must not put the old fraction back.
    pub(crate) fn set_exact(&mut self, pos: Vec2) {
        self.logical = pos;
        self.on_grid = Some(pos);
    }
}

/// Puts the fractional center back before follow and bounds run.
///
/// A transform that no longer matches the grid position was moved by something
/// else (`set_xy`, a panner) and becomes the new center.
pub(super) fn restore_subpixel_pos(
    mut cameras: Query<(&mut Transform, &mut SubpixelPos), With<super::PixelPerfectCamera>>,
) {
    for (mut transform, mut sample) in &mut cameras {
        let shown = transform.translation.truncate();
        let logical = match sample.on_grid {
            Some(on_grid) if on_grid == shown => sample.logical,
            _ => {
                sample.logical = shown;
                shown
            }
        };
        transform.translation.x = logical.x;
        transform.translation.y = logical.y;
    }
}

/// Records the follow position as the position that will be drawn.
///
/// The low-resolution target and nearest filtering define the pixel size.
/// The camera transform is left where follow and bounds put it, so the level
/// and the characters share one view.
pub(super) fn present_camera_at_follow_position(
    mut cameras: Query<(&Transform, &mut SubpixelPos), With<super::PixelPerfectCamera>>,
) {
    for (transform, mut sample) in &mut cameras {
        let logical = transform.translation.truncate();
        sample.logical = logical;
        sample.on_grid = Some(logical);
    }
}
