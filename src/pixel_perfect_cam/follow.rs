use crate::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum FollowType {
    Smooth { decay_rate: f32 },
    Instant,
}

/// Marker component that tells the pixel perfect camera to follow this entity
#[derive(Component)]
pub struct PixelPerfectCameraFollow {
    /// A value like `4.` is a good amount
    pub follow_type: FollowType,
    /// Screen that should chase this entity. [`None`] means the only camera.
    pub camera: Option<Entity>,
}
#[allow(unused)]
impl PixelPerfectCameraFollow {

    pub fn new_instant()-> Self {
        Self {
            follow_type: FollowType::Instant,
            camera: None
        }
    }

    /// The larger the decay rate, the faster the camera will
    /// track the target. Default is 4.0
    ///
    /// If you are using splitscreen, use `for_camera` instead.
    pub fn new_smooth(decay_rate: f32) -> Self {
        Self {
            follow_type: FollowType::Smooth { decay_rate },
            camera: None,
        }
    }

    /// Follow `camera`.
    ///
    /// The larger the decay rate, the faster the camera will track the target.
    pub fn for_camera(camera: Entity, follow_type: FollowType) -> Self {
        Self {
            follow_type,
            camera: Some(camera),
        }
    }
}
impl Default for PixelPerfectCameraFollow {
    fn default() -> Self {
        Self::new_smooth(4.)
    }
}

pub struct PixelPerfectCameraFollowPlugin;

impl Plugin for PixelPerfectCameraFollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (super::snap::restore_subpixel_pos, handle_follow)
                .chain()
                .in_set(PixelPerfectCameraControlsSet::Follow),
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
            follow_type: follow.follow_type.clone(),
        });
    }

    let dt = time.delta_secs();
    for (entity, mut transform) in &mut cameras {
        let Some(choice) = chosen.iter().find(|choice| choice.camera == entity) else {
            continue;
        };
        let desired_pos = Vec3::new(choice.target.x, choice.target.y, transform.translation.z);

        match choice.follow_type {
            FollowType::Smooth { decay_rate } => {
                transform
                    .translation
                    .smooth_nudge(&desired_pos, decay_rate, dt);
            }
            FollowType::Instant => {
                transform.translation = desired_pos;
            }
        }
    }
}

struct ChosenFollow {
    camera: Entity,
    target: Vec3,
    follow_type: FollowType,
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
