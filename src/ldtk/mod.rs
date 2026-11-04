//! LDtk project queries. IntGrid collision data lives in [`int_grid_cells`].

use crate::prelude::*;
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};
pub mod int_grid_cells;

/// Read-only access to loaded [`LdtkProject`] assets.
///
/// Use it as a system parameter. It does not spawn levels or wait for loads;
/// [`Self::first_level`] returns [`None`] and logs a warning when the project
/// is not loaded yet.
#[derive(SystemParam)]
pub struct LdtkInfo<'w> {
    projects: Res<'w, Assets<LdtkProject>>,
}

impl LdtkInfo<'_> {
    /// Returns the first level in the LDtk project, if any
    pub fn first_level(&self, project: &Handle<LdtkProject>) -> Option<&Level> {
        let projects: &Assets<LdtkProject> = self.projects.as_ref();
        let Some(project) = projects.get(project) else {
            warn!("LdtkInfo: LDtk project was not loaded, so the level was not framed");
            return None;
        };
        let Some(level) = project.iter_raw_levels().next() else {
            warn!("LdtkInfo: LDtk project has no levels");
            return None;
        };
        Some(level)
    }
}
