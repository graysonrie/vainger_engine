use crate::{components::Velocity2D, prelude::*};

pub(crate) struct IntGridCellsPlugin;

impl Plugin for IntGridCellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, populate_int_grid_cells);
    }
}

/// Reflects the IntGrid of the currently loaded level for the LDtk world
#[derive(Component)]
pub struct IntGridCells {
    pub width: i32,     // in cells
    pub height: i32,    // in cells
    pub cell_size: i32, // in pixels
    pub cells: Array2D<u8>,
    pub layer_name: &'static str,
}

/// Put this on the same entity containing the [`LdtkWorldBundle`]
///
/// Its IntGrid data will automatically get populated into here. Can be useful for
/// collision checking for example
impl IntGridCells {
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

#[derive(Clone, Default)]
pub struct TerrainContactsX {
    pub left: bool,
    pub right: bool,
    pub output: Vec2,
}

#[derive(Clone, Default)]
pub struct TerrainContactsY {
    pub floor: bool,
    pub ceiling: bool,
    pub output: Vec2,
}

pub trait CollisionCheck {
    fn is_colliding(&self, pos: Vec2, hitbox: &Hitbox) -> bool;
    #[must_use]
    /// Returns the output position. Assign that to the transform.translation, or whatever
    /// you are manipulating
    fn move_and_collide_x(
        &self,
        pos: Vec2,
        hitbox: &Hitbox,
        velocity: &mut Velocity2D,
    ) -> TerrainContactsX;
    #[must_use]
    /// Returns the output position. Assign that to the transform.translation, or whatever
    /// you are manipulating
    fn move_and_collide_y(
        &self,
        pos: Vec2,
        hitbox: &Hitbox,
        velocity: &mut Velocity2D,
    ) -> TerrainContactsY;
}

impl CollisionCheck for IntGridCells {
    fn is_colliding(&self, pos: Vec2, hitbox: &Hitbox) -> bool {
        self.overlaps_hitbox(hitbox, pos)
    }

    fn move_and_collide_x(
        &self,
        mut pos: Vec2,
        hitbox: &Hitbox,
        velocity: &mut Velocity2D,
    ) -> TerrainContactsX {
        if velocity.x != 0. && self.is_colliding(pos.add_to_x(velocity.x), hitbox) {
            while !self.is_colliding(pos.add_to_x(velocity.x.signum()), hitbox) {
                pos.x += velocity.x.signum();
            }
            velocity.x = 0.;
        }
        pos.x += velocity.x;
        let left = self.is_colliding(pos.add_to_x(-1.), hitbox);
        let right = self.is_colliding(pos.add_to_x(1.), hitbox);

        TerrainContactsX {
            left,
            right,
            output: pos,
        }
    }

    fn move_and_collide_y(
        &self,
        mut pos: Vec2,
        hitbox: &Hitbox,
        velocity: &mut Velocity2D,
    ) -> TerrainContactsY {
        if velocity.y != 0. && self.is_colliding(pos.add_to_y(velocity.y), hitbox) {
            // ceiling = velocity.y.is_sign_positive();
            // floor = velocity.y.is_sign_negative();
            while !self.is_colliding(pos.add_to_y(velocity.y.signum()), hitbox) {
                pos.y += velocity.y.signum();
            }
            velocity.y = 0.;
        }
        pos.y += velocity.y;
        let floor = self.is_colliding(pos.add_to_y(-1.), hitbox);
        let ceiling = self.is_colliding(pos.add_to_y(1.), hitbox);

        TerrainContactsY {
            floor,
            ceiling,
            output: pos,
        }
    }
}
