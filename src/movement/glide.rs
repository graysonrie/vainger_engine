use super::*;

pub struct MovementGlidePlugin;

impl Plugin for MovementGlidePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_glide.in_set(MovementSet));
    }
}

#[derive(Clone, Copy, Default)]
pub enum GlideMode {
    Once,
    #[default]
    Loop,
    PingPong,
}

#[derive(Component, Default)]
pub struct Glide {
    pub points: Vec<GlidePoint>,
    pub mode: GlideMode,

    current_idx: usize,
    /// Segment travel direction for [`GlideMode::PingPong`] (`1` forward, `-1` backward).
    direction: HorizontalDirection,
    elapsed: f32,
}
#[allow(unused)]
impl Glide {
    /// Constructs a new looping glide
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

#[derive(Clone, Debug)]
pub struct GlidePoint {
    pub position: Vec2,
    /// How long it takes to travel to this point
    pub duration: f32,
    /// How the interpolation behaves while traveling to this point
    pub ease: Option<EaseFunction>,
}
#[allow(unused)]
impl GlidePoint {
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            duration: 1.,
            ease: None,
        }
    }

    /// Marks how many seconds it will take to reach this point.
    ///
    /// Note that this does not account for the distance between the previous and next point.
    /// For example two points that are 100 units apart and two points that are 200 units apart will both take 1 second to reach
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

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
