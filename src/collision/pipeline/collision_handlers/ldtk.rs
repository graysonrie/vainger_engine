use crate::{
    collision::pipeline::{SolidWorldSyncSet, collision_check::SolidWorld},
    ldtk::int_grid_cells::{IntGridCells, populate_int_grid_cells},
    prelude::*,
};

/// Syncs LDtk [`IntGridCells`] into the shared [`SolidWorld`] resource.
pub struct LdtkSolidWorldPlugin;

impl Plugin for LdtkSolidWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SolidWorld>().add_systems(
            Update,
            sync_solid_world_from_ldtk
                .in_set(SolidWorldSyncSet)
                .after(populate_int_grid_cells),
        );
    }
}

fn sync_solid_world_from_ldtk(
    grids: Query<&IntGridCells, Changed<IntGridCells>>,
    mut solid: ResMut<SolidWorld>,
) {
    for grid in &grids {
        solid.load_grid(grid.width, grid.height, grid.cell_size, grid.cells.clone());
    }
}
