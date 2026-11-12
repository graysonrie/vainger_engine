use crate::{aseprite::AseInfo, prelude::*};

pub(crate) struct AsepriteHitboxPlugin;

impl Plugin for AsepriteHitboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(AsepriteHitbox::handle_add);
    }
}

/// Will insert a [`Hitbox`] and [`Anchor`] on the entity, given that they have a tag named "hitbox" (or whatever you specify.)
/// An error will be logged if this failed
#[derive(Component)]
pub struct AsepriteHitbox {
    /// The tag name to look for that contains the hitbox. Default: "hitbox"
    tag_name: &'static str,
    aseprite: Handle<Aseprite>,
}
impl AsepriteHitbox {
    pub fn new(aseprite: Handle<Aseprite>) -> Self {
        Self {
            tag_name: "hitbox",
            aseprite,
        }
    }

    pub fn new_with_tag_name(aseprite: Handle<Aseprite>, tag_name: &'static str) -> Self {
        Self { tag_name, aseprite }
    }
    fn handle_add(
        event: On<Add, Self>,
        q: Query<(Entity, &Self)>,
        ase: AseInfo,
        mut commands: Commands,
    ) {
        let Ok((entity, this)) = q.get(event.entity) else {
            return;
        };
        let Some(frame) = ase.hitbox_from_tag(&this.aseprite, this.tag_name) else {
            warn!(
                "Encountered aseprite hitbox with no opaque pixels! Tag name: {}",
                this.tag_name
            );
            return;
        };
        let mut e = commands.entity(entity);
        e.insert((frame.sprite_anchor, frame.hitbox));
    }
}
