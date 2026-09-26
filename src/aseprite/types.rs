use crate::prelude::*;

/// UV rect in 0–1 atlas space for one tile slice.
///
/// Can be useful for defining a region in a texture atlas, such as a slice region.
#[derive(Clone, Debug, Copy)]
pub struct TileSprite {
    pub uv: Rect,
}
