use crate::{
    collision::{
        CollisionContacts, get_collision_contacts, move_and_collide,
        pipeline::{
            collision_check::LdtkCollisionCheck,
            jump::{CollisionJumpPlugin, Gravity},
        },
    },
    components::Velocity2D,
    prelude::*,
    sets::InputsSet,
};

pub mod bundles;
pub mod collision_check;
pub mod jump;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Collision2DSet {
    Jump,
    MoveAndCollide,
}

/// Uses LDtk for collision checking
pub struct Collision2DPlugin {
    gravity: f32,
}
impl Default for Collision2DPlugin {
    fn default() -> Self {
        Self { gravity: -0.45 }
    }
}

impl Plugin for Collision2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CollisionJumpPlugin)
            .insert_resource(Collision2DGravityForce(self.gravity))
            .add_systems(
                Update,
                (
                    apply_platformer_velocity_from_intent,
                    apply_friction,
                    apply_speed_clamp,
                    handle_move_and_collide,
                    update_contacts,
                )
                    .in_set(Collision2DSet::MoveAndCollide),
            )
            .configure_sets(
                Update,
                (Collision2DSet::Jump, Collision2DSet::MoveAndCollide)
                    .after(InputsSet)
                    .chain(),
            );
    }
}

/// The force of gravity in the world
#[derive(Resource)]
struct Collision2DGravityForce(pub f32);
impl Default for Collision2DGravityForce {
    fn default() -> Self {
        Self(-0.45)
    }
}

/// Marker component that indicates this entity wants to participate in the
/// 2D AABB collision pipeline
#[derive(Component, Default)]
pub struct CollisionActor;

/// The character will be affected by friction
#[derive(Component)]
pub struct Friction {
    pub amount: f32,
}
impl Default for Friction {
    fn default() -> Self {
        Self { amount: 0.9 }
    }
}

pub trait CollisionCheck {
    fn is_colliding(&self, pos: Vec2, hitbox: &Hitbox) -> bool;

    fn is_world_loaded(&self) -> bool {
        true
    }
}

fn handle_move_and_collide(
    mut q: Query<(&mut Transform, &Hitbox, &mut Velocity2D), With<CollisionActor>>,
    collision: LdtkCollisionCheck,
) {
    for (mut transform, hitbox, mut velocity_2d) in &mut q {
        move_and_collide(&mut transform, hitbox, &mut velocity_2d, |pos, hitbox| {
            collision.is_colliding(pos, hitbox)
        });
    }
}

fn apply_friction(mut q: Query<(&mut Velocity2D, &Friction), With<CollisionActor>>) {
    for (mut velocity, friction) in &mut q {
        velocity.x *= friction.amount;
    }
}

#[derive(Component)]
pub struct SpeedClamp {
    pub min: Vec2,
    pub max: Vec2,
}
impl Default for SpeedClamp {
    fn default() -> Self {
        Self {
            min: vec2(-8., -20.),
            max: vec2(8., 20.),
        }
    }
}
fn apply_speed_clamp(mut q: Query<(&mut Velocity2D, &SpeedClamp), With<CollisionActor>>) {
    for (mut velocity, speed_clamp) in &mut q {
        velocity.0 = velocity.clamp(speed_clamp.min, speed_clamp.max);
    }
}

fn update_contacts(
    mut q: Query<(&mut Transform, &Hitbox, &mut CollisionContacts)>,
    collision: LdtkCollisionCheck,
) {
    for (transform, hitbox, mut contacts) in &mut q {
        *contacts = get_collision_contacts(transform.get_2d_pos(), hitbox, |pos, hitbox| {
            collision.is_colliding(pos, hitbox)
        });
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f32);
impl Default for MoveSpeed {
    fn default() -> Self {
        Self(1.)
    }
}

/// Should represent the direction the character was trying to move in their inputs
#[derive(Component, Default, Deref, DerefMut)]
pub struct MoveIntent(pub IVec2);

fn apply_platformer_velocity_from_intent(
    mut q: Query<(&mut Velocity2D, &MoveIntent, &MoveSpeed), (With<CollisionActor>, With<Gravity>)>,
) {
    for (mut velocity_2d, move_intent, move_speed) in &mut q {
        velocity_2d.x += move_intent.x as f32 * move_speed.0;
    }
}
