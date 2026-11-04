//! Picks one Aseprite tag per entity per frame, by priority.
//!
//! Put [`AnimationResolver`] on the same entity as [`AseAnimation`].
//! [`AseAnimation`] already requires [`AnimationState`]. Each [`Update`], in
//! [`FullAsepriteAnimationSet`]:
//!
//! 1. [`AnimationResolutionSet::Reset`] puts the selection back to the resolver's
//!    initial tag at priority 0.
//! 2. Your systems in [`AnimationResolutionSet::Resolve`] call
//!    [`AnimationResolver::propose`].
//! 3. [`AnimationResolutionSet::Apply`] plays the winning tag, looping, and
//!    rewinds the playhead when the tag changed.
//!
//! ```ignore
//! fn pick_anim(mut query: Query<&mut AnimationResolver>, jumping: Query<&Jumping>) {
//!     for mut resolver in &mut query {
//!         resolver.propose("run", 1);
//!         if jumping.single().is_ok() {
//!             resolver.propose("jump", 2);
//!         }
//!     }
//! }
//!
//! app.add_systems(Update, pick_anim.in_set(AnimationResolutionSet::Resolve));
//! ```
//!
//! ### NOTE:
//! Proposals die at the start of the next frame. A system that proposes once,
//! on enter or on a key press, leaves the entity on the initial tag again next
//! frame. Propose the tags that should be playing every frame.
//!
//! ### NOTE:
//! The propose system has to be in [`AnimationResolutionSet::Resolve`]. A
//! system in plain [`Update`] can run before the reset (the proposal is wiped)
//! or after the apply (it shows up one frame late, then gets wiped).
//!
//! ### NOTE:
//! Priority `0` never wins. After the reset the initial tag is already at
//! priority 0, and [`AnimationResolver::propose`] only replaces the selection
//! when the new priority is strictly greater. Start gameplay tags at 1.
//! Equal priorities keep whichever proposal landed first this frame, so give
//! competing tags different priorities if the winner should be predictable.

use crate::{aseprite::AseInfo, prelude::*};

/// Parent set, in [`Update`], for [`AnimationResolutionSet`]. The three child
/// sets run in order inside it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullAsepriteAnimationSet;

/// Ordered stages of tag selection. See the module docs for how to use them.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationResolutionSet {
    /// Clears every resolver back to its initial tag. Do not add game systems here.
    Reset,
    /// Where game systems call [`AnimationResolver::propose`].
    Resolve,
    /// Writes the winning tag onto [`AseAnimation`]. Do not add game systems here.
    Apply,
}

/// Registers the reset and apply systems.
///
/// Already added by [`crate::app::AppExt::setup_for_default_2d_pixel_game`].
/// You still add your own systems to [`AnimationResolutionSet::Resolve`].
pub struct AsepriteAnimationPlugin;

impl Plugin for AsepriteAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                AnimationResolutionSet::Reset,
                AnimationResolutionSet::Resolve,
                AnimationResolutionSet::Apply,
            )
                .in_set(FullAsepriteAnimationSet)
                .chain(),
        )
        .add_systems(Update, reset_resolver.in_set(AnimationResolutionSet::Reset))
        .add_systems(Update, apply_resolver.in_set(AnimationResolutionSet::Apply));
    }
}

/// The tag this entity should be playing, chosen fresh every frame.
///
/// `selected_tag` is whatever won this frame. Read it if something else needs
/// the decision; the apply system is what actually calls
/// [`Animation::play_loop`]. Tags are `'static` string slices, so they have to
/// be literals (or another string that lives for the whole program), not a
/// formatted [`String`].
///
/// See the [module docs](crate::aseprite::animation) for ordering and the priority-0 pitfall.
#[derive(Component)]
pub struct AnimationResolver {
    /// Tag that won this frame. Equals the initial tag until something proposes better.
    pub selected_tag: &'static str,
    priority: u8,
    initial_tag: &'static str,
}

impl AnimationResolver {
    /// Resting tag, used whenever nothing proposes a priority above 0.
    ///
    /// `initial_tag` must be a tag that exists in the entity's Aseprite file.
    /// A missing tag logs a warning from the apply system and does not change
    /// the playhead.
    pub fn new(initial_tag: &'static str) -> Self {
        Self {
            selected_tag: initial_tag,
            priority: 0,
            initial_tag,
        }
    }
    /// Offer `tag` at `priority` for this frame.
    ///
    /// A higher `priority` replaces a lower one. An equal or lower `priority`
    /// is ignored, including `0`, which is the initial tag's priority after
    /// the per-frame reset. Call this from a system in
    /// [`AnimationResolutionSet::Resolve`], every frame the tag should keep playing.
    pub fn propose(&mut self, tag: &'static str, priority: u8) {
        if priority > self.priority {
            self.priority = priority;
            self.selected_tag = tag;
        }
    }
    fn reset(&mut self) {
        self.selected_tag = self.initial_tag;
        self.priority = 0;
    }
}

fn reset_resolver(mut q: Query<&mut AnimationResolver>) {
    for mut resolver in &mut q {
        resolver.reset();
    }
}

fn apply_resolver(
    mut q: Query<(&mut AseAnimation, &AnimationResolver, &mut AnimationState)>,
    ase_info: AseInfo,
) {
    for (mut animation, resolver, mut anim_state) in &mut q {
        if let Some(ref tag) = animation.animation.tag
            && tag == resolver.selected_tag
        {
            // Already playing this animation
            continue;
        }
        animation.animation.play_loop(resolver.selected_tag);

        // TODO: may not be needed
        ase_info
            .reset_anim(&mut animation, resolver.selected_tag, &mut anim_state)
            .warn_if_err();
    }
}
