use crate::{assets::EngineArtHandles, prelude::*, ui::fonts::BitmapFont};

/// UFO 50 `sFontAscii` is packed left-to-right, top-to-bottom as 8×8 cells.
/// Frame 0 is `' '` (code 32); the last defined glyph is code 252.
const UFO_FONT_GLYPH_SIZE: u32 = 8;
const UFO_FONT_COLUMNS: u32 = 44;
const UFO_FONT_ROWS: u32 = 6;
const UFO_FONT_FIRST_CODE: u32 = 32;
const UFO_FONT_LAST_CODE: u32 = 252;

/// The `sFontAscii` font
pub fn default_font(world: &mut World) -> BitmapFont {
    let image = world
        .resource::<EngineArtHandles>()
        .default_ufo_font
        .clone();

    let layout =
        world
            .resource_mut::<Assets<TextureAtlasLayout>>()
            .add(TextureAtlasLayout::from_grid(
                UVec2::splat(UFO_FONT_GLYPH_SIZE),
                UFO_FONT_COLUMNS,
                UFO_FONT_ROWS,
                None,
                None,
            ));

    BitmapFont {
        image,
        layout,
        glyph_size: UVec2::splat(UFO_FONT_GLYPH_SIZE),
        first_code: UFO_FONT_FIRST_CODE,
        last_code: UFO_FONT_LAST_CODE,
    }
}
