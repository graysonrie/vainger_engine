//! A few shared meshes and color materials, filled in at startup.

use crate::prelude::*;

/// Handles created by [`PrimitiveAssetsPlugin`].
///
/// ### NOTE:
/// A Startup system fills these in. During other Startup systems they can
/// still be the default empty handles, and an empty handle does not draw.
/// Read them from [`Update`] onward.
#[allow(unused)]
#[derive(Resource, Default)]
pub struct PrimitiveAssets {
    /// A 32×32 rectangle mesh, in world pixels.
    pub mesh_rect_32_32: Handle<Mesh>,
    /// Unlit material, linear RGB `(1.0, 0.5, 0.5)`.
    pub material_orange: Handle<ColorMaterial>,
    /// Unlit material, linear RGB `(0.0, 1.0, 0.5)`.
    pub material_green: Handle<ColorMaterial>,
}

/// Spawns [`PrimitiveAssets`] on startup.
///
/// Already added by [`crate::app::AppExt::setup_for_default_2d_pixel_game`].
pub struct PrimitiveAssetsPlugin;

impl Plugin for PrimitiveAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrimitiveAssets>()
            .add_systems(Startup, setup);
    }
}

fn setup(
    mut assets: ResMut<PrimitiveAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh_rect_32_32 = meshes.add(Rectangle::new(32., 32.));
    let material_orange = materials.add(Color::linear_rgb(1., 0.5, 0.5));
    let material_green = materials.add(Color::linear_rgb(0.0, 1.0, 0.5));

    *assets = PrimitiveAssets {
        mesh_rect_32_32,
        material_orange,
        material_green,
    };
}
