use std::time::Instant;

use env_logger::filter::Filter;
use vecto_rs::linear::{Vector, VectorTrait};
use wgpu::{
    core::instance, util::{BufferInitDescriptor, DeviceExt}, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsages, Color, FilterMode, FragmentState, ImageCopyTextureBase, Operations, Origin3d, PipelineCompilationOptions, PipelineLayoutDescriptor, PresentMode, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderStages, StoreOp, Texture, TextureDescriptor, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension
};
use winit::{dpi::PhysicalSize, event::WindowEvent, window::Window};

use super::{Camera, ParticleInstance, RawParticleInstance, Vertex};
use crate::SIDE_LENGTH;

const TRIANGLE_VERTS: &[Vertex] = &[
    Vertex {
        position: [0.5, 0.866025, 0.0],
        tex_coords: [0.5, 1. - 0.866025],
    },
    Vertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.0, 1.],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        tex_coords: [1.0, 1.],
    },
];

pub struct Instance<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: &'a winit::window::Window,

    particle_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera: Camera,
    particle_bind_group: BindGroup,

    instances : Vec<ParticleInstance>,
    instance_buffer : Buffer,

    frame_count : u32,
    start : Instant
}

impl<'a> Instance<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::DX12, // Primary emits warnings/errors https://github.com/gfx-rs/wgpu/issues/3959, DX12 or Vulkan is fine
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = if size.width == 0 || size.height == 0 {
            eprintln!("Width or Height of window is 0, this is invalid!");
            eprintln!("Defaulting to 100 width and 100 height");
            PhysicalSize::new(100, 100)
        } else {
            size
        };

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("particle_shader.wgsl").into()),
        });

        let particle_diffuse = include_bytes!("particle.png");
        let particle_image = image::load_from_memory(particle_diffuse).unwrap();
        let particle_rgba = particle_image.to_rgba8();

        let image_dimensions = particle_rgba.dimensions();
        let image_size = wgpu::Extent3d {
            width: image_dimensions.0,
            height: image_dimensions.1,
            depth_or_array_layers: 1,
        };

        let particle_texture = device.create_texture(&TextureDescriptor {
            size: image_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some("particle_texture"),
            view_formats: &[],
        });

        queue.write_texture(
            ImageCopyTextureBase {
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
                texture: &particle_texture,
            },
            &particle_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * image_dimensions.0),
                rows_per_image: Some(image_dimensions.1),
            },
            image_size,
        );

        let particle_view = particle_texture.create_view(&TextureViewDescriptor::default());
        let particle_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let particle_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Particle Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });
        let particle_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Particle Bind Group"),
            layout: &particle_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&particle_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&particle_sampler),
                },
            ],
        });

        let camera = Camera::new(size, &device);

        let side_length = SIDE_LENGTH as usize;
        let instance_count = side_length * side_length;
        let mut instances = Vec::with_capacity(instance_count);

        for i in 0..instance_count
        {
            instances.push(ParticleInstance
            {
                position : Vector::new2((i / side_length) as f32 + 0.5, (i % side_length) as f32 + 0.5),
                velocity : Vector::new2(0., 0.)
            });
        }

        let raw_instances = instances.iter().map(ParticleInstance::raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label : Some("Particle Instance Buffer"),
            contents : bytemuck::cast_slice(&raw_instances),
            usage : BufferUsages::VERTEX | BufferUsages::COPY_DST
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Particle Shader Layout"),
            bind_group_layouts: &[&particle_bind_group_layout, camera.layout()],
            push_constant_ranges: &[],
        });

        let particle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Particle Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Vertex::desc(), RawParticleInstance::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("My Vertex Buffer"),
            contents: bytemuck::cast_slice(TRIANGLE_VERTS),
            usage: wgpu::BufferUsages::VERTEX,
        });        

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            particle_pipeline,
            camera,
            vertex_buffer: buffer,
            particle_bind_group,
            instance_buffer,
            instances,
            frame_count : 0,
            start : Instant::now()
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.height == 0 || new_size.width == 0 {
            return;
        }

        self.size = new_size;
        self.camera.width = new_size.width as f32;
        self.camera.height = new_size.height as f32;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self) {
        self.camera.update(&self.queue);
        self.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instances.iter_mut().map(|f| {f.update(); f.raw()}).collect::<Vec<_>>()));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: wgpu::LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.particle_pipeline);

            render_pass.set_bind_group(0, &self.particle_bind_group, &[]);
            render_pass.set_bind_group(1, self.camera.group(), &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.draw(0..(TRIANGLE_VERTS.len() as u32), 0..self.instances.len() as _);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        self.frame_count += 1;

        Ok(())
    }

    pub fn estimate_fps(&self) -> f32
    {
        (self.frame_count as f32) / self.start.elapsed().as_secs_f32()
    }

    pub fn reconfig(&mut self) {
        self.resize(self.size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn window(&self) -> &Window {
        &self.window
    }
}
