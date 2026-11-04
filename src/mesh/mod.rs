//! Colored and textured 2D triangle meshes, built in world pixels (Y-up).

use bevy::{asset::RenderAssetUsages, mesh::Indices};

use crate::prelude::*;

/// Accumulates quads, then bakes them into a triangle-list [`Mesh`].
///
/// Positions are in the entity's local space. Facing is `+Z`. Build one mesh,
/// insert it as an asset, and draw it with a `Mesh2d` and a [`ColorMaterial`]
/// on an entity that uses [`crate::pixel_perfect_cam::GAME_LAYERS`]
/// (the default) so it lands on the low-res canvas.
#[derive(Default)]
pub struct MeshBuilder2D {
    /// ```text
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
    /// ```text
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
    /// Empty builder. Same as [`Default::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// How many vertices have been pushed so far.
    pub fn num_vertices(&self) -> usize {
        self.positions.len()
    }

    /// Appends a solid quad covering `rect` (Bevy Y-up: `min` is bottom-left).
    ///
    /// `color` is linear RGBA in `0..=1`, not sRGB bytes. All four vertices
    /// share it. UVs are `(0, 0)` and the normal is `+Z`.
    ///
    /// ### NOTE:
    /// This does not write triangle indices. [`Self::build`] only draws indexed
    /// triangles, so a mesh made only of these quads is invisible.
    /// [`Self::push_rectangle_quad`] does write indices. There is no public way
    /// to add indices for a colored quad after the fact.
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

    /// Appends a textured quad covering `rect`, with triangle indices.
    ///
    /// `rect` is a normal Bevy rect: `min` is the bottom-left and `max` is the
    /// top-right. `uv` is in `0..=1` texture space and is mapped like this:
    ///
    /// - bottom-left vertex: `(uv.max.x, uv.min.y)`
    /// - bottom-right vertex: `(uv.min.x, uv.min.y)`
    /// - top-left vertex: `(uv.min.x, uv.max.y)`
    /// - top-right vertex: `(uv.max.x, uv.max.y)`
    ///
    /// ### NOTE:
    /// U is mirrored: the left edge of the quad receives `uv.max.x`. Pass a
    /// flipped `uv` if you need the texture's left edge on the quad's left.
    /// Vertices from this quad are indexed; vertices from
    /// [`Self::push_colored_quad`] are not, even when mixed into the same builder.
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

    /// Consumes the builder and returns a triangle-list mesh.
    ///
    /// The color attribute is linear RGBA. An empty index buffer draws nothing;
    /// see [`Self::push_colored_quad`].
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
