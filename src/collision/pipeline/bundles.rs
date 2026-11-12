use super::*;

#[derive(Component, Default)]
pub struct PlatformerPlayer;

#[derive(Bundle, Default)]
pub struct PlatformerActorBundle {
    actor: CollisionActor,
    friction: Friction,
    gravity: Gravity,
    velocity: Velocity2D,
    contacts: CollisionContacts,
    speed: MoveSpeed,
}
