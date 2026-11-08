//! Smooth follow for [`PixelPerfectCamera`](super::PixelPerfectCamera).

use crate::prelude::*;

/// Makes one [`PixelPerfectCamera`] chase this entity's `x` and `y`.
///
/// The camera keeps its own Z. Put this on a gameplay entity, not on the camera.
/// A component on the camera itself is ignored by the follow query.
///
/// [`Self::camera`] picks which screen follows. [`None`] follows the only camera,
/// which is the one-screen case ([`CameraPanner2dBundle`] uses that). With two or
/// more cameras, [`None`] does not move any of them.
///
/// ### NOTE:
/// Two followers that resolve to the same camera: one is applied and the extra
/// logs a warning every frame.
#[derive(Component)]
pub struct PixelPerfectCameraFollow {
    /// A value like `4.` is a good amount.
    pub decay_rate: f32,
    /// Screen that should chase this entity. [`None`] means the only camera.
    pub camera: Option<Entity>,
}

#[allow(unused)]
impl PixelPerfectCameraFollow {
    /// Follow the only camera.
    ///
    /// The larger the decay rate, the faster the camera will track the target.
    /// Default is 4.0.
    pub fn new(decay_rate: f32) -> Self {
        Self {
            decay_rate,
            camera: None,
        }
    }

    /// Follow `camera`.
    ///
    /// The larger the decay rate, the faster the camera will track the target.
    pub fn for_camera(camera: Entity, decay_rate: f32) -> Self {
        Self {
            decay_rate,
            camera: Some(camera),
        }
    }
}

impl Default for PixelPerfectCameraFollow {
    fn default() -> Self {
        Self::new(4.)
    }
}

/// Runs the follow system in [`PixelPerfectCameraControlsSet::Follow`].
///
/// Added by [`PixelPerfectCameraPlugin`]. Adding it again registers the system twice.
pub struct PixelPerfectCameraFollowPlugin;

impl Plugin for PixelPerfectCameraFollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            handle_follow.in_set(PixelPerfectCameraControlsSet::Follow),
        );
    }
}

fn handle_follow(
    mut cameras: Query<
        (Entity, &mut Transform),
        (With<PixelPerfectCamera>, Without<PixelPerfectCameraFollow>),
    >,
    followers: Query<(&Transform, &PixelPerfectCameraFollow), Without<PixelPerfectCamera>>,
    time: Res<Time>,
) {
    let ids: Vec<Entity> = cameras.iter().map(|(entity, _)| entity).collect();
    let mut chosen: Vec<ChosenFollow> = Vec::new();

    for (transform, follow) in &followers {
        let Some(camera) = resolve_follow(follow.camera, &ids) else {
            continue;
        };
        if chosen.iter().any(|choice| choice.camera == camera) {
            warn!("more than one PixelPerfectCameraFollow targets {camera}; ignoring the extra");
            continue;
        }
        chosen.push(ChosenFollow {
            camera,
            target: transform.translation,
            decay_rate: follow.decay_rate,
        });
    }

    let dt = time.delta_secs();
    for (entity, mut transform) in &mut cameras {
        let Some(choice) = chosen.iter().find(|choice| choice.camera == entity) else {
            continue;
        };
        let desired_pos = Vec3::new(choice.target.x, choice.target.y, transform.translation.z);
        transform
            .translation
            .smooth_nudge(&desired_pos, choice.decay_rate, dt);
    }
}

struct ChosenFollow {
    camera: Entity,
    target: Vec3,
    decay_rate: f32,
}

fn resolve_follow(target: Option<Entity>, cameras: &[Entity]) -> Option<Entity> {
    match target {
        Some(camera) if cameras.contains(&camera) => Some(camera),
        Some(camera) => {
            warn!("PixelPerfectCameraFollow targets {camera}, which is not a pixel-perfect camera");
            None
        }
        None if cameras.len() == 1 => Some(cameras[0]),
        None => None,
    }
}
