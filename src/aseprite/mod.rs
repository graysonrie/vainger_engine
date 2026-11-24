use std::time::Duration;

use crate::{aseprite::types::TileSprite, prelude::*};
pub mod animation;
pub mod hitbox;
pub mod types;
pub mod util;

/// Hitbox taken from the opaque pixels of one Aseprite frame.
///
/// [`Hitbox`]'s anchor is [`Anchor::CENTER`]: the origin is the center of that
/// rectangle, not the center of the canvas. [`Self::sprite_anchor`] is the
/// full-canvas sprite anchor that puts this same point on the entity transform.
#[derive(Clone, Debug)]
pub struct FrameHitbox {
    pub hitbox: Hitbox,
    pub sprite_anchor: Anchor,
}

/// Provides useful info about Aseprite assets. Does not mutate anything.
#[derive(SystemParam)]
pub struct AseInfo<'w> {
    aseprites: Res<'w, Assets<Aseprite>>,
    layouts: Res<'w, Assets<TextureAtlasLayout>>,
    images: Res<'w, Assets<Image>>,
    asset_server: Res<'w, AssetServer>,
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

    /// Builds a [`FrameHitbox`] from the opaque pixels of the first frame of `tag_name`.
    ///
    /// The loader flattens every layer into the frame, so pixels from other
    /// layers on that frame are included. Returns `None` when the tag, atlas, or
    /// opaque pixels are missing.
    ///
    /// Because by default a FrameHitbox has its origin in the center, if the hitbox
    /// dimensions are uneven, it causes the origin to land on a fractional position,
    /// which can screw up rendering with the pixel perfect camera, which is why
    /// `warn_if_uneven` is here if you want to be wary of that
    pub fn hitbox_from_tag(
        &self,
        handle: &Handle<Aseprite>,
        tag_name: &str,
        warn_if_uneven: bool,
    ) -> Option<FrameHitbox> {
        let ase = self.get(handle)?;
        let tag = ase.tags.get(tag_name)?;
        let layout = self.layouts.get(&ase.atlas_layout)?;
        let image = self.images.get(&ase.atlas_image)?;
        let frame = usize::from(*tag.range.start());
        let atlas_rect = layout.textures.get(ase.get_atlas_index(frame))?;
        let opaque = opaque_bounds(image, *atlas_rect)?;

        if warn_if_uneven && (opaque.size().x % 2 != 0 || opaque.size().y % 2 != 0) {
            let path = self
                .asset_server
                .get_path(handle)
                .map(|path| path.to_string())
                .unwrap_or_else(|| handle.id().to_string());

            warn!("hitbox_from_tag: uneven hitbox in {path} (tag: {tag_name})");
        }

        let frame_bounds = URect {
            min: opaque.min.saturating_sub(atlas_rect.min),
            max: opaque.max.saturating_sub(atlas_rect.min),
        };
        frame_hitbox(atlas_rect.size(), frame_bounds)
    }

    /// Forces the animation playhead back to the start and renders the frame
    pub fn reset_anim(
        &self,
        ase: &mut AseAnimation,
        tag: &str,
        state: &mut AnimationState,
    ) -> anyhow::Result<()> {
        let Some(sprite) = self.aseprites.get(&ase.aseprite) else {
            return Err(anyhow!("Aseprite has no sprite"));
        };

        let Some(meta) = sprite.tags.get(tag) else {
            return Err(anyhow!("Not a valid tag: {tag}"));
        };

        state.current_frame = *meta.range.start();
        state.relative_frame = 0;
        state.elapsed = Duration::ZERO;

        Ok(())
    }
}

/// `bounds` is in canvas pixels, origin at the top left, `max` exclusive.
fn frame_hitbox(canvas: UVec2, bounds: URect) -> Option<FrameHitbox> {
    let size = bounds.max.saturating_sub(bounds.min);
    if size.x == 0 || size.y == 0 || canvas.x == 0 || canvas.y == 0 {
        return None;
    }

    // Aseprite y grows down. Sprite space y grows up, with the origin at the canvas center.
    let center_x = (bounds.min.x + bounds.max.x) as f32 / 2.0 - canvas.x as f32 / 2.0;
    let center_y = canvas.y as f32 / 2.0 - (bounds.min.y + bounds.max.y) as f32 / 2.0;

    Some(FrameHitbox {
        hitbox: Hitbox::new(size.x, size.y, Anchor::CENTER),
        sprite_anchor: Anchor(Vec2::new(
            center_x / canvas.x as f32,
            center_y / canvas.y as f32,
        )),
    })
}

/// Inclusive-min, exclusive-max bounds of pixels with any alpha, in image space.
fn opaque_bounds(image: &Image, region: URect) -> Option<URect> {
    let mut min = UVec2::MAX;
    let mut max = UVec2::MIN;
    let mut found = false;

    for y in region.min.y..region.max.y {
        for x in region.min.x..region.max.x {
            let alpha = image.get_color_at(x, y).ok()?.alpha();
            if alpha <= 0.0 {
                continue;
            }
            found = true;
            min = min.min(uvec2(x, y));
            max = max.max(uvec2(x + 1, y + 1));
        }
    }

    if !found {
        return None;
    }
    Some(URect { min, max })
}

pub struct AsepriteExtPlugin;

impl Plugin for AsepriteExtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, watch_for_anim_complete);
    }
}

#[derive(EntityEvent)]
pub struct AnimationComplete(Entity);

fn watch_for_anim_complete(mut events: MessageReader<AnimationEvents>, mut commands: Commands) {
    for event in events.read() {
        let AnimationEvents::Finished(e) = event else {
            continue;
        };
        commands.entity(*e).trigger(AnimationComplete);
    }
}
