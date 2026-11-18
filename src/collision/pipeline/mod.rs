use crate::{
    collision::{
        CollisionContacts, get_collision_contacts, move_and_collide,
        pipeline::{
            collision_check::SolidWorld,
            jump::{CollisionJumpPlugin, Gravity},
            locomotion::LocomotionDescriptionPlugin,
        },
    },
    components::Velocity2D,
    prelude::*,
    sets::InputsSet,
};

pub mod bundles;
pub mod collision_check;
pub mod collision_handlers;
pub mod jump;
pub mod locomotion;

pub use collision_handlers::ldtk::LdtkSolidWorldPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullCollision2DSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SolidWorldSyncSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Collision2DSet {
    Jump,
    Move,
    Collide,
}

#[derive(Default, Clone, PartialEq)]
pub enum SolidWorldType {
    Ldtk,
    #[default]
    /// You must manually sync to the solid world
    Custom,
}

#[derive(Default)]
/// 2D AABB platformer collision pipeline.
///
/// Inserts an empty [`SolidWorld`]. Add a map backend plugin (e.g.
/// [`LdtkSolidWorldPlugin`]) so that world gets filled from your level data.
pub struct Collision2DPlugin {
    gravity: f32,
    solid_world_type: SolidWorldType,
}

impl Collision2DPlugin {
    /// Should be a negative number. `-0.45` seems to work well
    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    pub fn with_solid_world_type(mut self, solid_world_type: SolidWorldType) -> Self {
        self.solid_world_type = solid_world_type;
        self
    }
}

impl Plugin for Collision2DPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CollisionJumpPlugin)
            .add_plugins(LocomotionDescriptionPlugin)
            .init_resource::<SolidWorld>()
            .insert_resource(Collision2DGravityForce(self.gravity))
            .add_systems(
                Update,
                (
                    apply_platformer_velocity_from_intent,
                    apply_friction,
                    apply_speed_clamp,
                    snap_to_ground,
                )
                    .in_set(Collision2DSet::Move),
            )
            .add_systems(
                Update,
                (handle_move_and_collide, update_contacts).in_set(Collision2DSet::Collide),
            )
            .configure_sets(
                Update,
                (
                    Collision2DSet::Jump,
                    Collision2DSet::Move,
                    Collision2DSet::Collide,
                )
                    .in_set(FullCollision2DSet)
                    .after(InputsSet)
                    .chain(),
            )
            .configure_sets(Update, FullCollision2DSet.after(SolidWorldSyncSet));
        if self.solid_world_type == SolidWorldType::Ldtk {
            app.add_plugins(LdtkSolidWorldPlugin);
        }
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

/// Defaults to `1.0`
#[derive(Component, Deref, DerefMut)]
pub struct MoveSpeed(pub f32);
impl Default for MoveSpeed {
    fn default() -> Self {
        Self(1.)
    }
}

/// If present on an entity, while you can still write to their [`MoveIntent`], the system that
/// handles it will not do anything
#[derive(Component)]
pub struct IgnoreMoveIntent;

/// Should represent the direction the character was trying to move in their inputs
#[derive(Component, Default, Deref, DerefMut)]
pub struct MoveIntent(pub IVec2);

fn apply_platformer_velocity_from_intent(
    mut q: Query<
        (&mut Velocity2D, &MoveIntent, &MoveSpeed),
        (
            With<CollisionActor>,
            With<Gravity>,
            Without<IgnoreMoveIntent>,
        ),
    >,
) {
    for (mut velocity_2d, move_intent, move_speed) in &mut q {
        velocity_2d.x += move_intent.x as f32 * move_speed.0;
    }
}

#[derive(Component, Default)]
pub struct DisablePlatformerMovement;

fn handle_move_and_collide(
    mut q: Query<
        (&mut Transform, &Hitbox, &mut Velocity2D),
        (With<CollisionActor>, Without<DisablePlatformerMovement>),
    >,
    solid: Res<SolidWorld>,
) {
    for (mut transform, hitbox, mut velocity_2d) in &mut q {
        move_and_collide(&mut transform, hitbox, &mut velocity_2d, |pos, hitbox| {
            solid.is_colliding(pos, hitbox)
        });
    }
}

fn update_contacts(
    mut q: Query<(&Transform, &Hitbox, &mut CollisionContacts)>,
    solid: Res<SolidWorld>,
) {
    for (transform, hitbox, mut contacts) in &mut q {
        *contacts = get_collision_contacts(transform.get_2d_pos(), hitbox, |pos, hitbox| {
            solid.is_colliding(pos, hitbox)
        });
    }
}

/// The entity will try to snap to the ground as soon as they can
#[derive(Component)]
#[require(Hitbox, Transform)]
pub struct SnapToGround {
    pub max_attempts: u32,
}
impl Default for SnapToGround {
    fn default() -> Self {
        Self { max_attempts: 512 }
    }
}

fn snap_to_ground(
    mut q: Query<(Entity, &mut Transform, &Hitbox, &SnapToGround), With<CollisionActor>>,
    solid: Res<SolidWorld>,
    mut commands: Commands,
) {
    if !solid.is_loaded() {
        return;
    }
    for (entity, mut transform, hitbox, snap) in &mut q {
        let mut attempts = 0;
        let max_attempts = snap.max_attempts;
        while solid.is_colliding(transform.get_2d_pos(), hitbox) {
            transform.translation.y += 1.;
            if attempts > max_attempts {
                warn!(
                    "Could not snap to ground: Could not ascend above ground in time. Will try again later"
                );
            }
            attempts += 1;
        }
        attempts = 0;
        while !solid.is_colliding(transform.get_2d_pos(), hitbox) {
            transform.translation.y -= 1.;
            if attempts > max_attempts {
                warn!(
                    "Could not snap to ground: Could not descend onto ground in time. Will try again later"
                );
            }
            attempts += 1;
        }
        transform.translation.y += 1.;
        commands.entity(entity).remove::<SnapToGround>();
    }
}
