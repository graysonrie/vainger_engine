use crate::prelude::*;

/// UV rect in 0–1 atlas space for one tile slice.
///
/// Can be useful for defining a region in a texture atlas, such as a slice region.
#[derive(Clone, Debug, Copy)]
pub struct TileSprite {
    /// UV rectangle in `0..=1`, min corner first.
    pub uv: Rect,
}

/// Represents a certain tag in an Aseprite asset. Can be converted into an `AseAnimation` with `into`
#[derive(Clone, Debug)]
pub struct TagSprite {
    /// Loaded sheet that contains `tag`.
    pub aseprite: Handle<Aseprite>,
    /// Tag name inside that file. Has to be a `'static` string, typically a literal.
    pub tag: &'static str,
}
impl TagSprite {
    /// A tag handle. Convert it with `AseAnimation::from(tag_sprite)` when you spawn.
    pub fn new(aseprite: Handle<Aseprite>, tag: &'static str) -> Self {
        Self { aseprite, tag }
    }
}

impl From<TagSprite> for AseAnimation {
    fn from(value: TagSprite) -> Self {
        Self {
            aseprite: value.aseprite,
            animation: Animation::tag(value.tag),
        }
    }
}
