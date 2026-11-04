//! Smooth follow for [`PixelPerfectCamera`].

use crate::prelude::*;

/// Makes [`PixelPerfectCamera`] chase this entity's `x` and `y`.
///
/// The camera keeps its own Z. Put this on a gameplay entity, not on the camera.
/// A component on the camera itself is ignored by the follow query.
///
/// ### NOTE:
/// The follow system uses a single-entity query. Zero followers, or more than
/// one, and the system errors and does not move the camera. The plugin is
/// always installed by [`PixelPerfectCameraPlugin`], so the error is logged
/// every frame until exactly one entity has this component.
/// [`CameraPanner2dBundle`] includes one. Do not spawn a second follower
/// alongside the panner.
#[derive(Component)]
pub struct PixelPerfectCameraFollow {
    /// A value like `4.` is a good amount
    pub decay_rate: f32,
}
#[allow(unused)]
impl PixelPerfectCameraFollow {
    /// The larger the decay rate, the faster the camera will
    /// track the target.
    ///
    /// Default is 4.0
    pub fn new(decay_rate: f32) -> Self {
        Self { decay_rate }
    }
}
impl Default for PixelPerfectCameraFollow {
    fn default() -> Self {
        Self { decay_rate: 4. }
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
