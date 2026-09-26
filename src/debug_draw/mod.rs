use crate::prelude::*;

/// Wraps gizmos so you can draw
#[derive(SystemParam)]
pub struct DebugDraw<'w, 's> {
    gizmos: Gizmos<'w, 's>,
}

impl DebugDraw<'_, '_> {
    /// Spawns a 1px rectangle with a white border.
    /// This is in immediate-mode, so you must have a system
    /// running in `Update` to draw this each frame
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
