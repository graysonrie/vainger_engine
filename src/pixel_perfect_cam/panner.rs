//! An entity moved with WASD or the left stick. The pixel-perfect camera follows it.

use crate::{pixel_perfect_cam::follow::PixelPerfectCameraFollow, prelude::*};

/// Entity the pixel-perfect camera follows, driven by WASD or the left stick.
///
/// `speed` in [`Self::new`] is pixels per second. The bundle inserts
/// [`PixelPerfectCameraFollow`] with no camera target, so it drives the only
/// screen. With more than one [`PixelPerfectCamera`], set
/// [`PixelPerfectCameraFollow::camera`] or the panner will not move a camera.
///
/// ### NOTE:
/// The bundle includes a visible 8×8 black [`Sprite`] at the entity origin.
/// That sprite is on the default render layer, so it shows up in the game view.
#[allow(unused)]
#[derive(Bundle)]
pub struct CameraPanner2dBundle {
    camera_panner: CameraPanner2d,
    actions: InputMap<CameraPanner2dAction>,
    transform: Transform,
    follow: PixelPerfectCameraFollow,
    sprite: Sprite,
}
#[allow(unused)]
impl CameraPanner2dBundle {
    /// Panner that moves at `speed` pixels per second.
    ///
    /// WASD and the left gamepad stick both drive `CameraPanner2dAction::Move`.
    /// The bundle's [`InputMap`] requires an [`ActionState`], and Bevy inserts
    /// that automatically. The input plugin is already registered by
    /// [`PixelPerfectCameraPlugin`].
    pub fn new(speed: f32) -> Self {
        Self {
            camera_panner: CameraPanner2d { speed },
            actions: default_input_map(),
            transform: Transform::from_translation(Vec3::ZERO),
            follow: PixelPerfectCameraFollow::default(),
            sprite: Sprite::from_color(Color::BLACK, Vec2::splat(8.)),
        }
    }
}

#[derive(Component)]
struct CameraPanner2d {
    speed: f32,
}

/// Actions understood by [`CameraPanner2dBundle`].
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Actionlike)]
#[actionlike(DualAxis)]
pub enum CameraPanner2dAction {
    /// Dual-axis pan. Bound to WASD and the left stick by [`CameraPanner2dBundle::new`].
    Move,
}
fn default_input_map() -> InputMap<CameraPanner2dAction> {
    use CameraPanner2dAction::*;

    let mut input_map = InputMap::default();
    input_map.insert_dual_axis(Move, GamepadStick::LEFT);
    input_map.insert_dual_axis(Move, VirtualDPad::wasd());

    input_map
}

/// Registers leafwing input for [`CameraPanner2dAction`] and the pan system.
///
/// Added by [`PixelPerfectCameraPlugin`]. Spawning [`CameraPanner2dBundle`] is
/// enough; adding this plugin a second time registers the pan system twice,
/// so the entity moves at double speed.
pub struct CameraPanner2dPlugin;

impl Plugin for CameraPanner2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraPanner2dAction>::default())
            .add_systems(Update, handle_pan);
    }
}

fn handle_pan(
    mut query: Query<(
        &ActionState<CameraPanner2dAction>,
        &mut Transform,
        &CameraPanner2d,
    )>,
    time: Res<Time>,
) {
    for (action, mut transform, camera_panner) in query.iter_mut() {
        let dir = action.clamped_axis_pair(&CameraPanner2dAction::Move);
        transform.translation += dir.extend(0.) * camera_panner.speed * time.delta_secs();
    }
}
