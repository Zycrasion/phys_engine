use bytemuck::{Pod, Zeroable};
use vecto_rs::linear::{Mat4, Vector, VectorTrait};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, Device, Queue,
    ShaderStages,
};
use winit::dpi::PhysicalSize;

use crate::SIDE_LENGTH;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_array(
    [  
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 0.5, 0.5,
        0.0, 0.0, 0.0, 1.0,
    ]
);

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view_projection: [f32; 16],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_projection: Mat4::identity().get_contents(),
        }
    }

    pub fn update(&mut self, proj: Mat4) {
        self.view_projection = proj.get_contents();
    }
}

pub struct Camera {
    pub eye: Vector,
    pub width: f32,
    pub height: f32,

    uniform: CameraUniform,
    buffer: Buffer,
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl Camera {
    pub fn new(size: PhysicalSize<u32>, device: &Device) -> Self {
        let uniform = CameraUniform::new();
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Camera Uniform Init"),
            contents: bytemuck::cast_slice(&[uniform.view_projection]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Camera Uniform Bind Group Layout"),
            entries: &[Self::projection_layout()],
        });

        let bind_group = Self::create_bind_group(&buffer, &bind_group_layout, device);

        Self {
            eye: Vector::new3(0., 0., -2.),
            width: size.width as f32,
            height: size.height as f32,
            buffer,
            uniform,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    fn create_bind_group(buffer: &Buffer, layout: &BindGroupLayout, device: &Device) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    fn projection_layout() -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    pub fn update(&mut self, queue: &Queue) {
        self.uniform.update(self.build_projection_matrix());
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn build_projection_matrix(&self) -> Mat4 {
        let view = Mat4::new_translation(self.eye * -1.);
        let aspect = self.height / self.width;

        let projection = Mat4::new_orthographic_matrix(0., SIDE_LENGTH as f32, 0., SIDE_LENGTH as f32 / aspect, 0.1, 10.);
        // let view = Mat4::new_perspective_matrix(1., 1., 40., 0.1, 100.);

        projection * view
    }
}
