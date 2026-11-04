use crate::prelude::*;

/// Displacement in world pixels, applied as-is by [`crate::collision::move_and_collide`].
///
/// ### NOTE:
/// This is not pixels per second. The collision pass does not multiply it by
/// frame time, so a speed stored in pixels per second runs faster at higher
/// frame rates. Scale by `time.delta_secs()` before the call if that is what
/// you store. The default app setup caps the frame rate at 60, which hides
/// the difference until that cap is removed.
#[derive(Component, Default, Copy, Clone, Deref, DerefMut)]
pub struct Velocity2D(pub Vec2);
