use std::{mem::size_of, ops::Range};

use super::model;

pub struct Object {
    pub label: String,
    pub model: model::Model,
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: (f32, f32, f32),
    pub instances: Range<u32>,
    pub uniform: ObjectUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Object {
    pub fn new(
        device: &wgpu::Device,
        label: String,
        model: model::Model,
        position: Option<cgmath::Point3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        scale: Option<(f32, f32, f32)>,
        instances: Option<Range<u32>>,
    ) -> Self {
        let uniform = ObjectUniform::new();

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} object buffer", &label)),
            size: size_of::<ObjectUniform>() as u64, // windows xp version canned. ) :
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, //つづ: look into this and why it crashes when true.
        });

        let layout = Object::layout(device);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", &label)),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            label,
            model,
            position: position.unwrap_or((0.0, 0.0, 0.0).into()),
            rotation: rotation.unwrap_or((0.0, 0.0, 0.0, 1.0).into()),
            scale: scale.unwrap_or((1.0, 1.0, 1.0)),
            instances: instances.unwrap_or(Range { start: 0, end: 1 }),
            uniform,
            buffer,
            bind_group,
        }
    }

    // つづ: try making a global layout that only has to be built once.
    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("object bind group layout"),
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

    pub fn update(&mut self) {
        use cgmath::EuclideanSpace;
        self.uniform.transformation = (cgmath::Matrix4::from_translation(self.position.to_vec())
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.0, self.scale.1, self.scale.2))
        .into();
    }
}

// つづ: consider packing all of this into the object struct.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ObjectUniform {
    transformation: [[f32; 4]; 4],
}

impl ObjectUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            transformation: cgmath::Matrix4::identity().into(),
        }
    }
}
