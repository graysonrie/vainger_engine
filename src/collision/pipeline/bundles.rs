use crate::aseprite::hitbox::AsepriteHitbox;

use super::*;

#[derive(Component, Default)]
pub struct PlatformerPlayer;

#[derive(Bundle, Default)]
pub struct PlatformerActorBundle {
    actor: CollisionActor,
    friction: Friction,
    velocity: Velocity2D,
    contacts: CollisionContacts,
    speed: MoveSpeed,
}

#[derive(Bundle)]
pub struct AsepritePlatformerActorBundle {
    pub character: PlatformerActorBundle,
    pub hitbox: AsepriteHitbox,
    pub ase: AseAnimation,
    pub sprite: Sprite,
    pub transform: Transform,
}
impl AsepritePlatformerActorBundle {
    pub fn new(aseprite: Handle<Aseprite>, tag: &'static str, pos: Vec2) -> Self {
        Self {
            character: PlatformerActorBundle::default(),
            hitbox: AsepriteHitbox::new(aseprite.clone()),
            ase: AseAnimation::from_tag(aseprite, tag),
            sprite: Sprite::default(),
            transform: Transform::from_translation(pos.extend(0.)),
        }
    }
}
