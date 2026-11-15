use crate::prelude::*;

/// Shared solid collision grid used by the 2D collision pipeline.
///
/// Map backends (LDtk, Tiled, …) write into this resource; movement systems
/// only ever query it via [`SolidWorld::is_colliding`].
#[derive(Resource)]
pub struct SolidWorld {
    width: i32,
    height: i32,
    cell_size: i32,
    cells: Array2D<u8>,
}

impl Default for SolidWorld {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cell_size: 0,
            cells: Array2D::filled_by_column_major(|| 0, 0, 0),
        }
    }
}

impl SolidWorld {
    /// Whether a collision grid has been loaded.
    #[must_use]
    pub fn is_loaded(&self) -> bool {
        self.cell_size > 0 && self.cells.column_len() > 0
    }

    /// True if `hitbox` at `pos` overlaps any solid cell.
    #[must_use]
    pub fn is_colliding(&self, pos: Vec2, hitbox: &Hitbox) -> bool {
        if self.cell_size <= 0 {
            return false;
        }

        let MinMax { min, max } = hitbox
            .min_max_positions_in_world(pos)
            .shrink_by_val(Vec2::splat(0.001));

        let cell = self.cell_size as f32;
        let x0 = (min.x / cell).floor() as i32;
        let x1 = (max.x / cell).floor() as i32;
        let y0 = (min.y / cell).floor() as i32;
        let y1 = (max.y / cell).floor() as i32;

        for y in y0..=y1 {
            for x in x0..=x1 {
                if (0..self.width).contains(&x)
                    && (0..self.height).contains(&y)
                    && self.cells[(y as usize, x as usize)] != 0
                {
                    return true;
                }
            }
        }
        false
    }

    /// Replaces the solid grid with the given cell data.
    pub fn load_grid(&mut self, width: i32, height: i32, cell_size: i32, cells: Array2D<u8>) {
        self.width = width;
        self.height = height;
        self.cell_size = cell_size;
        self.cells = cells;
    }

    /// Clears solid data (e.g. on level unload).
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
