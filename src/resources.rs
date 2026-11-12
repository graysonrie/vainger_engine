use crate::prelude::*;

#[allow(unused)]
#[derive(Resource, Default)]
pub struct PrimitiveAssets {
    pub mesh_rect_32_32: Handle<Mesh>,
    pub material_orange: Handle<ColorMaterial>,
    pub material_green: Handle<ColorMaterial>,
}

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
