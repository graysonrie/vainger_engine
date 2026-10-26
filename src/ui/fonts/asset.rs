use bevy::asset::{AssetLoader, ParseAssetPathError};
use ron::de::SpannedError;
use serde::Deserialize;
use thiserror::Error;

use crate::prelude::*;

/// An 8×8 atlas font. Glyph index `n` is codepoint `32 + n`.
///
/// Example .font.ron definition:
///```ignore
/// (
/// image: "sfont_ascii.png",
/// glyph_size: (8, 8),
/// columns: 44,
/// rows: 6,
/// first_code: 32,
/// last_code: 252,
/// )
/// ```
#[derive(Clone, Debug, Asset, TypePath)]
pub struct BitmapFont {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub glyph_size: UVec2,
    first_code: u32,
    last_code: u32,
}

impl BitmapFont {
    pub fn glyph_index(&self, c: char) -> Option<usize> {
        let code = c as u32;
        if (self.first_code..=self.last_code).contains(&code) {
            Some((code - self.first_code) as usize)
        } else {
            None
        }
    }
}

#[derive(Deserialize)]
pub struct BitmapFontSource {
    image: String,
    glyph_size: (u32, u32),
    columns: u32,
    rows: u32,
    first_code: u32,
    last_code: u32,
}

#[derive(Error, Debug)]
pub enum BitmapFontError {
    #[error("no parent")]
    NoParent,

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Asset error")]
    Asset(#[from] ParseAssetPathError),

    #[error("ron error")]
    Deserialization(#[from] SpannedError),
}
#[derive(Default, TypePath)]
pub(crate) struct BitmapFontLoader;

impl AssetLoader for BitmapFontLoader {
    type Asset = BitmapFont;

    type Settings = ();

    type Error = BitmapFontError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut bevy::asset::LoadContext<'_>,
    ) -> Result<BitmapFont, Self::Error> {
        let mut bytes = Vec::new();

        reader.read_to_end(&mut bytes).await?;

        let src: BitmapFontSource = ron::de::from_bytes(&bytes)?;

        let image_path = load_context
            .path()
            .parent()
            .ok_or(BitmapFontError::NoParent)?
            .resolve_str(&src.image)?;

        let image = load_context.load(image_path);

        let glyph_size = uvec2(src.glyph_size.0, src.glyph_size.1);

        let layout = load_context.add_labeled_asset(
            "layout".to_string(),
            TextureAtlasLayout::from_grid(glyph_size, src.columns, src.rows, None, None),
        );

        Ok(BitmapFont {
            image,
            layout,
            glyph_size,
            first_code: src.first_code,
            last_code: src.last_code,
        })
    }
}
