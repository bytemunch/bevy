use std::f32::consts::PI;

use crate::{
    mesh::{Indices, Mesh, MeshBuilder, Meshable},
    render_asset::RenderAssetUsages,
};
use bevy_math::primitives::Ellipsoid;
use wgpu::PrimitiveTopology;

/// Ellipsoid mesh options
#[derive(Clone, Copy, Debug)]
pub struct EllipsoidOptions {
    /// The number of longitudinal sectors, aka the horizontal resolution.
    #[doc(alias = "horizontal_resolution")]
    sectors: usize,
    /// The number of latitudinal stacks, aka the vertical resolution.
    #[doc(alias = "vertical_resolution")]
    stacks: usize,
}

impl Default for EllipsoidOptions {
    fn default() -> Self {
        Self {
            sectors: 32,
            stacks: 16,
        }
    }
}

/// A builder used for creating a [`Mesh`] with an [`Ellipsoid`] shape.
#[derive(Clone, Copy, Debug, Default)]
pub struct EllipsoidMeshBuilder {
    /// The [`Ellipsoid`] shape.
    pub ellipsoid: Ellipsoid,
    pub options: EllipsoidOptions,
}

impl EllipsoidMeshBuilder {
    /// Creates a new [`SphereMeshBuilder`] from a radius and [`SphereKind`].
    #[inline]
    pub const fn new(a: f32, b: f32, c: f32) -> Self {
        Self {
            ellipsoid: Ellipsoid { a, b, c },
            options: EllipsoidOptions {
                sectors: 32,
                stacks: 16,
            },
        }
    }

    /// Creates a UV ellipsoid [`Mesh`] with the given number of
    /// longitudinal sectors and latitudinal stacks, aka horizontal and vertical resolution.
    ///
    /// A good default is `32` sectors and `18` stacks.
    pub fn uv(&self, sectors: usize, stacks: usize) -> Mesh {
        // Largely inspired from http://www.songho.ca/opengl/gl_sphere.html

        let sectors_f32 = sectors as f32;
        let stacks_f32 = stacks as f32;
        let a_inv = 1. / self.ellipsoid.a;
        let b_inv = 1. / self.ellipsoid.b;
        let c_inv = 1. / self.ellipsoid.c;
        let sector_step = 2. * PI / sectors_f32;
        let stack_step = PI / stacks_f32;

        let mut vertices: Vec<[f32; 3]> = Vec::with_capacity(stacks * sectors);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(stacks * sectors);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(stacks * sectors);
        let mut indices: Vec<u32> = Vec::with_capacity(stacks * sectors * 2 * 3);

        for i in 0..stacks + 1 {
            let stack_angle = PI / 2. - (i as f32) * stack_step;
            let x = self.ellipsoid.a * stack_angle.cos();
            let y = self.ellipsoid.b * stack_angle.cos();
            let z = self.ellipsoid.c * stack_angle.sin();

            for j in 0..sectors + 1 {
                let sector_angle = (j as f32) * sector_step;
                let x = x * sector_angle.cos();
                let y = y * sector_angle.sin();

                vertices.push([x, y, z]);
                normals.push([x * a_inv, y * b_inv, z * c_inv]);
                uvs.push([(j as f32) / sectors_f32, (i as f32) / stacks_f32]);
            }
        }

        // indices
        //  k1--k1+1
        //  |  / |
        //  | /  |
        //  k2--k2+1
        for i in 0..stacks {
            let mut k1 = i * (sectors + 1);
            let mut k2 = k1 + sectors + 1;
            for _j in 0..sectors {
                if i != 0 {
                    indices.push(k1 as u32);
                    indices.push(k2 as u32);
                    indices.push((k1 + 1) as u32);
                }
                if i != stacks - 1 {
                    indices.push((k1 + 1) as u32);
                    indices.push(k2 as u32);
                    indices.push((k2 + 1) as u32);
                }
                k1 += 1;
                k2 += 1;
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl MeshBuilder for EllipsoidMeshBuilder {
    /// Builds a [`Mesh`] according to the configuration in `self`.
    ///
    /// # Panics
    ///
    /// Panics if the sphere is a [`SphereKind::Ico`] with a subdivision count
    /// that is greater than or equal to `80` because there will be too many vertices.
    fn build(&self) -> Mesh {
        self.uv(self.options.sectors, self.options.stacks)
    }
}

impl Meshable for Ellipsoid {
    type Output = EllipsoidMeshBuilder;

    fn mesh(&self) -> Self::Output {
        EllipsoidMeshBuilder {
            ellipsoid: *self,
            ..Default::default()
        }
    }
}

impl From<Ellipsoid> for Mesh {
    fn from(ellipsoid: Ellipsoid) -> Self {
        ellipsoid.mesh().build()
    }
}
