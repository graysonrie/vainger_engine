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

pub trait Vec2Ext {
    /// Computes the vector from start to target, then clamp that vector to your maximum radius.
    ///
    /// Ex: How the player cursor works in Terraria
    fn clamp_towards(&self, target: Vec2, radius: f32) -> Vec2;
    fn add_to_x(self, val: f32) -> Vec2;
    fn add_to_y(self, val: f32) -> Vec2;
    fn neg_and_pos(val: f32) -> Vec2;
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

pub trait SpriteExt {
    fn flip_x_val(&self) -> f32;
}
impl SpriteExt for Sprite {
    fn flip_x_val(&self) -> f32 {
        if self.flip_x { -1. } else { 1. }
    }
}

pub trait ResultExt {
    /// Will print a tracing warning to the console if an error occurred
    fn warn_if_err(&self);
}

impl<T> ResultExt for anyhow::Result<T> {
    fn warn_if_err(&self) {
        if let Err(e) = self {
            warn!("{e}")
        }
    }
}
