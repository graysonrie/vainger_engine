use crate::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub trait ActionStateExt<A>
where
    A: leafwing_input_manager::Actionlike,
{
    /// The X and Y components in the output will either be -1, 0, or 1, with nothing in between
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
