//! Axis-aligned hitbox in world pixels.
//!
//! The position you pass to the query methods is where [`Hitbox`]'s anchor
//! sits, not necessarily the sprite's visual center. [`Anchor::CENTER`] means
//! that position is the box center.

use crate::prelude::*;

/// Axis-aligned box. `anchor` is the point that sits on the world position you pass in.
///
/// ### NOTE:
/// This anchor is the hitbox's own. It is not kept in sync with a sprite
/// [`Anchor`] unless you set them yourself, or use
/// [`crate::aseprite::hitbox::AsepriteHitbox`], which writes both.
#[derive(Clone, Debug, Component, Default)]
pub struct Hitbox {
    size: UVec2,
    anchor: Anchor,
}

/// Bottom-left (`min`) and top-right (`max`) of a box, Y-up.
#[derive(Clone, Debug, Copy)]
pub struct MinMax {
    /// Inclusive lower corner (left, bottom).
    pub min: Vec2,
    /// Inclusive upper corner (right, top).
    pub max: Vec2,
}
impl MinMax {

    pub fn shrink_by_val(&self, val: Vec2) -> Self {
        MinMax {
            min: self.min + val,
            max: self.max - val,
        }
    }
}

impl Hitbox {
    /// Width and height in pixels.
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// `width` and `height` are in pixels. `anchor` uses Bevy's anchor space,
    /// where [`Anchor::CENTER`] is `(0, 0)` and the box extends half a size
    /// each way from there.
    pub const fn new(width: u32, height: u32, anchor: Anchor) -> Self {
        Self {
            size: uvec2(width, height),
            anchor,
        }
    }

    /// Returns the pixel positions that represent the four corners of this bounding box
    /// within the world. `world_pos` is the pixel position that the sprite's `anchor` is
    /// currently at
    ///
    /// The ordering is: BL, BR, TR, TL
    ///
    /// Visually, this looks like:
    /// ```ignore
    /// 4-----------3
    /// |           |
    /// |           |
    /// |           |
    /// 1-----------2
    /// ```
    pub fn four_corners_in_world(&self, world_pos: Vec2) -> [Vec2; 4] {
        let size = self.size_vec2();
        let anchor = self.anchor.as_vec();

        // Corners relative to the rectangle center:
        let corner_offsets = [
            Vec2::new(-0.5, -0.5), // bottom left
            Vec2::new(0.5, -0.5),  // bottom right
            Vec2::new(0.5, 0.5),   // top right
            Vec2::new(-0.5, 0.5),  // top left
        ];

        // So for each corner, find its offset (relative to anchor), multiply by size, and add to world_pos
        corner_offsets.map(|corner| {
            // Offset from anchor to corner
            let delta = (corner - anchor) * size;
            world_pos + delta
        })
    }

    /// Bottom-left and top-right of this box when its anchor is at `world_pos`.
    ///
    /// Same placement rules as [`Self::four_corners_in_world`].
    // TODO: see if this can be made any more efficient
    pub fn min_max_positions_in_world(&self, world_pos: Vec2) -> MinMax {
        let corners = self.four_corners_in_world(world_pos);
        let min = corners.into_iter().reduce(Vec2::min).unwrap();
        let max = corners.into_iter().reduce(Vec2::max).unwrap();
        MinMax { min, max }
    }

    /// Half the width and height, in pixels.
    #[inline]
    pub fn size_vec2_half(&self) -> Vec2 {
        self.size.as_vec2() / 2.
    }
    /// Width and height as floats, in pixels.
    #[inline]
    pub fn size_vec2(&self) -> Vec2 {
        self.size.as_vec2()
    }
    /// Returns the pixel position that will horizontally center the hitbox on the `point`,
    /// and ensure that the bottom of the hitbox is sitting directly on top of the `point`
    pub fn above_point_centered(&self, point: Vec2) -> Vec2 {
        let size = self.size_vec2();
        let anchor = self.anchor.as_vec();
        // World = transform + (local - anchor) * size, local in [-0.5, 0.5].
        // Put the bottom-center local point on `point`.
        let local_bottom_center = Vec2::new(0.0, -0.5);
        point - (local_bottom_center - anchor) * size
    }

    /// Offset from the anchor to the bottom-center of the box, in pixels.
    ///
    /// Add this to the anchor's world position to get that point in the world.
    /// It is not itself a world position.
    pub fn bottom_center(&self) -> Vec2 {
        // The local-space bottom-center is at (0.0, -0.5).
        // To get this in the hitbox's local space, offset it by (-self.anchor) and scale by self.size.
        let size = self.size_vec2();
        let anchor = self.anchor.as_vec();
        // Local space point for bottom center
        let local_bottom_center = Vec2::new(0.0, -0.5);
        // Offset from anchor to desired point, scaled by size
        (local_bottom_center - anchor) * size
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.size.x
    }

    /// `width / 2`, rounded down.
    pub fn width_half(&self) -> u32 {
        self.size.x / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_anchor_sits_above_point() {
        let hitbox = Hitbox::new(24, 24, Anchor::CENTER);
        let pos = hitbox.above_point_centered(Vec2::new(80.0, -192.0));
        assert_eq!(pos, Vec2::new(80.0, -180.0));
        let corners = hitbox.four_corners_in_world(pos);
        let ys: Vec<f32> = corners.iter().map(|c| c.y).collect();
        assert!(ys.iter().any(|y| (*y - -192.0).abs() < f32::EPSILON));
        assert!(ys.iter().any(|y| (*y - -168.0).abs() < f32::EPSILON));
    }
}
