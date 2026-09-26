use crate::{pixel_perfect_cam::follow::PixelPerfectCameraFollow, prelude::*};

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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect, Actionlike)]
#[actionlike(DualAxis)]
pub enum CameraPanner2dAction {
    Move,
}
fn default_input_map() -> InputMap<CameraPanner2dAction> {
    use CameraPanner2dAction::*;

    let mut input_map = InputMap::default();
    input_map.insert_dual_axis(Move, GamepadStick::LEFT);
    input_map.insert_dual_axis(Move, VirtualDPad::wasd());

    input_map
}

pub struct CameraPanner2dPlugin;

impl Plugin for CameraPanner2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<CameraPanner2dAction>::default())
            .add_systems(Update, handle_pan.in_set(PixelPerfectCameraControlsSet));
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
