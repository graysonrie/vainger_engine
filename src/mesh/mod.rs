use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, RegularPolygonMeshBuilder},
};

use crate::prelude::*;

#[derive(Default)]
pub struct MeshBuilder2D {
    /// ```ignore
    ///   Suppose you want this shape:
    ///        B
    ///       / \
    ///      /   \
    ///     A-----C
    /// You could define its vertices as:
    /// let positions = vec![
    ///     [-50.0, -50.0, 0.0], // A
    ///     [  0.0,  50.0, 0.0], // B
    ///     [ 50.0, -50.0, 0.0], // C
    /// ];
    /// ```
    positions: Vec<[f32; 3]>,
    /// Describe which direction the surface faces.
    ///
    /// Ex: [0,0,1] means the surface faces towards the +Z direction
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    /// Tells the GPU which vertices make triangles
    ///
    /// Visually:
    /// ```ignore
    /// 3 -------- 2
    /// |        / |
    /// |      /   |
    /// |    /     |
    /// |  /       |
    /// 0 -------- 1
    /// Triangle 1: 0, 1, 2
    /// Triangle 2: 0, 2, 3
    /// ```
    indices: Vec<u32>,
    /// linear RGBA colors
    colors: Vec<[f32; 4]>,
}
impl MeshBuilder2D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_vertices(&self) -> usize {
        self.positions.len()
    }

    pub fn push_colored_quad(&mut self, rect: Rect, color: [f32; 4]) {
        let tl = [rect.min.x, rect.min.y, 0.];
        let tr = [rect.max.x, rect.min.y, 0.];
        let bl = [rect.min.x, rect.max.y, 0.];
        let br = [rect.max.x, rect.max.y, 0.];

        self.positions.extend_from_slice(&[tl, tr, bl, br]);
        self.uvs.extend_from_slice(&[[0., 0.]; 4]); // No UVs
        self.normals.extend_from_slice(&[[0., 0., 1.]; 4]);
        self.colors.extend_from_slice(&[color; 4]);
    }

    /// Where the `min` on `Rect` should correspond to the top-left position and
    /// `max` should correspond to the bottom-right position
    pub fn push_rectangle_quad(&mut self, rect: Rect, uv: Rect) {
        let base = self.positions.len() as u32;

        let tl = [rect.min.x, rect.min.y, 0.];
        let tr = [rect.max.x, rect.min.y, 0.];
        let bl = [rect.min.x, rect.max.y, 0.];
        let br = [rect.max.x, rect.max.y, 0.];

        self.positions.extend_from_slice(&[tl, tr, bl, br]);

        self.normals.extend_from_slice(&[[0., 0., 1.]; 4]);

        self.uvs.extend_from_slice(&[
            [uv.max.x, uv.min.y],
            [uv.min.x, uv.min.y],
            [uv.min.x, uv.max.y],
            [uv.max.x, uv.max.y],
        ]);
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn build(self) -> Mesh {
        Mesh::new(
            bevy::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(self.indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}
