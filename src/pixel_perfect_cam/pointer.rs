use crate::prelude::*;
use bevy::{ecs::system::SystemParam, window::PrimaryWindow};

/// Can get the cursor world position, respecting the pixel perfect cameras.
///
/// The cursor is tested against each screen's canvas. The hit uses that
/// camera's position and [`CameraZoom`]. A letterbox bar or an empty grid cell
/// is not a hit.
///
/// Works for both desktop and mobile targets.
#[derive(SystemParam)]
pub struct GamePointer<'w, 's> {
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    outer: Single<'w, 's, (&'static Camera, &'static GlobalTransform), With<OuterCamera>>,
    cameras: Query<
        'w,
        's,
        (
            &'static GlobalTransform,
            &'static ScreenSlot,
            &'static CameraZoom,
        ),
        With<PixelPerfectCamera>,
    >,
    canvases: Query<'w, 's, (&'static Transform, &'static Sprite), With<GameCanvas>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    touches: Res<'w, Touches>,
}

#[allow(unused)]
impl GamePointer<'_, '_> {
    /// Logical screen position of the pointer, if it is currently over the window.
    ///
    /// Desktop: [`Window::cursor_position`] (None after `CursorLeft` or when dragged
    /// outside while the button is still held). Touch is only used when the mouse is
    /// not captured outside the client area, so laptop trackpads cannot keep a stale
    /// "inside" hit after the cursor has left.
    fn screen_position(&self) -> Option<Vec2> {
        if let Some(pos) = self.window.cursor_position() {
            return Some(pos);
        }
        // Windows captures the mouse while dragging; the button stays pressed but
        // the cursor is no longer in the client area.
        if self.mouse.pressed(MouseButton::Left) {
            return None;
        }
        self.touches.first_pressed_position()
    }

    /// Cursor or finger position in world pixels (Y-up), on the screen under the pointer.
    ///
    /// [`None`] when the pointer is outside the OS window, over the letterbox,
    /// or over an empty grid cell. On Windows, dragging with the left button
    /// held captures the mouse; once the cursor leaves the client area this
    /// returns [`None`] even though the button is still down.
    ///
    /// A point on the shared edge of two cells belongs to the earlier screen
    /// (the one higher in the grid, then the one further left).
    ///
    /// ### NOTE:
    /// [`Self::is_inside_window`] does not mean "inside a game view".
    pub fn position(&self) -> Option<Vec2> {
        let cursor = self.screen_position()?;
        let (cam, transform) = *self.outer;
        let on_canvas = cam.viewport_to_world_2d(transform, cursor).ok()?;

        let mut best: Option<PointerHit> = None;
        for (camera_transform, slot, zoom) in &self.cameras {
            let Ok((sprite_transform, sprite)) = self.canvases.get(slot.canvas) else {
                continue;
            };
            let half = sprite.custom_size.unwrap_or(Vec2::ZERO) * 0.5;
            let local = on_canvas - sprite_transform.translation.truncate();
            let inside =
                local.x >= -half.x && local.x <= half.x && local.y >= -half.y && local.y <= half.y;
            if !inside {
                continue;
            }
            let world = camera_transform.translation().truncate()
                + local / super::layout::effective_zoom(zoom.0);
            let hit = PointerHit {
                cell_min: slot.rect.min,
                world,
            };
            let closer = best.as_ref().is_none_or(|best| hit.is_before(best));
            if closer {
                best = Some(hit);
            }
        }
        best.map(|hit| hit.world)
    }

    /// Whether the mouse is over the window, or a finger is down.
    ///
    /// Independent of world-space conversion, which can fail while the pointer is
    /// still inside the window (e.g. camera target size not ready yet).
    pub fn is_inside_window(&self) -> bool {
        self.screen_position().is_some()
    }

    /// Left mouse button or a touch went down this frame.
    ///
    /// Right and middle buttons are ignored. A touch counts on any finger.
    pub fn just_pressed(&self) -> bool {
        self.mouse.just_pressed(MouseButton::Left) || self.touches.any_just_pressed()
    }

    /// Left mouse button is held, or at least one finger is on the screen.
    ///
    /// This is not the same as "the pointer is over the game canvas". A held
    /// button still reports pressed after the cursor has left the window.
    /// Use [`Self::is_inside_window`] for that.
    pub fn pressed(&self) -> bool {
        self.mouse.pressed(MouseButton::Left)
            || self.touches.any_just_pressed()
            || self.touches.iter().next().is_some()
    }
}

struct PointerHit {
    cell_min: UVec2,
    world: Vec2,
}

impl PointerHit {
    fn is_before(&self, other: &Self) -> bool {
        self.cell_min.y < other.cell_min.y
            || (self.cell_min.y == other.cell_min.y && self.cell_min.x < other.cell_min.x)
    }
}
