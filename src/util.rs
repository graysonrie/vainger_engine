//! Small extensions used throughout the engine. Methods are on Bevy types
//! (`Transform`, `Vec2`, `Sprite`, and so on).

use std::fmt::Display;

use crate::prelude::*;

/// Extra constructors and accessors for transforms.
pub trait TransformExt {
    /// Creates a `Transform` with x and y values of 0.0. Can be useful for setting a custom z value for z-ordering.
    ///
    /// Recall that the greater the value, the higher up it will be
    fn from_z(z: f32) -> Transform {
        Transform::from_xyz(0., 0., z)
    }

    /// Get the 2D position of the `Transform` in the world.
    ///
    /// Convenience for
    /// ```ignore
    /// self.translation.truncate()
    /// ```
    fn get_2d_pos(&self) -> Vec2;
}

impl TransformExt for Transform {
    fn get_2d_pos(&self) -> Vec2 {
        self.translation.truncate()
    }
}
impl TransformExt for GlobalTransform {
    fn get_2d_pos(&self) -> Vec2 {
        self.translation().truncate()
    }
}

/// In-place 2D edits that keep `z`.
pub trait Vec3Ext {
    /// Adds the 2D position to this, preserving the z value
    fn add_2d_pos(&mut self, pos: &Vec2);
}
impl Vec3Ext for Vec3 {
    fn add_2d_pos(&mut self, pos: &Vec2) {
        self.x += pos.x;
        self.y += pos.y;
    }
}

/// Copy and in-place edits for 2D vectors.
pub trait Vec2Ext {
    /// Point on the segment from `self` toward `target`, at most `radius` away from `self`.
    ///
    /// This is the "cursor can only get this far from the player" clamp.
    /// A `radius` of `0` returns `self`. Negative radii are not meaningful.
    fn clamp_towards(&self, target: Vec2, radius: f32) -> Vec2;
    /// Copy with `val` added to `x`.
    fn add_to_x(self, val: f32) -> Vec2;
    /// Copy with `val` added to `y`.
    fn add_to_y(self, val: f32) -> Vec2;
    /// `(-val, val)`. Call as `Vec2::neg_and_pos(val)`.
    fn neg_and_pos(val: f32) -> Vec2;
    /// Sets both components to `0` and returns that value.
    ///
    /// This mutates `self`. It does not return a fresh [`Vec2::ZERO`] while
    /// leaving the original alone.
    fn zero(&mut self) -> Self;
}
impl Vec2Ext for Vec2 {
    fn clamp_towards(&self, target: Vec2, radius: f32) -> Vec2 {
        let start = self;
        start + (target - start).clamp_length_max(radius)
    }
    fn add_to_x(mut self, val: f32) -> Vec2 {
        self.x += val;
        self
    }
    fn add_to_y(mut self, val: f32) -> Vec2 {
        self.y += val;
        self
    }
    fn neg_and_pos(val: f32) -> Vec2 {
        vec2(-val, val)
    }
    fn zero(&mut self) -> Self {
        self.x = 0.;
        self.y = 0.;
        *self
    }
}

/// Build a vector from a single float.
pub trait F32Ext {
    /// Creates a `Vec2` with this number as the `x` component and leaves the `y` component at 0
    fn as_vec2_x(&self) -> Vec2;
}
impl F32Ext for f32 {
    fn as_vec2_x(&self) -> Vec2 {
        vec2(*self, 0.)
    }
}

/// Build a vector from a single integer.
pub trait I32Ext {
    /// Creates an `IVec2` with this number as the `y` component and leaves the `x` component at 0
    fn as_ivec2_y(&self) -> IVec2;
}

impl I32Ext for i32 {
    fn as_ivec2_y(&self) -> IVec2 {
        ivec2(0, *self)
    }
}

/// Sign helpers for sprite flipping.
pub trait SpriteExt {
    /// `-1` when [`Sprite::flip_x`] is set, otherwise `1`.
    ///
    /// Multiply an X offset or an X scale by this to mirror it with the sprite.
    /// [`Sprite::flip_y`] is ignored.
    fn flip_x_val(&self) -> f32;
}
impl SpriteExt for Sprite {
    fn flip_x_val(&self) -> f32 {
        if self.flip_x { -1. } else { 1. }
    }
}

/// Logging helpers for [`Result`].
pub trait ResultExt {
    /// Will print a tracing warning to the console if an error occurred
    fn warn_if_err(&self);
}

impl<T, E> ResultExt for Result<T, E>
where
    E: Display,
{
    fn warn_if_err(&self) {
        if let Err(e) = self {
            warn!("{e}")
        }
    }
}
