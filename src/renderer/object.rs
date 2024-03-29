use std::mem::size_of;

use cgmath::Matrix4;
use cgmath::SquareMatrix;
use wgpu::util::DeviceExt;

use super::instance;
use super::model;

pub struct Object {
    pub label: String,
    pub model: model::Model,
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: (f32, f32, f32),
    pub instances: Vec<instance::Instance>,
    // there would be an instance count here, but vectors have their own length field.
    pub instance_buffer: wgpu::Buffer,
    pub transformation_uniform: [[f32; 4]; 4],
    pub transformation_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Object {
    pub fn new(
        label: String,
        device: &wgpu::Device,
        model: model::Model,
        position: Option<cgmath::Point3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        scale: Option<(f32, f32, f32)>,
        instances: Option<Vec<instance::Instance>>,
    ) -> Self {
        // this code is here to show how not accounting for collect() can break everything.
        // this code would also be useless since it wouldn't store the sole instance in the
        // instances variable if none was provided...
        /*let instance_data = if let Some(taste_buds_last_around_ten_days) = instances {
            taste_buds_last_around_ten_days
        } else {
            vec![instance::Instance {
                position: (0.0, 0.0, 0.0).into(),
                rotation: (0.0, 0.0, 0.0, 1.0).into(),
            }]
        }.iter().map(instance::Instance::to_raw).collect();*/

        let instances = if let Some(taste_buds_last_around_ten_days) = instances {
            taste_buds_last_around_ten_days
        } else {
            vec![instance::Instance {
                position: (0.0, 0.0, 0.0).into(),
                rotation: (0.0, 0.0, 0.0, 1.0).into(),
            }]
        };

        let instance_data: Vec<instance::InstanceRaw> =
            instances.iter().map(instance::Instance::to_raw).collect();
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("{} instance buffer", &label)),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let transformation_uniform = Matrix4::identity().into();

        let transformation_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} object buffer", label)),
            size: size_of::<[[f32; 4]; 4]>() as u64, // windows xp version canned. ) :
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false, //つづ: look into this and why it crashes when true.
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", &label)),
            layout: &Object::layout(&device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: transformation_buffer.as_entire_binding(),
            }],
        });

        Self {
            label,
            model,
            position: position.unwrap_or((0.0, 0.0, 0.0).into()),
            rotation: rotation.unwrap_or((0.0, 0.0, 0.0, 1.0).into()),
            scale: scale.unwrap_or((1.0, 1.0, 1.0)),
            instances,
            instance_buffer,
            transformation_uniform,
            transformation_buffer,
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

    pub fn update(&mut self, queue: &wgpu::Queue) {
        use cgmath::EuclideanSpace;
        self.transformation_uniform = (cgmath::Matrix4::from_translation(self.position.to_vec())
            * cgmath::Matrix4::from(self.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(self.scale.0, self.scale.1, self.scale.2))
        .into();

        queue.write_buffer(
            &self.transformation_buffer,
            0,
            bytemuck::cast_slice(&[self.transformation_uniform]),
        );
    }
}
