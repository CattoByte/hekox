use cgmath::Rotation3;
use model::Vertex;
use std::ops::Mul;
use wgpu::util::DeviceExt;
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
mod object;
mod resource;
mod texture;

const NUM_INSTANCES_PER_ROW: u32 = 2;
static mut INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
    NUM_INSTANCES_PER_ROW as f32 * 0.5,
);

/*#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    //colour: [f32; 3],
    tex_coords: [f32; 2],
}

// つづ: use the vertex_attr_array macro.
// つづ: consider the above after model loading is implemented.
impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, 0.0], // top left
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0], // top right
        tex_coords: [-1.0, 1.0],
    },
    Vertex {
        position: [-0.5, 0.5, 0.0], // bottom left
        tex_coords: [1.0, -1.0],
    },
    Vertex {
        position: [0.5, 0.5, 0.0], // bottom right
        tex_coords: [-1.0, -1.0],
    },
];

const INDICES: &[u16] = &[
    // ccw...
    0, 1, 2, 3, 2, 1,
];
*/
struct State {
    window: Window,
    size: winit::dpi::PhysicalSize<u32>,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    diffuse_bing_group: wgpu::BindGroup,
    camera: camera::Camera,
    camera_uniform: camera::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    //vertex_buffer: wgpu::Buffer,
    //index_buffer: wgpu::Buffer,
    //num_indices: u32,
    objects: Vec<object::Object>,
    render_pipeline: wgpu::RenderPipeline,
}

impl State {
    // Wgpu requires some async code
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let diffuse_bytes = include_bytes!("../textures/test.png"); // will this always work?
        let diffuse_texture = texture::Texture::from_image_bytes(
            Some("diffuse_texture"),
            &device,
            &queue,
            diffuse_bytes,
        )
        .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bing_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("diffuse_bing_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let camera = camera::Camera {
            eye: (0.0, 0.0, 7.5).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fov: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let mut camera_uniform = camera::CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            // look into using the include_wgsl! macro.
        });

        /*let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;*/

        let instances = (0..NUM_INSTANCES_PER_ROW)
            .flat_map(|z| {
                use cgmath::prelude::*;
                (0..NUM_INSTANCES_PER_ROW).flat_map(move |x| {
                    (0..NUM_INSTANCES_PER_ROW).map(move |y| {
                        let position = unsafe {
                            cgmath::Vector3 {
                                x: x as f32,
                                y: y as f32,
                                z: z as f32,
                            } - INSTANCE_DISPLACEMENT
                        }; // つづ: remove unsafe once INSTANCE_DISPLACEMENT is no longer static.
                        let rotation = if position.is_zero() {
                            cgmath::Quaternion::from_axis_angle(
                                cgmath::Vector3::unit_z(),
                                cgmath::Deg(0.0),
                            )
                        } else {
                            cgmath::Quaternion::from_axis_angle(
                                position.normalize(),
                                cgmath::Deg(45.0),
                            )
                        };

                        instance::Instance { position, rotation }
                    })
                })
            })
            .collect::<Vec<_>>();

        let mut objects: Vec<object::Object> = Vec::new();
        let model_bytes = include_bytes!("../models/junk.glb");
        let model = resource::load_model_bytes(
            "junk",
            model_bytes,
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        objects.push(object::Object::new(
            &device,
            "junk".to_string(),
            model,
            None,
            None,
            None,
            None,
        ));
        let model_bytes = include_bytes!("../models/junk.glb");
        let model = resource::load_model_bytes(
            "junk",
            model_bytes,
            &device,
            &queue,
            &texture_bind_group_layout,
        )
        .unwrap();
        objects.push(object::Object::new(
            &device,
            "junk2".to_string(),
            model,
            Some((3.0, 0.0, -2.0).into()),
            None,
            Some((1.0, 5.0, 1.0).into()),
            None,
        ));
        for i in &mut objects {
            i.update(&queue);
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &object::Object::layout(&device),
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
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
            diffuse_bing_group,
            camera,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_texture,
            //vertex_buffer,
            //index_buffer,
            //num_indices,
            objects,
            render_pipeline,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        //if new_size.width > 0 && new_size.height > 0
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    unsafe fn update(&mut self) {
        /*use cgmath::InnerSpace;
        static mut COUNTER: f32 = 0.0;
        let forward = self.camera.target - self.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(cgmath::Vector3::unit_y());

        self.camera.eye = self.camera.target
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
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
            label: Some("render_pass"),
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
            render_pass.draw_object_instanced(&i, &self.camera_bind_group);
        }

        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(window).await;
    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::RedrawRequested(window_id) if window_id == state.window().id() => {
            unsafe {
                state.update();
            }
            match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size), // reconfigure if lost
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            state.window().request_redraw();
        }
        _ => {}
    });
}
