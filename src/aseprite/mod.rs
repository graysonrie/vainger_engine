use crate::{aseprite::types::TileSprite, prelude::*};
pub mod types;

/// Provides useful info about Aseprite assets. Does not mutate anything.
#[derive(SystemParam)]
pub struct AseInfo<'w> {
    aseprites: Res<'w, Assets<Aseprite>>,
    layouts: Res<'w, Assets<TextureAtlasLayout>>,
    images: Res<'w, Assets<Image>>,
}

impl AseInfo<'_> {
    pub(crate) fn get(&self, handle: &Handle<Aseprite>) -> Option<&Aseprite> {
        self.aseprites.get(handle)
    }

    /// Finds the given slice in the Aseprite file and returns the bounding
    /// rect of it.
    ///
    /// Returns `None` if there was no such slice
    pub fn size_of_slice(&self, handle: &Handle<Aseprite>, name: &str) -> Option<Vec2> {
        let slice = self.get(handle)?.slices.get(name)?;
        Some(slice.rect.size())
    }

    /// Returns the canvas size you defined for this Aseprite file
    ///
    /// Returns `None` if the texture layout couldn't be found for some
    /// reason
    #[allow(dead_code)]
    pub fn canvas_size(&self, handle: &Handle<Aseprite>) -> Option<Vec2> {
        let ase = self.get(handle)?;
        let layout = self.layouts.get(&ase.atlas_layout)?;
        let first = layout.textures.first()?;
        Some(first.size().as_vec2())
    }

    /// Finds the set of images with this tag and returns the size of
    /// the first image in the set
    ///
    /// Returns `None` if the tag does not exist
    ///
    /// Realistically, you can just use `canvas_size` since a tagged image
    /// takes up the whole Aseprite canvas anyway
    pub fn size_of_tag(&self, handle: &Handle<Aseprite>, tag_name: &str) -> Option<Vec2> {
        let ase = self.get(handle)?;
        let tag = ase.tags.get(tag_name)?;
        let layout = self.layouts.get(&ase.atlas_layout)?;
        let frame = *tag.range.start() as usize;
        let idx = ase.get_atlas_index(frame);
        let rect = layout.textures.get(idx)?;
        Some(rect.size().as_vec2())
    }

    /// Returns the UV coords of the slice on its texture atlas
    ///
    /// Will return `None` if the slice does not exis
    pub fn tile_sprite_for_slice(
        &self,
        handle: &Handle<Aseprite>,
        slice_name: &str,
    ) -> Option<TileSprite> {
        let ase = self.get(handle)?;
        let slice = ase.slices.get(slice_name)?;
        let layout = self.layouts.get(&ase.atlas_layout)?;
        let image = self.images.get(&ase.atlas_image)?;
        let rect = layout.textures.get(slice.atlas_id)?;
        let size = image.size_f32();
        if size.x <= 0.0 || size.y <= 0.0 {
            return None;
        }

        Some(TileSprite {
            uv: Rect::from_corners(
                Vec2::new(rect.min.x as f32 / size.x, rect.min.y as f32 / size.y),
                Vec2::new(rect.max.x as f32 / size.x, rect.max.y as f32 / size.y),
            ),
        })
    }
}
