use crate::collision::CollisionContacts;

use super::*;

pub struct CollisionJumpPlugin;

impl Plugin for CollisionJumpPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_do_jump);
        app.add_observer(on_jump_succeeded);
        app.add_systems(
            Update,
            (
                handle_jump_from_buffer,
                handle_jump_cut,
                handle_gravity,
                advance_jump_buffer,
            )
                .chain()
                .in_set(Collision2DSet::Jump),
        );
    }
}

/// If present, this entity will have the ability to jump
///
/// Defaults to `12.0`
#[derive(Component)]
pub struct Jump {
    height: f32,
}
impl Default for Jump {
    fn default() -> Self {
        Self { height: 12. }
    }
}
impl Jump {
    pub fn new(height: f32) -> Self {
        Self { height }
    }
}

/// Tells this entity to perform a jump.
///
/// Note that this does **not** mean the entity *will jump*,
/// since it will first check to see if the entity is grounded
#[derive(EntityEvent, Clone, Debug)]
pub struct DoJump(pub Entity);

#[derive(Component)]
#[require(Jump)]
pub struct JumpBuffer {
    // Value in frames
    pressed: u8,
    // Value in frames
    released: u8,
    pressed_max: u8,
    released_max: u8,
}

impl JumpBuffer {
    /// Records a new jump press and clears any previous release.
    ///
    /// A same-frame tap should call [`Self::set_released`] after this.
    pub fn set_pressed(&mut self) {
        self.pressed = self.pressed_max;
        self.released = 0;
    }

    /// Sets `released` to the max value
    pub fn set_released(&mut self) {
        self.released = self.released_max;
    }
    pub fn pressed_active(&self) -> bool {
        self.pressed > 0
    }
    pub fn released_active(&self) -> bool {
        self.released > 0
    }
    fn advance(&mut self) {
        let was_pressed = self.pressed > 0;
        self.pressed = self.pressed.saturating_sub(1);
        if was_pressed && self.pressed == 0 {
            // Press buffer ended without a jump (a successful jump clears
            // `pressed` before this runs). Drop the early release with it.
            self.released = 0;
        } else if self.pressed == 0 {
            self.released = self.released.saturating_sub(1);
        }
    }
    pub fn zero_released(&mut self) {
        self.released = 0;
    }
}
impl Default for JumpBuffer {
    fn default() -> Self {
        Self {
            pressed: 0,
            released: 0,
            pressed_max: 8,
            released_max: 4,
        }
    }
}

#[derive(Component)]
#[require(Gravity)]
pub struct CoyoteTime {
    max_frames: u32,
}
impl CoyoteTime {
    pub fn is_active(&self, falling: u32) -> bool {
        falling < self.max_frames
    }
}
impl Default for CoyoteTime {
    fn default() -> Self {
        Self { max_frames: 4 }
    }
}

/// The character will be affected by gravity
///
/// Note: if you are wanting to check if the character is grounded, prefer using
/// [`TerrainContacts`] instead as it is more reliable than checking `falling`
#[derive(Component)]
pub struct Gravity {
    pub scale: f32,
    pub falling: u32,
}
impl Default for Gravity {
    fn default() -> Self {
        Self {
            scale: 1.,
            falling: 0,
        }
    }
}

fn handle_jump_from_buffer(
    mut q: Query<(Entity, &mut JumpBuffer), (With<CollisionActor>, With<Jump>)>,
    mut commands: Commands,
) {
    for (entity, jump_buffer) in &mut q {
        if jump_buffer.pressed_active() {
            commands.trigger(DoJump(entity));
        }
    }
}

fn on_jump_succeeded(event: On<JumpSucceeded>, mut q: Query<&mut JumpBuffer>) {
    if let Ok(mut buffer) = q.get_mut(event.0) {
        buffer.pressed = 0;
    }
}

/// Emitted when a [`DoJump`] event succeeds (the entity performs a jump)
#[derive(EntityEvent)]
pub struct JumpSucceeded(pub Entity);

fn on_do_jump(
    event: On<DoJump>,
    mut q: Query<
        (
            &Jump,
            Option<&CoyoteTime>,
            Option<&JumpCut>,
            &mut JumpBuffer,
            &mut Velocity2D,
            &CollisionContacts,
            &Gravity,
        ),
        With<CollisionActor>,
    >,
    mut commands: Commands,
) {
    if let Ok((jump, coyote, jump_cut, mut jump_buffer, mut velocity, contacts, gravity)) =
        q.get_mut(event.0)
    {
        let can_jump = contacts.floor || coyote.map_or_default(|c| c.is_active(gravity.falling));
        if can_jump {
            commands.trigger(JumpSucceeded(event.0));
            velocity.y = jump.height;
            // The press can be several frames old. If the button was already
            // released, start at the cut height instead of full height.
            if jump_buffer.released_active()
                && let Some(jump_cut) = jump_cut
            {
                velocity.y *= jump_cut.amount;
                jump_buffer.released = 0;
            }
        }
    }
}

fn handle_gravity(
    mut q: Query<(&mut Velocity2D, &mut Gravity, &CollisionContacts), With<CollisionActor>>,
    grav: Res<Collision2DGravityForce>,
) {
    for (mut velocity, mut gravity, contacts) in &mut q {
        velocity.y += gravity.scale * grav.0;
        if contacts.floor {
            gravity.falling = 0;
        } else {
            gravity.falling += 1;
        }
    }
}

fn advance_jump_buffer(mut q: Query<&mut JumpBuffer>) {
    for mut buffer in &mut q {
        buffer.advance();
    }
}

/// Will cut the players jump by multiplying the Y velocity by `amount` if the
/// release buffer in [`JumpBuffer`] is on
#[derive(Component)]
#[require(JumpBuffer, Velocity2D)]
pub struct JumpCut {
    amount: f32,
}
impl JumpCut {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}
impl Default for JumpCut {
    fn default() -> Self {
        Self { amount: 0.5 }
    }
}

fn handle_jump_cut(
    mut q: Query<(&mut Velocity2D, &mut JumpBuffer, &JumpCut), With<CollisionActor>>,
) {
    for (mut velocity, mut jump_buffer, jump_cut) in &mut q {
        // `0.0` is sign-positive. Landing zeroes vertical speed, and treating
        // that as "rising" would consume the release before the buffered jump
        // assigns its upward velocity.
        if velocity.y > 0.0 && jump_buffer.released_active() {
            velocity.y *= jump_cut.amount;
            jump_buffer.released = 0;
        }
    }
}
