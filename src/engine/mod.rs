mod cam;
mod instance;
mod fps;

use bytemuck::{Pod, Zeroable};
pub use cam::*;
pub use instance::*;
use vecto_rs::linear::{Mat4, Vector, Vector4, VectorTrait};
use wgpu::{vertex_attr_array, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::SIDE_LENGTH;

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
    old_position : Vector,
}

#[repr(C)]
#[derive(Pod, Zeroable, Clone, Copy, Debug)]
pub struct RawParticleInstance {
    offset_vector : [f32; 2]
}

impl ParticleInstance {
    pub fn new(x : f32, y : f32) -> Self
    {
        Self
        {
            position : Vector::new2(x, y),
            old_position : Vector::new2(x, y)
        }
    }
    
    pub fn update(&mut self, raw : &mut RawParticleInstance)
    {
        let mut velocity = self.position - self.old_position;
        velocity.y -= 0.01;
        self.position += velocity;
        self.old_position = self.position - velocity;
        if self.position.y < 0.5
        {
            self.old_position.y = self.position.y + velocity.y;
            self.position.y = 0.5;
        }
        if self.position.x < -0.5
        {
            self.old_position.x += SIDE_LENGTH as f32;
            self.position.x += SIDE_LENGTH as f32;
        }

        if self.position.x > SIDE_LENGTH as f32 + 0.5
        {
            self.old_position.x -= SIDE_LENGTH as f32;
            self.position.x -= SIDE_LENGTH as f32;
        }

        raw.offset_vector[0] = self.position.x;
        raw.offset_vector[1] = self.position.y;
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
