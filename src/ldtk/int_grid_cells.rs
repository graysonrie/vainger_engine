//! A copy of one LDtk IntGrid, kept on the world entity for collision checks.
//!
//! [`crate::app::AppExt::setup_for_default_2d_pixel_game`] installs the system
//! that fills [`IntGridCells`] when a level spawns.

use crate::prelude::*;

pub(crate) struct IntGridCellsPlugin;

impl Plugin for IntGridCellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, populate_int_grid_cells);
    }
}

/// Copy of one LDtk IntGrid layer from the level that most recently spawned.
///
/// Put this on the same entity as [`LdtkWorldBundle`]. [`Self::new`] names the
/// layer. A system installed by
/// [`crate::app::AppExt::setup_for_default_2d_pixel_game`] replaces the grid
/// when a level spawns.
///
/// Cell `(x, y)` is the bottom-left cell, matching `bevy_ecs_ldtk` grid coords.
/// Index the array as `cells[(y, x)]` (row, then column). `0` is empty. Any
/// other stored value is that cell's IntGrid id.
///
/// Pair it with [`crate::collision::move_and_collide`] by passing
/// [`Self::overlaps_hitbox`] as the solid test.
///
/// ### NOTE:
/// [`Self::overlaps_hitbox`] and [`Self::is_occupied`] measure world pixels
/// from the level's bottom-left. The level's world offset is not subtracted.
/// That corner has to sit at the world origin, or you pass positions relative
/// to it.
///
/// ### NOTE:
/// The level must embed its layer instances in the project. External `.ldtkl`
/// files have no layer data, the fill fails, and this component stays at
/// whatever it already held (empty, if you just spawned [`Self::new`]).
///
/// ### NOTE:
/// IntGrid values have to fit in a `u8` (`0..=255`). A larger value fails the
/// fill and leaves the previous grid in place. A wrong [`Self::layer_name`]
/// does the same, and logs a warning.
///
/// ### NOTE:
/// Every [`IntGridCells`] in the world is rewritten from each spawned level.
/// The last spawn wins. Out-of-range positions count as empty: the void
/// outside the grid does not collide.
#[derive(Component)]
pub struct IntGridCells {
    /// Grid width in cells.
    pub width: i32,
    /// Grid height in cells.
    pub height: i32,
    /// Width and height of one cell, in pixels.
    pub cell_size: i32,
    /// Cell values, indexed `(row, column)` = `(y, x)`. `0` is empty.
    pub cells: Array2D<u8>,
    /// LDtk layer identifier to copy. This is the name you set in LDtk, not the layer type.
    pub layer_name: &'static str,
}

impl IntGridCells {
    /// Empty grid that will be filled from the IntGrid layer named `layer_name`.
    ///
    /// `layer_name` is compared to the layer's identifier string in the project.
    pub fn new(layer_name: &'static str) -> Self {
        Self {
            width: 0,
            height: 0,
            cell_size: 0,
            cells: Array2D::filled_by_column_major(|| 0, 0, 0),
            layer_name,
        }
    }
}

