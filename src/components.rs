use crate::prelude::*;

#[derive(Component, Default, Copy, Clone, Deref, DerefMut)]
pub struct Velocity2D(pub Vec2);
