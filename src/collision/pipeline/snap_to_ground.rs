use super::*;

pub(super) struct SnapToGroundPlugin;

impl Plugin for SnapToGroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            snap_to_ground
                .after(Collision2DSet::Move)
                .before(Collision2DSet::Collide),
        );
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
