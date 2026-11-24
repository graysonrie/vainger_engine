use crate::aseprite::hitbox::AsepriteHitbox;

use super::*;

/// A platforming character that can move and is affected by gravity
///
/// Does not contain components for jumping
#[derive(Bundle, Default)]
pub struct PlatformerActorBundle {
    actor: CollisionActor,
    friction: Friction,
    velocity: Velocity2D,
    contacts: CollisionContacts,
    speed: MoveSpeed,
    gravity: Gravity,
    move_intent: MoveIntent,
    transform: Transform,
    /// Drawn on whole pixels. Movement still accumulates the fractional position.
    subpixel: SpriteSubpixel,
}

/// Superset of [`PlatformerActorBundle`]. It only adds extra components
/// for the hitbox and Aseprite sprite, so it adds no actual
/// platformer logic
#[derive(Bundle)]
pub struct AsepritePlatformerActorBundle {
    pub character: PlatformerActorBundle,
    pub hitbox: AsepriteHitbox,
    pub ase: AseAnimation,
    pub sprite: Sprite,
}
impl AsepritePlatformerActorBundle {
    pub fn new(aseprite: Handle<Aseprite>, tag: &'static str, pos: Vec2) -> Self {
        let transform = Transform::from_translation(pos.extend(0.));
        Self {
            character: PlatformerActorBundle {
                transform,
                ..Default::default()
            },
            hitbox: AsepriteHitbox::new(aseprite.clone()),
            ase: AseAnimation::from_tag(aseprite, tag),
            sprite: Sprite::default(),
        }
    }
    pub fn with_move_speed(mut self, speed: f32) -> Self {
        self.character.speed.0 = speed;
        self
    }

    pub fn with_z(mut self, z: f32) -> Self {
        self.character.transform.translation.z = z;
        self
    }
}
