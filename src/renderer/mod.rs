use cgmath::Rotation3;
use model::Vertex;
use std::{ops::Mul, sync::Arc};
use wgpu::util::DeviceExt;
// つづ: see if game_loop's winit should be used.
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

use self::model::DrawObject; // this is a trait (which might be deprecated lol).

mod camera;
mod instance;
mod model;
pub mod object;
pub mod resource;
pub mod texture;

const NUM_INSTANCES_PER_ROW: u32 = 2;
static mut INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

pub struct State {
    window: Arc<Window>,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    camera: camera::Camera,
    depth_texture: texture::Texture,
    pub objects: Vec<object::Object>,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    // Wgpu requires some async code
    pub async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&*window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None, // つづ: check if label can be asigned to device.
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0], // つづ: run some tests on this. it may cause
            // some issues just like present_mode did...
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let mut camera = camera::Camera::new(
            "the".to_string(), // the camera uniform, the camera buffer, etc.
            &device,
            (0.0, 0.0, 6.0),
            (0.0, 0.0, 0.0),
            cgmath::Vector3::unit_y(),
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        camera.update(&queue);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth texture");

        let objects: Vec<object::Object> = Vec::new();

        /*for i in &mut objects {
            i.update(&queue);
        }*/

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            // look into using the include_wgsl! macro.
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[
                    &texture::Texture::layout(&device),
                    &camera::Camera::layout(&device),
                    &object::Object::layout(&device),
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING), // no linear!
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            window,
            size,
            surface,
            device,
            queue,
            config,
            camera,
            depth_texture,
            objects,
            render_pipeline,
        }
    }

    // つづ: reconsider this; could be deprecated in favour of making the window variable public.
    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        //if new_size.width > 0 && new_size.height > 0
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth texture");
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self, elapsed: f32) {
        /*use cgmath::InnerSpace;
        let forwards = self.camera.target - self.camera.eye;
        let forwards_norm = forwards.normalize();
        let forwards_mag = forwards.magnitude();
        let right = forwards_norm.cross(cgmath::Vector3::unit_y());*/

        /*self.camera.eye = self.camera.target
            - (forward - right * (COUNTER / 2.0).cos() * 0.0125).normalize() * forward_mag;
        //self.camera.eye.z = COUNTER.cos() * 0.2 + 1.0;
        self.camera.up.x = COUNTER.cos() * 0.4;
        self.camera.eye.y = (COUNTER * 0.2).cos() * 1.5;
        self.camera.target.y = COUNTER.sin() * 0.5; // i truly don't understand.
                                                    // turns out i put += instead of =...
                                                    //INSTANCE_DISPLACEMENT.y = (NUM_INSTANCES_PER_ROW as f32
                                                    //   - NUM_INSTANCES_PER_ROW as f32 / 2.0)
                                                    // * COUNTER.cos()
                                                    // * 0.1;
        COUNTER += 0.02;

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                use cgmath::prelude::*;
                (0..NUM_INSTANCES_PER_ROW).flat_map(move |x| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |y| {
                        let modified_y = y as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0; // * (COUNTER * 0.2).sin() * 0.5;
                        INSTANCE_DISPLACEMENT.y = modified_y * (COUNTER * 0.125).cos() * 0.1 + 0.5;
                        let position = unsafe {
                            cgmath::Vector3 {
                                x: x as f32,
                                y: modified_y,
                                z: z as f32,
                            } - INSTANCE_DISPLACEMENT
                        }; // つづ: remove unsafe once INSTANCE_DISPLACEMENT is no longer static.
                        let rotation = if position.is_zero() {
                            cgmath::Quaternion::from_axis_angle(
                                cgmath::Vector3::unit_z(),
                                cgmath::Deg(0.0),
                            )
                        } else {
                            let quat_a =
                                cgmath::Quaternion::from_angle_z(cgmath::Deg(COUNTER.cos() * 30.0));
                            let quat_b =
                                cgmath::Quaternion::from_angle_y(cgmath::Deg(COUNTER.sin() * 30.0));
                            let quat_c = cgmath::Quaternion::from_angle_y(cgmath::Deg(180.0));
                            let tmp = quat_b.mul(quat_a);
                            quat_c.mul(tmp)
                        };

                        instance::Instance { position, rotation }
                    })
                })
            })
            .collect::<Vec<_>>();

        let instance_data = instances
            .iter()
            .map(instance::Instance::to_raw)
            .collect::<Vec<_>>();

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        );

        self.objects[0].position.x = (COUNTER * 0.125).sin();
        self.objects[0].position.y = (COUNTER * 0.125).cos();
        self.objects[0].rotation =
            cgmath::Quaternion::from_angle_y(cgmath::Deg((COUNTER * 3.0) % 360.0));
        self.objects[0].scale = (1.0, (COUNTER * 0.25).sin() * 0.5 + 0.75, 1.0);
        self.objects[0].update(&self.queue);*/
        self.camera.update(&self.queue);
        for i in &mut self.objects {
            i.update(&self.queue);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });

        render_pass.set_pipeline(&self.render_pipeline);
        //render_pass.set_bind_group(0, &self.diffuse_bing_group, &[]);
        //render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
        //render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        //render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        //render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        //render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        for i in &self.objects {
            render_pass.draw_object_instanced(&i, &self.camera.bind_group);
        }

        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

// つづ: deprecate this if no use is found for it.
pub async fn run() {}
