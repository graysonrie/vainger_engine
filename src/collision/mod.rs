use crate::{components::Velocity2D, prelude::*};

/// Performs a full 2D move and collide pass with both X and Y
pub fn move_and_collide(
    transform: &mut Transform,
    hitbox: &Hitbox,
    velocity: &mut Velocity2D,
    collision_check_fn: impl Fn(Vec2, &Hitbox) -> bool,
) {
    let z = transform.translation.z;
    let xout = move_and_collide_x(
        transform.get_2d_pos(),
        hitbox,
        velocity,
        &collision_check_fn,
    );

    let yout = move_and_collide_y(xout, hitbox, velocity, &collision_check_fn);

    transform.translation = yout.extend(z);
}

#[must_use]
/// Returns the output position. Assign that to the transform.translation, or whatever
/// you are manipulating
fn move_and_collide_x(
    mut pos: Vec2,
    hitbox: &Hitbox,
    velocity: &mut Velocity2D,
    collision_check_fn: &impl Fn(Vec2, &Hitbox) -> bool,
) -> Vec2 {
    if velocity.x != 0. && collision_check_fn(pos.add_to_x(velocity.x), hitbox) {
        while !collision_check_fn(pos.add_to_x(velocity.x.signum()), hitbox) {
            pos.x += velocity.x.signum();
        }
        velocity.x = 0.;
    }
    pos.x += velocity.x;
    pos
}
#[must_use]
/// Returns the output position. Assign that to the transform.translation, or whatever
/// you are manipulating
fn move_and_collide_y(
    mut pos: Vec2,
    hitbox: &Hitbox,
    velocity: &mut Velocity2D,
    collision_check_fn: &impl Fn(Vec2, &Hitbox) -> bool,
) -> Vec2 {
    if velocity.y != 0. && collision_check_fn(pos.add_to_y(velocity.y), hitbox) {
        while !collision_check_fn(pos.add_to_y(velocity.y.signum()), hitbox) {
            pos.y += velocity.y.signum();
        }
        velocity.y = 0.;
    }
    pos.y += velocity.y;

    pos
}

#[must_use = "You should assign the contacts to something"]
pub fn get_collision_contacts(
    pos: Vec2,
    hitbox: &Hitbox,
    collision_check_fn: impl Fn(Vec2, &Hitbox) -> bool,
) -> CollisionContacts {
    let left = collision_check_fn(pos.add_to_x(-1.), hitbox);
    let right = collision_check_fn(pos.add_to_x(1.), hitbox);

    let floor = collision_check_fn(pos.add_to_y(-1.), hitbox);
    let ceiling = collision_check_fn(pos.add_to_y(1.), hitbox);

    CollisionContacts {
        left,
        right,
        floor,
        ceiling,
    }
}

/// Describes in what directions the entity is touching the collision
///
/// ### NOTE:
/// You are responsible for making a system to determine the
/// contacts, but all you have to do is make a simple system to
/// call [`get_collision_contacts`]
#[derive(Component, Default)]
#[require(Transform, Hitbox)]
pub struct CollisionContacts {
    pub floor: bool,
    pub ceiling: bool,
    pub left: bool,
    pub right: bool,
}
