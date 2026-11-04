use crate::{aseprite::AseInfo, prelude::*};

pub(crate) struct AsepriteHitboxPlugin;

impl Plugin for AsepriteHitboxPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(AsepriteHitbox::handle_add);
    }
}

/// On insert, builds a [`Hitbox`] and sprite [`Anchor`] from the opaque pixels
/// of an Aseprite tag.
///
/// The plugin that watches this component is added by
/// [`crate::app::AppExt::setup_for_default_2d_pixel_game`]. [`Self::new`] looks
/// for a tag named `"hitbox"`. [`Self::new_with_tag_name`] picks another one.
///
/// The hitbox anchor is [`Anchor::CENTER`] (the center of the opaque rectangle).
/// The sprite anchor is the one that puts that same point on the entity
/// transform. Position the entity at the hitbox center.
///
/// ### NOTE:
/// The observer runs once, when this component is inserted. If `aseprite` is
/// not loaded yet, or the tag has no opaque pixels, it logs a warning and
/// never tries again. Insert this after the asset has loaded (for example
/// once the loading state has finished), or remove and reinsert it later.
///
/// ### NOTE:
/// `bevy_aseprite_ultra` flattens every visible layer into the frame. Other
/// layers' pixels on that same frame are part of the hitbox. Put the tag on
/// frames whose only opaque pixels are the box you want, or the box will
/// include the rest of the sprite.
///
/// ### NOTE:
/// A successful insert overwrites any [`Hitbox`] and [`Anchor`] already on the
/// entity.
#[derive(Component)]
pub struct AsepriteHitbox {
    /// Tag whose first frame is scanned for opaque pixels. [`Self::new`] uses `"hitbox"`.
    tag_name: &'static str,
    /// Sheet to read. Must already be loaded when this component is inserted.
    aseprite: Handle<Aseprite>,
}
impl AsepriteHitbox {
    /// Hitbox from the first frame of the tag `"hitbox"`.
    ///
    /// See the struct docs for the load-timing pitfall.
    pub fn new(aseprite: Handle<Aseprite>) -> Self {
        Self {
            tag_name: "hitbox",
            aseprite,
        }
    }

    /// Same as [`Self::new`], but the opaque pixels are taken from `tag_name`.
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
