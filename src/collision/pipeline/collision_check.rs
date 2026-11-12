use crate::{ldtk::int_grid_cells::IntGridCells, prelude::*};

#[derive(SystemParam)]
pub struct LdtkCollisionCheck<'w, 's> {
    level: Single<'w, 's, &'static IntGridCells>,
}

impl LdtkCollisionCheck<'_, '_> {
    pub fn is_world_loaded(&self) -> bool {
        self.level.cells.column_len() > 0
    }

    pub fn is_colliding(&self, pos: Vec2, hitbox: &Hitbox) -> bool {
        self.level.overlaps_hitbox(hitbox, pos)
    }
}
