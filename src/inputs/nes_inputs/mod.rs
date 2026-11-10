use crate::{
    collision::pipeline::{MoveIntent, jump::JumpBuffer},
    inputs::{ActionStateExt, IgnoreInputs},
    prelude::*,
    sets,
};

/// A convenient stick deadzone value you can use, for example in conjunction
/// with `clamped_axis_pair_binary`
pub const STICK_DEADZONE: f32 = 0.3;

pub struct NESInputsPlugin;

impl Plugin for NESInputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<NESAction>::default())
            .add_systems(
                Update,
                (write_to_move_intent, write_to_jump_buffer).in_set(sets::InputsSet),
            );
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Actionlike)]
pub enum NESAction {
    #[actionlike(DualAxis)]
    Move,
    /// The button to the right on a real NES controller
    #[actionlike(Button)]
    A,
    /// The button to the left on a real NES controller
    #[actionlike(Button)]
    B,
    #[actionlike(Button)]
    Start,
}

impl NESAction {
    pub fn default_input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        input_map.insert_dual_axis(Self::Move, GamepadStick::LEFT);
        input_map.insert_dual_axis(Self::Move, VirtualDPad::arrow_keys());

        input_map.insert(Self::A, GamepadButton::South);
        input_map.insert(Self::A, KeyCode::KeyZ);

        input_map.insert(Self::B, GamepadButton::West);
        input_map.insert(Self::B, KeyCode::KeyX);

        input_map.insert(Self::Start, GamepadButton::Start);
        input_map.insert(Self::Start, KeyCode::Enter);

        input_map
    }
}

fn write_to_move_intent(
    mut q: Query<(&mut MoveIntent, &ActionState<NESAction>), Without<IgnoreInputs>>,
) {
    for (mut move_intent, action_state) in &mut q {
        let move_delta = action_state.clamped_axis_pair_binary(&NESAction::Move, STICK_DEADZONE);
        move_intent.0 = move_delta;
    }
}

fn write_to_jump_buffer(
    mut q: Query<(&mut JumpBuffer, &ActionState<NESAction>), Without<IgnoreInputs>>,
) {
    for (mut jump_buffer, action_state) in &mut q {
        if action_state.just_pressed(&NESAction::A) {
            jump_buffer.set_pressed();
        }

        if action_state.just_released(&NESAction::A) {
            jump_buffer.set_released();
        }
    }
}
