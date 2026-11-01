use crate::prelude::*;

pub trait AseAnimationExt {
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

pub trait AnimationExt {
    fn play_once(&mut self, tag_name: &str);
}

impl AnimationExt for Animation {
    fn play_once(&mut self, tag_name: &str) {
        self.play(tag_name, AnimationRepeat::Count(1))
    }
}
