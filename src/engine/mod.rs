mod cam;
mod instance;

use bytemuck::{Pod, Zeroable};
pub use cam::*;
pub use instance::*;
use vecto_rs::linear::{Mat4, Vector, Vector4};
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ParticleInstance {
    position: Vector,
    velocity : Vector,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct RawParticleInstance {
    offset_vector : [f32; 2]
}

impl ParticleInstance {
    pub fn update(&mut self)
    {
        // self.velocity.y -= 0.01;
        // if self.position.y < 0.0
        // {
        //     self.velocity.y *= -1.;
        // }
        // self.position += self.velocity;

    }

    pub fn raw(&self) -> RawParticleInstance {
        RawParticleInstance {
            offset_vector: [self.position.x, self.position.y],
        }
    }
}

impl RawParticleInstance {
    const ATTRIB: [VertexAttribute; 1] =
        vertex_attr_array![5 => Float32x2];
    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            step_mode: VertexStepMode::Instance,
            array_stride: std::mem::size_of::<RawParticleInstance>() as wgpu::BufferAddress,
            attributes: &Self::ATTRIB,
        }
    }
}
