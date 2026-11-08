use crate::prelude::*;
use bevy::camera::visibility::RenderLayers;

/// Reads [`FadeScreen`] messages as processes fades during `Update`
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScreenFadeSet;

pub struct ScreenTransitionPlugin;

impl Plugin for ScreenTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FadeScreen>()
            .add_message::<ScreenFadeComplete>()
            .add_observer(spawn_overlay_on_camera)
            .add_systems(
                Update,
                (
                    FadeScreen::handle_read,
                    handle_screen_fade,
                    handle_in_out_fades,
                )
                    .chain()
                    .in_set(ScreenFadeSet),
            );
    }
}

/// Spawns the screen overlay as a child of the camera.
///
/// The quad starts on no render layer. Layout puts it on a layer only this
/// camera draws, and sizes it to the visible view.
fn spawn_overlay_on_camera(event: On<Add, PixelPerfectCamera>, mut commands: Commands) {
    commands.entity(event.entity).with_children(|parent| {
        parent.spawn((
            Sprite::from_color(Color::BLACK.with_alpha(0.0), Vec2::splat(1.0)),
            Transform::from_xyz(0.0, 0.0, 100.0),
            Visibility::Hidden,
            ScreenFadeOverlay,
            RenderLayers::none(),
        ));
    });
}

/// Requests a fade on one [`PixelPerfectCamera`].
#[derive(Message, Clone, Copy, Debug)]
pub struct FadeScreen {
    /// Screen whose overlay should fade.
    pub camera: Entity,
    pub fade_type: FadeDirective,
    /// In secs
    pub duration: f32,
}
impl FadeScreen {
    pub fn new(camera: Entity, directive: FadeDirective, duration: f32) -> Self {
        Self {
            camera,
            fade_type: directive,
            duration,
        }
    }
    pub fn fade_in(camera: Entity, duration: f32) -> Self {
        Self::new(camera, FadeDirective::In, duration)
    }

    pub fn fade_out(camera: Entity, duration: f32) -> Self {
        Self::new(camera, FadeDirective::Out, duration)
    }

    pub fn fade_in_out(camera: Entity, duration: f32) -> Self {
        Self::new(camera, FadeDirective::in_out(duration), duration)
    }
}

#[derive(Clone, Debug, Copy, PartialEq)]
pub enum FadeDirective {
    In,
    Out,
    InOut { out_duration: f32 },
}
impl FadeDirective {
    pub fn in_out(out_duration: f32) -> Self {
        Self::InOut { out_duration }
    }
}

/// Marker that tells the screen transition to immediately request a fade out upon
/// completion
#[derive(Component)]
struct DoFadeOut {
    duration: f32,
}

impl FadeScreen {
    fn handle_read(
        mut reader: MessageReader<Self>,
        overlays: Query<(Entity, &ChildOf), With<ScreenFadeOverlay>>,
        mut commands: Commands,
    ) {
        for message in reader.read() {
            let Some(entity) = overlays
                .iter()
                .find(|(_, child_of)| child_of.0 == message.camera)
                .map(|(entity, _)| entity)
            else {
                warn!(
                    "Got a screen fade for {}, but that camera has no overlay",
                    message.camera
                );
                continue;
            };
            let to = match message.fade_type {
                FadeDirective::In => 1.,
                FadeDirective::Out => 0.,
                FadeDirective::InOut { .. } => 1.,
            };

            let from = match message.fade_type {
                FadeDirective::In => 0.,
                FadeDirective::Out => 1.,
                FadeDirective::InOut { .. } => 0.,
            };

            let fade_type = match message.fade_type {
                FadeDirective::In => FadeType::In,
                FadeDirective::Out => FadeType::Out,
                FadeDirective::InOut { .. } => FadeType::In,
            };

            commands.entity(entity).insert(ScreenFade {
                alpha: 0.,
                from,
                to,
                elapsed: 0.,
                duration: message.duration,
                fade_type,
            });

            if let FadeDirective::InOut { out_duration } = message.fade_type {
                commands.entity(entity).insert(DoFadeOut {
                    duration: out_duration,
                });
            }
        }
    }
}

/// Marker component for the screen fade overlay
#[derive(Component)]
pub struct ScreenFadeOverlay;

/// Describes an in-process screen fade
#[derive(Component)]
struct ScreenFade {
    /// The current alpha value
    pub alpha: f32,
    /// The alpha value from
    from: f32,
    /// The alpha value to
    to: f32,
    /// In secs
    elapsed: f32,
    /// In secs
    duration: f32,
    fade_type: FadeType,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum FadeType {
    In,
    Out,
}

/// Sent when a fade on `camera` finishes.
///
/// An in-out fade sends [`FadeType::In`] when the screen is covered, then
/// [`FadeType::Out`] when it is clear again.
#[derive(Message, Clone, Debug)]
pub struct ScreenFadeComplete {
    /// Screen whose overlay finished this step.
    pub camera: Entity,
    pub fade_type: FadeType,
}

fn handle_screen_fade(
    mut fade: Query<
        (
            &mut Sprite,
            &mut ScreenFade,
            Entity,
            &ChildOf,
            &mut Visibility,
        ),
        With<ScreenFadeOverlay>,
    >,
    time: Res<Time>,
    mut commands: Commands,
    mut completes: MessageWriter<ScreenFadeComplete>,
) {
    for (mut sprite, mut screen_fade, entity, child_of, mut vis) in &mut fade {
        screen_fade.elapsed = (screen_fade.elapsed + time.delta_secs()).min(screen_fade.duration);
        let t = screen_fade.elapsed / screen_fade.duration;
        screen_fade.alpha = screen_fade.from.lerp(screen_fade.to, t);
        *vis = Visibility::Visible;

        if t >= 1. {
            commands.entity(entity).remove::<ScreenFade>();
            completes.write(ScreenFadeComplete {
                camera: child_of.0,
                fade_type: screen_fade.fade_type,
            });
            if screen_fade.fade_type == FadeType::Out {
                *vis = Visibility::Hidden;
            }
        }

        sprite.color.set_alpha(screen_fade.alpha);
    }
}

fn handle_in_out_fades(
    mut fades: MessageReader<ScreenFadeComplete>,
    q: Query<(Entity, &ChildOf, &DoFadeOut), With<ScreenFadeOverlay>>,
    mut trigger_fade: MessageWriter<FadeScreen>,
    mut commands: Commands,
) {
    for fade in fades.read() {
        if fade.fade_type != FadeType::In {
            continue;
        }
        for (entity, child_of, fadeout) in &q {
            if child_of.0 != fade.camera {
                continue;
            }
            let duration = fadeout.duration;
            commands.entity(entity).remove::<DoFadeOut>();
            trigger_fade.write(FadeScreen::fade_out(fade.camera, duration));
        }
    }
}
