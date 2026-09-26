use bevy::ecs::system::{ReadOnlySystemParam, SystemState};

use crate::prelude::*;

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

pub trait Vec2Ext {
    /// Computes the vector from start to target, then clamp that vector to your maximum radius.
    ///
    /// Ex: How the player cursor works in Terraria
    fn clamp_towards(&self, target: Vec2, radius: f32) -> Vec2;
}
impl Vec2Ext for Vec2 {
    fn clamp_towards(&self, target: Vec2, radius: f32) -> Vec2 {
        let start = self;
        start + (target - start).clamp_length_max(radius)
    }
}

pub trait F32Ext {
    /// Creates a `Vec2` with this number as the `x` component and leaves the `y` component at 0
    fn as_vec2_x(&self) -> Vec2;
}
impl F32Ext for f32 {
    fn as_vec2_x(&self) -> Vec2 {
        vec2(*self, 0.)
    }
}

pub trait I32Ext {
    /// Creates an `IVec2` with this number as the `y` component and leaves the `x` component at 0
    fn as_ivec2_y(&self) -> IVec2;
}

impl I32Ext for i32 {
    fn as_ivec2_y(&self) -> IVec2 {
        ivec2(0, *self)
    }
}