impl IntGridCells {
    /// `true` when any non-zero cell overlaps `hitbox` placed so its anchor is at `world_pos`.
    ///
    /// `world_pos` is the same anchor position [`Hitbox::min_max_positions_in_world`]
    /// expects. A `cell_size` of `0` or less returns `false`.
    ///
    /// ### NOTE:
    /// Only the value matters here: every non-zero id is solid. Read [`Self::cells`]
    /// yourself when different ids should collide differently. Cells the box
    /// merely touches on the outside of an edge can still count; the test
    /// nudges the bounds by a thousandth of a pixel before choosing cells.
    pub fn overlaps_hitbox(&self, hitbox: &Hitbox, world_pos: Vec2) -> bool {
        if self.cell_size <= 0 {
            return false;
        }

        let MinMax { min, max } = hitbox
            .min_max_positions_in_world(world_pos)
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

    /// `true` when the cell under the world point `world_pos` is non-zero.
    ///
    /// Points outside the grid return `false`. This tests a single point, not a hitbox.
    pub fn is_occupied(&self, world_pos: Vec2) -> bool {
        let Some(cell) = self.cell_at(world_pos) else {
            return false;
        };

        self.cells[(cell.0 as usize, cell.1 as usize)] != 0
    }

    /// Returns the x,y pos of the LDtk cell that this world
    /// position hits
    fn cell_at(&self, world: Vec2) -> Option<(i32, i32)> {
        let x = (world.x / self.cell_size as f32).floor() as i32;
        let y = (world.y / self.cell_size as f32).floor() as i32;
        (x >= 0 && y >= 0 && x < self.width && y < self.height).then_some((x, y))
    }

    /// Builds a grid from the level's `collision_layer_name` IntGrid.
    ///
    /// Cell `(x, y)` is the bottom-left cell of the level, matching `bevy_ecs_ldtk` grid coords.
    fn from_level(
        level: &Level,
        collision_layer_name: &'static str,
    ) -> Result<Self, anyhow::Error> {
        let layers = level
            .layer_instances
            .as_ref()
            .ok_or_else(|| anyhow!("level '{}' has no embedded layer data", level.identifier))?;
        let layer = layers
            .iter()
            .find(|layer| layer.identifier == collision_layer_name)
            .ok_or_else(|| {
                anyhow!(
                    "level '{}' has no {collision_layer_name} IntGrid layer",
                    level.identifier
                )
            })?;

        let width = layer.c_wid;
        let height = layer.c_hei;
        let cell_size = layer.grid_size;
        if width < 0 || height < 0 || cell_size <= 0 {
            return Err(anyhow!(
                "level '{}' has an invalid collision grid ({width}x{height}, cell {cell_size})",
                level.identifier
            ));
        }

        let width_us = width as usize;
        let height_us = height as usize;
        let expected = width_us.checked_mul(height_us).ok_or_else(|| {
            anyhow!(
                "level '{}' collision grid is too large to index",
                level.identifier
            )
        })?;
        if layer.int_grid_csv.len() != expected {
            return Err(anyhow!(
                "level '{}' collision csv has {} cells, expected {expected}",
                level.identifier,
                layer.int_grid_csv.len()
            ));
        }

        let mut cells = Array2D::filled_with(0, height_us, width_us);
        for (index, value) in layer.int_grid_csv.iter().copied().enumerate() {
            if value == 0 {
                continue;
            }
            let stored = u8::try_from(value).map_err(|_| {
                anyhow!(
                    "level '{}' collision value {value} does not fit in u8",
                    level.identifier
                )
            })?;
            let coords = int_grid_index_to_grid_coords(index, width as u32, height as u32)
                .ok_or_else(|| {
                    anyhow!(
                        "level '{}' collision index {index} is outside the grid",
                        level.identifier
                    )
                })?;
            let x = usize::try_from(coords.x)?;
            let y = usize::try_from(coords.y)?;
            cells.set(y, x, stored).map_err(|err| {
                anyhow!(
                    "level '{}' collision cell ({x}, {y}): {err}",
                    level.identifier
                )
            })?;
        }

        Ok(Self {
            width,
            height,
            cell_size,
            cells,
            layer_name: collision_layer_name,
        })
    }
}

/// Copies the spawned level's Collision IntGrid onto the drive world's [`IntGridCells`].
fn populate_int_grid_cells(
    mut spawned: MessageReader<LevelEvent>,
    mut grids: Query<(&LdtkProjectHandle, &mut IntGridCells)>,
    projects: Res<Assets<LdtkProject>>,
) {
    for event in spawned.read() {
        let LevelEvent::Spawned(level_iid) = event else {
            continue;
        };

        for (handle, mut grid) in &mut grids {
            let Some(project) = projects.get(handle.id()) else {
                warn!("drive level spawned before its LDtk project was loaded");
                continue;
            };
            let Some(level) = project.get_raw_level_by_iid(level_iid.get()) else {
                continue;
            };

            match IntGridCells::from_level(level, grid.layer_name) {
                Ok(populated) => *grid = populated,
                Err(err) => warn!("failed to build drive collision grid: {err:#}"),
            }
        }
    }
}
