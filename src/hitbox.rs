use crate::prelude::*;

/// Represents a theoretical hitbox
#[derive(Clone, Debug, Component)]
pub struct Hitbox {
    size: UVec2,
    anchor: Anchor,
}

impl Hitbox {
    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub const fn new(width: u32, height: u32, anchor: Anchor) -> Self {
        Self {
            size: uvec2(width, height),
            anchor,
        }
    }

    /// Returns the pixel positions that represent the four corners of this bounding box
    /// within the world. `world_pos` is the pixel position that the sprite's `anchor` is
    /// currently at
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

    #[inline]
    pub fn size_vec2_half(&self) -> Vec2 {
        self.size.as_vec2() / 2.
    }
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

    pub fn width(&self) -> u32 {
        self.size.x
    }

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
