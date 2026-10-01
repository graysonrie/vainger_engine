use crate::prelude::*;

/// Marker component that tells the pixel perfect camera to follow this entity
#[derive(Component)]
pub struct PixelPerfectCameraFollow {
    /// A value like `4.` is a good amount
    pub decay_rate: f32,
}
#[allow(unused)]
impl PixelPerfectCameraFollow {
    /// The `decay_rate` should be between 0-1.
    /// The larger the decay rate, the faster the camera will
    /// track the target
    pub fn new(decay_rate: f32) -> Self {
        Self { decay_rate }
    }
}
impl Default for PixelPerfectCameraFollow {
    fn default() -> Self {
        Self { decay_rate: 4. }
    }
}

pub struct PixelPerfectCameraFollowPlugin;

impl Plugin for PixelPerfectCameraFollowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, handle_follow.in_set(PixelPerfectCameraControlsSet::Follow));
    }
}

fn handle_follow(
    mut camera_transform: Single<
        &mut Transform,
        (With<PixelPerfectCamera>, Without<PixelPerfectCameraFollow>),
    >,
    follow: Single<(&Transform, &PixelPerfectCameraFollow), Without<PixelPerfectCamera>>,
    time: Res<Time>,
) {
    let (follow_transform, cam_follow) = *follow;

    let Vec3 { x, y, .. } = follow_transform.translation;
    let desired_pos = Vec3::new(x, y, camera_transform.translation.z);

    camera_transform.translation.smooth_nudge(
        &desired_pos,
        cam_follow.decay_rate,
        time.delta_secs(),
    );
}
