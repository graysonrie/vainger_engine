use crate::prelude::*;

use super::{
    ActiveBackgrounds, BackgroundFade, BackgroundSpawner, ParallaxBackground, ParallaxTile,
};

/// Request a crossfade to another named [`super::BackgroundDef`].
///
/// `duration` of `0.0` swaps instantly.
#[derive(Message, Clone, Debug)]
pub struct CrossfadeBackground {
    pub to: &'static str,
    pub duration: f32,
}

impl CrossfadeBackground {
    #[allow(dead_code)]
    pub fn new(to: &'static str, duration: f32) -> Self {
        Self { to, duration }
    }
}

pub(crate) fn handle_crossfade_messages(
    mut messages: MessageReader<CrossfadeBackground>,
    mut spawner: BackgroundSpawner,
    mut active: ResMut<ActiveBackgrounds>,
    backgrounds: Query<&ParallaxBackground>,
    mut sprites: Query<(&ParallaxTile, &mut Sprite)>,
) {
    for request in messages.read() {
        start_crossfade(
            request,
            &mut spawner,
            &mut active,
            &backgrounds,
            &mut sprites,
        );
    }
}

fn start_crossfade(
    request: &CrossfadeBackground,
    spawner: &mut BackgroundSpawner,
    active: &mut ActiveBackgrounds,
    backgrounds: &Query<&ParallaxBackground>,
    sprites: &mut Query<(&ParallaxTile, &mut Sprite)>,
) {
    if let Some(fade) = &active.fade {
        if background_id(backgrounds, fade.to) == Some(request.to) {
            return;
        }
    } else if let Some(current) = active.current
        && background_id(backgrounds, current) == Some(request.to)
    {
        return;
    }


    if request.duration <= 0.0 {
        despawn_active(spawner, active);
        active.current = spawner.spawn(request.to, 1.0);
        return;
    }

    let from = if let Some(fade) = active.fade.take() {
        spawner.despawn(fade.from);
        set_stack_alpha(sprites, fade.to, 1.0);
        fade.to
    } else if let Some(current) = active.current.take() {
        current
    } else {
        active.current = spawner.spawn(request.to, 1.0);
        return;
    };

    if background_id(backgrounds, from) == Some(request.to) {
        active.current = Some(from);
        set_stack_alpha(sprites, from, 1.0);
        return;
    }

    let Some(incoming) = spawner.spawn(request.to, 0.0) else {
        active.current = Some(from);
        return;
    };

    active.fade = Some(BackgroundFade {
        from,
        to: incoming,
        elapsed: 0.0,
        duration: request.duration,
    });
}

pub(crate) fn apply_background_fade(
    time: Res<Time>,
    mut active: ResMut<ActiveBackgrounds>,
    mut commands: Commands,
    mut sprites: Query<(&ParallaxTile, &mut Sprite)>,
) {
    let Some(fade) = active.fade.as_mut() else {
        return;
    };

    fade.elapsed += time.delta_secs();
    let t = if fade.duration <= 0.0 {
        1.0
    } else {
        (fade.elapsed / fade.duration).clamp(0.0, 1.0)
    };

    set_stack_alpha(&mut sprites, fade.from, 1.0 - t);
    set_stack_alpha(&mut sprites, fade.to, t);

    if t < 1.0 {
        return;
    }

    let from = fade.from;
    let to = fade.to;
    commands.entity(from).despawn();
    set_stack_alpha(&mut sprites, to, 1.0);
    active.fade = None;
    active.current = Some(to);
}

fn background_id(backgrounds: &Query<&ParallaxBackground>, entity: Entity) -> Option<&'static str> {
    backgrounds.get(entity).ok().map(|bg| bg.id)
}

fn set_stack_alpha(sprites: &mut Query<(&ParallaxTile, &mut Sprite)>, root: Entity, alpha: f32) {
    for (tile, mut sprite) in sprites.iter_mut() {
        if tile.root == root {
            sprite.color.set_alpha(alpha);
        }
    }
}

fn despawn_active(spawner: &mut BackgroundSpawner, active: &mut ActiveBackgrounds) {
    if let Some(fade) = active.fade.take() {
        spawner.despawn(fade.from);
        spawner.despawn(fade.to);
    }
    if let Some(current) = active.current.take() {
        spawner.despawn(current);
    }
}
