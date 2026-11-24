use super::*;

pub struct DebugHitboxPlugin;

impl Plugin for DebugHitboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (handle_add, handle_remove));
    }
}

#[derive(Component, Default)]
pub struct RenderDebugHitbox;

#[derive(Component)]
struct DebugHitbox;

/// Will add the debug hitbox as a child of the entity, since the entity will likely
/// already have a `Sprite` component on it
fn handle_add(
    q: Query<(&Hitbox, Entity, &Transform), Added<RenderDebugHitbox>>,
    mut commands: Commands,
) {
    for (hitbox, entity, transform) in q {
        // we divide by transform.scale since the parent likely has a
        // scale greater than 1, which would throw off the sizing since the
        // child inherits the parent's scale
        commands.entity(entity).with_child((
            DebugHitbox,
            Sprite::from_color(
                Color::srgb_u8(240, 0, 0).with_alpha(0.6),
                hitbox.size.as_vec2() / transform.scale.truncate(),
            ),
            hitbox.anchor,
        ));
    }
}

fn handle_remove(
    debug_hitboxes: Query<(Entity, &ChildOf), With<DebugHitbox>>,
    mut commands: Commands,
    mut removed: RemovedComponents<RenderDebugHitbox>,
) {
    for removed_e in removed.read() {
        for (entity, child_of) in debug_hitboxes {
            if removed_e == child_of.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}
