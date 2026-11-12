use crate::prelude::*;
use std::error::Error;
use std::fmt;

use aseprite_loader::loader::{AsepriteFile, LayerSelection};
use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages, io::Reader},
    image::ImageSampler,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

/// Implements an Asset Loader for loading in Aseprite files as images, which is what LDtk maps use.
pub struct AssetHelpersPlugin;

impl Plugin for AssetHelpersPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_loader(AsepriteImageLoader);
    }
}

/// Loads an `.aseprite` file as a flat [`Image`] so LDtk tilesets can reference them.
///
/// `bevy_ecs_ldtk` asks the asset server for an [`Image`]. `bevy_aseprite_ultra` only
/// registers an `Aseprite` loader, so those tileset loads would otherwise fail.
#[derive(Default, TypePath)]
struct AsepriteImageLoader;

#[derive(Debug)]
struct AsepriteImageError(String);

impl fmt::Display for AsepriteImageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for AsepriteImageError {}

impl AssetLoader for AsepriteImageLoader {
    type Asset = Image;
    type Settings = ();
    type Error = AsepriteImageError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Image, Self::Error> {
        let mut bytes = Vec::new();
        reader
            .read_to_end(&mut bytes)
            .await
            .map_err(|err| AsepriteImageError(err.to_string()))?;

        let file = AsepriteFile::load(&bytes).map_err(|err| AsepriteImageError(err.to_string()))?;
        if file.frames().is_empty() {
            return Err(AsepriteImageError(
                "aseprite tileset has no frames".to_string(),
            ));
        }

        let (width, height) = file.size();
        let mut buffer = vec![0; usize::from(width) * usize::from(height) * 4];
        file.render_frame(0, buffer.as_mut_slice(), &LayerSelection::Visible)
            .map_err(|err| AsepriteImageError(err.to_string()))?;

        Ok(Image {
            sampler: ImageSampler::nearest(),
            ..Image::new(
                Extent3d {
                    width: u32::from(width),
                    height: u32::from(height),
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                buffer,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )
        })
    }

    fn extensions(&self) -> &[&str] {
        &["aseprite", "ase"]
    }
}
