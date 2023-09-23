use std::mem::size_of;

use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;

use super::{model, texture};

pub struct Element {
    pub label: String,
    pub position: cgmath::Point2<f32>,
    pub scale: (f32, f32),
    pub mesh: model::Mesh,
    pub material: model::Material, // つづ: support for alternate materials.
    pub transformation_uniform: [[f32; 4]; 4],
    pub transformation_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Element {
    pub fn new(
        label: String,
        device: &wgpu::Device,
        texture: texture::Texture,
        position: Option<cgmath::Point2<f32>>, // つづ: should these be options?
        //rotation: Option<cgmath::Quaternion<f32>>,
        scale: Option<(f32, f32)>,
    ) -> Self {
        /*let vertices: [model::ModelVertex; 4] = [
            model::ModelVertex {
                // top left
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            model::ModelVertex {
                // bottom left
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            model::ModelVertex {
                // bottom right
                position: [-1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            model::ModelVertex {
                // top right
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ];
        let indices: [u32; 6] = [0, 1, 2, 3, 0, 2];*/
        let vertices: [model::ModelVertex; 4] = [
            model::ModelVertex {
                // top left
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
            model::ModelVertex {
                // bottom left
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            model::ModelVertex {
                // bottom right
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            model::ModelVertex {
                // top right
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
        ];
        let indices: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} vertex buffer", label)),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} index buffer", label)),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let mesh = model::Mesh {
            label: format!("{} mesh", label),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", label)),
            layout: &texture::Texture::layout(device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        let material = model::Material {
            label: format!("{} material", label),
            texture,
            bind_group,
        };

        let transformation_uniform = Matrix4::identity().into();

        let transformation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} element buffer", label)),
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", &label)),
            layout: &Element::layout(&device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transformation_buffer.as_entire_binding(),
            }],
        });
        Self {
            label,
            position: position.unwrap_or((0.0, 0.0).into()),
            scale: scale.unwrap_or((1.0, 1.0)),
            mesh,
            material,
            transformation_uniform,
            transformation_buffer,
            bind_group,
        }
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("element bind group layout"),
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
        })
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        use cgmath::EuclideanSpace;
        // つづ: there has to be a way to optimize this...
        self.transformation_uniform =
            (cgmath::Matrix4::from_translation(
                cgmath::Point3::from((self.position.x, self.position.y, 1.0)).to_vec(),
            ) * cgmath::Matrix4::from_nonuniform_scale(self.scale.0, self.scale.1, 1.0))
            .into();

        queue.write_buffer(
            &self.transformation_buffer,
            0,
            bytemuck::cast_slice(&[self.transformation_uniform]),
        );
    }
}
