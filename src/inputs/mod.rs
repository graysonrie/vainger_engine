//! Digital reads from leafwing [`ActionState`]s.

use crate::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// Turns analog leafwing axes into digital directions.
pub trait ActionStateExt<A>
where
    A: leafwing_input_manager::Actionlike,
{
    /// Dual-axis value collapsed to `-1`, `0`, or `1` on each axis.
    ///
    /// `stick_deadzone` is the absolute value below which an axis reads as `0`.
    /// A virtual d-pad already returns `-1`, `0`, or `1`, so the deadzone
    /// mainly matters for analog sticks. There is no extra outer clamp: a
    /// stick past the deadzone contributes only its sign.
    fn clamped_axis_pair_binary(&self, action: &A, stick_deadzone: f32) -> IVec2;
}

impl<A> ActionStateExt<A> for ActionState<A>
where
    A: leafwing_input_manager::Actionlike,
{
    fn clamped_axis_pair_binary(&self, action: &A, stick_deadzone: f32) -> IVec2 {
        let val = self.clamped_axis_pair(action);
        let x = if val.x.abs() < stick_deadzone {
            0
        } else {
            val.x.signum() as i32
        };
        let y = if val.y.abs() < stick_deadzone {
            0
        } else {
            val.y.signum() as i32
        };
        ivec2(x, y)
    }
}
