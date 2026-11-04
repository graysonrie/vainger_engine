//! Pixel move-and-collide, plus a separate contact probe.
//!
//! [`move_and_collide`] slides the transform on X, then on Y.
//! [`get_collision_contacts`] only reports which sides are touching; nothing
//! in this module stores that for you.
//!
//! ```ignore
//! move_and_collide(&mut transform, &hitbox, &mut velocity, |pos, hitbox| {
//!     grid.overlaps_hitbox(hitbox, pos)
//! });
//! *contacts = get_collision_contacts(transform.get_2d_pos(), &hitbox, |pos, hitbox| {
//!     grid.overlaps_hitbox(hitbox, pos)
//! });
//! ```

use crate::{components::Velocity2D, prelude::*};

/// Slides `transform` by `velocity`, resolving X fully before Y.
///
/// `collision_check_fn` receives a candidate world position and the same
/// `hitbox`, and returns `true` when that placement overlaps something solid.
/// `hitbox`'s anchor is what sits on the position. Z on `transform` is left
/// alone.
///
/// On an axis that would end inside a solid, the move steps one world unit at
/// a time until the next unit is blocked, then that axis of `velocity` is set
/// to `0`. An axis whose destination is free moves the full displacement and
/// keeps its velocity.
///
/// ### NOTE:
/// `velocity` is a displacement in pixels for this call, not units per second.
/// Scale a per-second speed by `time.delta_secs()` first. See [`Velocity2D`].
///
/// ### NOTE:
/// Only the destination is tested, then (if that is solid) the cells along the
/// remaining step. A move that starts and ends in open space but skips across
/// a thin wall will pass through it. Keep displacements smaller than the
/// thinnest solid you need to catch.
///
/// ### NOTE:
/// The correction step is one world unit, which is one pixel at the default
/// game scale. Sub-pixel positions can stop a pixel early. Already overlapping
/// a solid does not push the entity out: velocity on that axis becomes `0`
/// and the entity stays where it was.
///
/// ### NOTE:
/// This does not write [`CollisionContacts`]. Probe after the move with
/// [`get_collision_contacts`]. X is solved before Y, so a corner can cancel
/// horizontal velocity and still leave the vertical move free.
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
/// Resolves the X axis of `velocity`. Returns the resulting position; the caller
/// assigns it. See [`move_and_collide`] for the step rules.
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
/// Resolves the Y axis of `velocity` after X. Returns the resulting position.
/// See [`move_and_collide`] for the step rules.
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
/// Probes one pixel left, right, down, and up from `pos` and reports which sides hit.
///
/// `pos` is the hitbox anchor's world position (the same value
/// [`move_and_collide`] writes to the transform). `collision_check_fn` is the
/// same solid test you passed to the move: `true` means the nudged placement
/// overlaps a solid.
///
/// Down (`pos.y - 1`) is [`CollisionContacts::floor`]. Up is
/// [`CollisionContacts::ceiling`]. This matches Bevy's Y-up axis.
///
/// ### NOTE:
/// Nothing calls this for you. [`move_and_collide`] will not fill a
/// [`CollisionContacts`] component. Run this after moving, in your own system,
/// and store the returned value.
///
/// ### NOTE:
/// The probe is exactly one world unit. A solid that only touches on a
/// sub-pixel overlap, or a gap smaller than a pixel, will not match the side
/// you expect. Call it on the post-move position; probing the pre-move
/// position reports last frame's contacts.
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
    /// Solid one pixel below the hitbox (`-Y`).
    pub floor: bool,
    /// Solid one pixel above the hitbox (`+Y`).
    pub ceiling: bool,
    /// Solid one pixel to the left (`-X`).
    pub left: bool,
    /// Solid one pixel to the right (`+X`).
    pub right: bool,
}
