//! Moves an entity through world-space waypoints.
//!
//! Spawn [`Glide`] on the entity you want to move. The system writes
//! `translation.x` and `translation.y` every frame and leaves `z` alone.
//! [`super::MovementPlugin`] installs it.
//!
//! ### NOTE:
//! While this component is present, that system owns X and Y. It keeps
//! overwriting them even after [`GlideMode::Once`] has reached the last point.
//! Remove the component to take the transform back.

use super::*;

/// Ticks every [`Glide`] in [`super::MovementSet`] during [`Update`].
///
/// Installed by [`super::MovementPlugin`]. Adding this plugin yourself as well
/// runs the tick twice per frame.
pub struct MovementGlidePlugin;

impl Plugin for MovementGlidePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_glide.in_set(MovementSet));
    }
}

/// How a [`Glide`] advances after the last waypoint.
#[derive(Clone, Copy, Default)]
pub enum GlideMode {
    /// Travel to the last point and stay there. X and Y are still written every frame.
    Once,
    /// After the last point, travel back to the first. The first point's
    /// `duration` and `ease` are used for that wrapping segment.
    #[default]
    Loop,
    /// Reverse direction at either end. The destination point's `duration`
    /// and `ease` apply on the way back too, including the first point.
    PingPong,
}

/// Tweens [`Transform`] through [`points`](Self::points).
///
/// [`Glide::new`] starts in [`GlideMode::Loop`]. Chain [`Glide::once`] or
/// [`Glide::pingpong`] to change that. The entity needs a [`Transform`]; this
/// component does not insert one.
///
/// ### NOTE:
/// Fewer than two points does nothing. The entity stays where it is.
///
/// ### NOTE:
/// `duration` and `ease` live on the point being traveled **to**, not the one
/// being left. They are a travel time in seconds, not a speed, and they ignore
/// the distance between the points. Two points 10 pixels apart and two points
/// 200 pixels apart with `duration` 1 both take one second. The first point's
/// `duration` and `ease` are unused until a loop or ping-pong segment ends on
/// that point.
#[derive(Component, Default)]
pub struct Glide {
    /// Waypoints in order. The entity starts at index 0 and moves toward the next one.
    pub points: Vec<GlidePoint>,
    /// What happens at the ends. Default from [`Glide::new`] is [`GlideMode::Loop`].
    pub mode: GlideMode,

    current_idx: usize,
    /// Segment travel direction for [`GlideMode::PingPong`] (`1` forward, `-1` backward).
    direction: HorizontalDirection,
    elapsed: f32,
}
#[allow(unused)]
impl Glide {
    /// Looping glide through `points`. Chain [`Self::once`] or [`Self::pingpong`] to change the mode.
    ///
    /// ### NOTE:
    /// Pass at least two points. With fewer, the system leaves the transform where it is.
    pub fn new(points: impl IntoIterator<Item = GlidePoint>) -> Self {
        Self {
            points: points.into_iter().collect(),
            ..Default::default()
        }
    }
    /// Sets the [GlideMode] to `Once`
    pub fn once(mut self) -> Self {
        self.mode = GlideMode::Once;
        self
    }
    /// Sets the [GlideMode] to `PingPong`
    pub fn pingpong(mut self) -> Self {
        self.mode = GlideMode::PingPong;
        self
    }
    /// Sets the [GlideMode] to `Loop`
    pub fn looping(mut self) -> Self {
        self.mode = GlideMode::Loop;
        self
    }
}

/// One waypoint. See [`Glide`] for which point's `duration` and `ease` are read.
#[derive(Clone, Debug)]
pub struct GlidePoint {
    /// World position of this waypoint. Z is not stored; the entity's Z stays put.
    pub position: Vec2,
    /// Seconds to spend traveling to this point. `0` completes the segment on the same frame.
    ///
    /// This is not scaled by distance. See [`Glide`].
    pub duration: f32,
    /// Easing for the travel to this point. `None` is linear.
    pub ease: Option<EaseFunction>,
}
#[allow(unused)]
impl GlidePoint {
    /// Waypoint at `position`. Duration is 1 second and easing is linear.
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            duration: 1.,
            ease: None,
        }
    }

    /// Seconds to spend traveling to this point.
    ///
    /// ### NOTE:
    /// This does not account for the distance from the previous point.
    /// Points that are 100 units apart and points that are 200 units apart
    /// both take `duration` seconds. It is a time, not a speed.
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Easing curve for the travel to this point. Without this call the travel is linear.
    pub fn ease(mut self, ease: EaseFunction) -> Self {
        self.ease = Some(ease);
        self
    }
}

fn update_glide(time: Res<Time>, mut query: Query<(&mut Transform, &mut Glide)>) {
    for (mut transform, mut glide) in &mut query {
        let len = glide.points.len();
        if len < 2 {
            continue;
        }

        let last_idx = len - 1;

        if matches!(glide.mode, GlideMode::Once) && glide.current_idx >= last_idx {
            let position = glide.points[last_idx].position;
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            continue;
        }

        let glide_dir_value = horizontal_direction_as_value(&glide.direction);
        let (from_idx, to_idx) = match glide.mode {
            GlideMode::Loop => (glide.current_idx, (glide.current_idx + 1) % len),
            GlideMode::Once => (glide.current_idx, glide.current_idx + 1),
            GlideMode::PingPong => (
                glide.current_idx,
                (glide.current_idx as i32 + glide_dir_value) as usize,
            ),
        };

        let from_position = glide.points[from_idx].position;
        let to_position = glide.points[to_idx].position;
        let to_duration = glide.points[to_idx].duration;
        let to_ease = glide.points[to_idx].ease;

        glide.elapsed += time.delta_secs();

        let t = if to_duration > 0. {
            (glide.elapsed / to_duration).clamp(0., 1.)
        } else {
            1.
        };

        if let Some(ease) = to_ease {
            let curve = EasingCurve::new(from_position, to_position, ease);
            let position = curve.sample_clamped(t);

            transform.translation.x = position.x;
            transform.translation.y = position.y;
        } else {
            // Linear ease
            let position = from_position.lerp(to_position, t);

            transform.translation.x = position.x;
            transform.translation.y = position.y;
        }

        if glide.elapsed >= to_duration {
            glide.elapsed -= to_duration;
            glide.current_idx = to_idx;

            if matches!(glide.mode, GlideMode::PingPong) {
                if to_idx == 0 {
                    glide.direction = HorizontalDirection::Right;
                } else if to_idx == last_idx {
                    glide.direction = HorizontalDirection::Left;
                }
            }
        }
    }
}
