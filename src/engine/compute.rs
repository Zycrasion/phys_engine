use bytemuck::{Pod, Zeroable};
use vecto_rs::linear::Vector;
use wgpu::{
    core::device::queue, include_wgsl, naga::front::wgsl, util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, Buffer, BufferSlice, BufferUsages, CommandEncoder, ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions, Queue, RenderPassDescriptor
};
use winit::window::Window;

use crate::SIDE_LENGTH;

use super::ParticleInstance;

#[repr(C)]
#[derive(Clone, Copy, Zeroable, Pod)]
struct Uniforms
{
    length : u32,
    _padding : u32,
    mouse_position : [f32; 2],
}

pub struct ParticleCompute {
    compute_pipeline: ComputePipeline,

    particle_bind_group: BindGroup,
    particle_buffer: Buffer,

    uniforms : Uniforms,
    uniform_buffer : Buffer,
}

impl ParticleCompute {
    pub fn new(device: &Device) -> Self {
        let side_length = SIDE_LENGTH as usize;
        let instance_count = side_length * side_length;
        let mut instances = Vec::with_capacity(instance_count);

        for i in 0..instance_count {
            instances.push(ParticleInstance::new(
                (i / side_length) as f32 + 0.5,
                (i % side_length) as f32 + 0.5,
            ));
        }

        let raw_instances = instances
            .iter()
            .map(ParticleInstance::raw)
            .collect::<Vec<_>>();
        let particle_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Particle Instance Buffer"),
            contents: bytemuck::cast_slice(&raw_instances),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST | BufferUsages::STORAGE,
        });

        let uniforms = Uniforms
        {
            length : SIDE_LENGTH as u32,
            _padding : 0,
            mouse_position : [SIDE_LENGTH as f32 / 2., SIDE_LENGTH as f32 / 2.],
        };

        let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor
        {
            label : Some("Uniform Buffer"),
            contents : bytemuck::cast_slice(&[uniforms]),
            usage : BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let compute_shader = include_wgsl!("particle_compute.wgsl");
        let compute_shader = device.create_shader_module(compute_shader);
        let compute_pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("Particle Compute Pipeline"),
            layout: None,
            module: &compute_shader,
            entry_point: "main",
            compilation_options: PipelineCompilationOptions::default(),
            cache: None,
        });
        let particle_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Particle Buffer"),
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        Self {
            particle_buffer,
            compute_pipeline,
            particle_bind_group,
            uniform_buffer,
            uniforms
        }
    }

    pub fn particle_count(&self) -> u32
    {
        (SIDE_LENGTH * SIDE_LENGTH) as u32
    }

    pub fn get_particle_buffer(&self) -> BufferSlice
    {
        self.particle_buffer.slice(..)
    }

    pub fn mouse(&mut self, mouse : Vector, queue : &Queue)
    {
        self.uniforms.mouse_position = [mouse.x, mouse.y];
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));
    }

    pub fn compute(&self, encoder: &mut CommandEncoder) {
        let mut particle_compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("Particle Compute"),
            timestamp_writes: None,
        });

        

        particle_compute_pass.set_pipeline(&self.compute_pipeline);
        particle_compute_pass.set_bind_group(0, &self.particle_bind_group, &[]);
        particle_compute_pass.dispatch_workgroups(SIDE_LENGTH as u32, SIDE_LENGTH as u32, 1);
    }
}
