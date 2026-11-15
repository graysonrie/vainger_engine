use crate::{prelude::*, preludes::collision_pipeline::*};

/// Updates [`LocomotionDescriptionState`] components
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LocomotionDescriptionSet;

pub struct LocomotionDescriptionPlugin;

impl Plugin for LocomotionDescriptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, describe_state.in_set(LocomotionDescriptionSet))
            .configure_sets(Update, LocomotionDescriptionSet.after(FullCollision2DSet));
    }
}

fn describe_state(
    mut q: Query<
        (
            &mut LocomotionDescriptionState,
            &Velocity2D,
            &CollisionContacts,
            &Sprite,
            &MoveIntent,
        ),
        With<CollisionActor>,
    >,
) {
    use LocomotionDescription::*;
    for (mut anim_state, velocity, contacts, sprite, move_wish) in &mut q {
        if !contacts.floor {
            if velocity.y > 0. {
                anim_state.set_next(Rising);
                return;
            } else {
                anim_state.set_next(Falling);
                return;
            }
        }
        if velocity.x.signum() != sprite.flip_x_val() {
            anim_state.set_next(Turn);
        }
        if velocity.x.abs() > 0.3 || move_wish.x != 0 {
            if move_wish.x == 0 {
                // The character isn't trying to move but still has leftover speed
                anim_state.set_next(Decelerating);
            }
            anim_state.set_next(Run);
            return;
        }
        anim_state.set_next(Idle);
    }
}

#[derive(Component, Default)]
#[require(CollisionContacts, Velocity2D, Gravity)]
pub struct LocomotionDescriptionState {
    pub current: LocomotionDescription,
}
impl LocomotionDescriptionState {
    pub fn set_next(&mut self, next: LocomotionDescription) {
        self.current = next;
    }
}

#[derive(Clone, Debug, Default)]
pub enum LocomotionDescription {
    #[default]
    Idle,
    Run,
    Decelerating,
    Turn,
    Rising,
    Falling,
}
