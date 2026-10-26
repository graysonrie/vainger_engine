use crate::prelude::*;

/// UV rect in 0–1 atlas space for one tile slice.
///
/// Can be useful for defining a region in a texture atlas, such as a slice region.
#[derive(Clone, Debug, Copy)]
pub struct TileSprite {
    pub uv: Rect,
}

/// Represents a certain tag in an Aseprite asset. Can be converted into an `AseAnimation` with `into`
#[derive(Clone, Debug)]
pub struct TagSprite {
    pub aseprite: Handle<Aseprite>,
    pub tag: &'static str,
}
impl TagSprite {
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
