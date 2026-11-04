use crate::prelude::*;
use bevy::{ecs::system::SystemParam, window::PrimaryWindow};

/// Can get the cursor world position, respecting the pixel perfect camera
///
/// Works for both desktop and mobile targets
#[derive(SystemParam)]
pub struct GamePointer<'w, 's> {
    window: Single<'w, 's, &'static Window, With<PrimaryWindow>>,
    outer: Single<'w, 's, (&'static Camera, &'static GlobalTransform), With<OuterCamera>>,
    inner: Single<'w, 's, &'static GlobalTransform, With<PixelPerfectCamera>>,
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

    /// Cursor or finger position in game-pixel world space (the inner camera, Y-up).
    ///
    /// [`None`] when the pointer is outside the OS window. On Windows, dragging
    /// with the left button held captures the mouse; once the cursor leaves the
    /// client area this returns [`None`] even though the button is still down.
    ///
    /// ### NOTE:
    /// Letterbox bars are still inside the window, so a cursor there returns a
    /// world position that can fall outside the canvas. [`Self::is_inside_window`]
    /// does not mean "inside the game view".
    pub fn position(&self) -> Option<Vec2> {
        let cursor = self.screen_position()?;
        let (cam, transform) = *self.outer;
        let on_canvas = cam.viewport_to_world_2d(transform, cursor).ok()?;
        Some(self.inner.translation().truncate() + on_canvas)
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
