//! Shorthand constructors for `bevy_aseprite_ultra` animations.

use crate::prelude::*;

/// Build an [`AseAnimation`] from a tag name.
pub trait AseAnimationExt {
    /// Animation that plays `tag_name` on `handle`.
    ///
    /// The repeat mode is whatever [`Animation::tag`] defaults to (looping).
    /// This does not check that the tag exists; a bad name fails later, when
    /// the animation system tries to sample it.
    fn from_tag(handle: Handle<Aseprite>, tag_name: &str) -> AseAnimation;
}

impl AseAnimationExt for AseAnimation {
    fn from_tag(handle: Handle<Aseprite>, tag_name: &str) -> AseAnimation {
        AseAnimation {
            aseprite: handle,
            animation: Animation::tag(tag_name),
        }
    }
}

/// Extra playback modes for [`Animation`].
pub trait AnimationExt {
    /// Play `tag_name`, then stop.
    ///
    /// This calls [`Animation::play`] with [`AnimationRepeat::Count`]`(1)`.
    /// [`Animation::play_loop`] never emits the finished event.
    ///
    /// ### NOTE:
    /// `bevy_aseprite_ultra` keeps replaying a [`AnimationRepeat::Count`] while
    /// the count is above zero, and emits [`AnimationEvents::Finished`] only
    /// once the count has already hit zero. `Count(1)` therefore plays the tag
    /// **twice**, and [`super::AnimationComplete`] fires after the second pass.
    fn play_once(&mut self, tag_name: &str);
}

impl AnimationExt for Animation {
    fn play_once(&mut self, tag_name: &str) {
        self.play(tag_name, AnimationRepeat::Count(1))
    }
}
