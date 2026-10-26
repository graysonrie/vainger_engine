use crate::prelude::*;
use bevy_ecs_ldtk::{ldtk::Level, prelude::*};

/// Provides convenience methods around common LDtk operations
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
