use crate::prelude::*;

/// Immediate-mode gizmo drawing in world pixels (Y-up).
///
/// Use it as a system parameter. Gizmos last one frame, so the system that
/// draws has to run every frame you want the shape visible.
#[derive(SystemParam)]
pub struct DebugDraw<'w, 's> {
    gizmos: Gizmos<'w, 's>,
}

impl DebugDraw<'_, '_> {
    /// White outline of `rect` in world pixels.
    ///
    /// `rect.min` is the bottom-left and `rect.max` is the top-right. The stroke
    /// is a gizmo line, not a sprite, and it is gone next frame unless you draw
    /// it again. Call this from a system that runs while the game is visible,
    /// typically [`Update`].
    pub fn draw_outline_rect(&mut self, rect: &Rect) {
        let min = rect.min;
        let max = rect.max;
        let center = (min + max) / 2.;
        let size = max - min;

        self.gizmos
            .rect_2d(Isometry2d::from_translation(center), size, Color::WHITE);
    }
}
//
