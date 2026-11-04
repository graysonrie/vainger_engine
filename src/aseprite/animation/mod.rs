use crate::{aseprite::AseInfo, prelude::*};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FullAsepriteAnimationSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AnimationResolutionSet {
    Reset,
    Resolve,
    Apply,
}

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

#[derive(Component)]
pub struct AnimationResolver {
    pub selected_tag: &'static str,
    priority: u8,
    initial_tag: &'static str,
}

impl AnimationResolver {
    pub fn new(initial_tag: &'static str) -> Self {
        Self {
            selected_tag: initial_tag,
            priority: 0,
            initial_tag,
        }
    }
    /// A higher priority wins over lower ones
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
