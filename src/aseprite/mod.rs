//! Aseprite queries, hitboxes from opaque pixels, and animation-finished events.
//!
//! Playback itself comes from `bevy_aseprite_ultra` ([`AseAnimation`],
//! [`Animation`]). [`animation`] picks which tag plays.
//! [`crate::app::AppExt::setup_for_default_2d_pixel_game`] adds the animation
//! and hitbox plugins. It does not add [`AsepriteExtPlugin`].

use std::time::Duration;

use crate::{aseprite::types::TileSprite, prelude::*};
pub mod hitbox;
pub mod types;
pub mod util;
pub mod animation;

/// Hitbox taken from the opaque pixels of one Aseprite frame.
///
/// [`Hitbox`]'s anchor is [`Anchor::CENTER`]: the origin is the center of that
/// rectangle, not the center of the canvas. [`Self::sprite_anchor`] is the
/// full-canvas sprite anchor that puts this same point on the entity transform.
#[derive(Clone, Debug)]
pub struct FrameHitbox {
    /// Box around the opaque pixels. Its anchor is [`Anchor::CENTER`], so the
    /// box center is the origin of this hitbox, not the canvas center.
    pub hitbox: Hitbox,
    /// Sprite [`Anchor`] that puts the hitbox center on the entity transform.
    ///
    /// Assign this to the sprite. [`hitbox::AsepriteHitbox`] does that when it
    /// inserts the component. Leaving the sprite at [`Anchor::CENTER`] draws
    /// the canvas center on the transform instead, and the hitbox will not
    /// line up with the pixels.
    pub sprite_anchor: Anchor,
}

/// Read-only lookup over loaded [`Aseprite`] assets, their atlases, and images.
///
/// Use it as a system parameter. Methods return [`None`] when the asset is
/// still loading or the tag or slice name is wrong; they do not retry next frame.
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
    /// Will return `None` if the slice does not exist, the atlas is missing,
    /// or the image has a zero size.
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
    pub fn hitbox_from_tag(
        &self,
        handle: &Handle<Aseprite>,
        tag_name: &str,
    ) -> Option<FrameHitbox> {
        let ase = self.get(handle)?;
        let tag = ase.tags.get(tag_name)?;
        let layout = self.layouts.get(&ase.atlas_layout)?;
        let image = self.images.get(&ase.atlas_image)?;
        let frame = usize::from(*tag.range.start());
        let atlas_rect = layout.textures.get(ase.get_atlas_index(frame))?;
        let opaque = opaque_bounds(image, *atlas_rect)?;
        let frame_bounds = URect {
            min: opaque.min.saturating_sub(atlas_rect.min),
            max: opaque.max.saturating_sub(atlas_rect.min),
        };
        frame_hitbox(atlas_rect.size(), frame_bounds)
    }

    /// Rewinds `state` to the first frame of `tag` and clears its elapsed time.
    ///
    /// `ase` is only read, for its [`AseAnimation::aseprite`] handle. This does
    /// not change which tag `ase` is playing. Set that yourself (or use
    /// [`animation::AnimationResolver`]) before calling this, or the playhead
    /// and the tag will disagree until the next animation tick.
    ///
    /// Returns an error if the [`Aseprite`] asset is not loaded yet, or if
    /// `tag` is not in the file.
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    #[test]
    fn offset_rectangle_origin_is_its_center() {
        let canvas = uvec2(64, 64);
        // Pixels x=10..30, y=20..50 in top-left space. Center is left and below the canvas center.
        let frame = frame_hitbox(
            canvas,
            URect {
                min: uvec2(10, 20),
                max: uvec2(30, 50),
            },
        )
        .expect("bounds");

        assert_eq!(frame.hitbox.size(), uvec2(20, 30));
        let anchor = frame.sprite_anchor.as_vec();
        assert!((anchor.x - (-12.0 / 64.0)).abs() < 1e-5);
        assert!((anchor.y - (-3.0 / 64.0)).abs() < 1e-5);

        // Entity transform is the rectangle center, so the box extends half its size each way.
        // Those edges are the painted pixels: x=10..30, y=20..50 on the 64² canvas.
        let corners = frame.hitbox.four_corners_in_world(Vec2::ZERO);
        assert!((corners[0] - Vec2::new(-10.0, -15.0)).length() < 1e-4);
        assert!((corners[2] - Vec2::new(10.0, 15.0)).length() < 1e-4);
    }

    #[test]
    fn opaque_bounds_ignore_transparent_pixels() {
        let width = 8;
        let height = 8;
        let mut data = vec![0u8; (width * height * 4) as usize];
        for y in 1..3 {
            for x in 2..5 {
                let index = ((y * width + x) * 4) as usize;
                data[index + 3] = 255;
            }
        }
        let image = Image::new(
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        let bounds = opaque_bounds(
            &image,
            URect {
                min: UVec2::ZERO,
                max: uvec2(width, height),
            },
        )
        .expect("opaque pixels");
        let frame = frame_hitbox(uvec2(width, height), bounds).expect("hitbox");

        assert_eq!(bounds.min, uvec2(2, 1));
        assert_eq!(bounds.max, uvec2(5, 3));
        assert_eq!(frame.hitbox.size(), uvec2(3, 2));
        let anchor = frame.sprite_anchor.as_vec();
        assert!((anchor.x - (-0.5 / 8.0)).abs() < 1e-5);
        assert!((anchor.y - (2.0 / 8.0)).abs() < 1e-5);
    }
}

/// Forwards `bevy_aseprite_ultra`'s finished-animation message as [`AnimationComplete`].
///
/// ### NOTE:
/// [`crate::app::AppExt::setup_for_default_2d_pixel_game`] does not add this.
/// Without it, nothing is listening and [`AnimationComplete`] never fires.
///
/// ### NOTE:
/// Only [`AnimationEvents::Finished`] is forwarded. Looping playback emits
/// [`AnimationEvents::LoopCycleFinished`] instead, so a tag started with
/// [`Animation::play_loop`] (or [`animation::AnimationResolver`], which uses
/// that) will not complete. [`util::AnimationExt::play_once`] is what emits
/// `Finished`. That helper passes `Count(1)`, and the upstream player plays
/// a `Count(1)` tag twice before it fires. See [`util::AnimationExt::play_once`].
pub struct AsepriteExtPlugin;

impl Plugin for AsepriteExtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, watch_for_anim_complete);
    }
}

/// Triggered on the animated entity when its non-looping animation finishes.
///
/// Observe it with `On<AnimationComplete>`. The wrapped entity is the one that
/// was playing the animation. See [`AsepriteExtPlugin`] for when this is sent.
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
