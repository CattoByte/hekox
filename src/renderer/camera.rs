use std::mem::size_of;

use cgmath::{prelude::*, Matrix4, Point3, Vector3};
use wgpu::util::DeviceExt;

#[rustfmt::skip]
// why was this pub in the first place???
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub enum Projection {
    Perspective(f32),
    Orthographic,
}

pub struct Camera {
    pub label: String,
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub projection: Projection,
    pub znear: f32,
    pub zfar: f32,
    pub uniform: [[f32; 4]; 4],
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new<E: Into<Point3<f32>>, T: Into<Point3<f32>>, U: Into<Vector3<f32>>>(
        label: String,
        device: &wgpu::Device,
        eye: E,
        target: T,
        up: U,
        aspect: f32,
        projection: Projection,
        znear: f32,
        zfar: f32,
    ) -> Self {
        let uniform = Matrix4::identity().into();

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("{} camera buffer", label)),
            size: size_of::<[[f32; 4]; 4]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} camera bind group", label)),
            layout: &Camera::layout(&device),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            label,
            eye: eye.into(),
            target: target.into(),
            up: up.into(),
            aspect,
            projection,
            znear,
            zfar,
            uniform,
            buffer,
            bind_group,
        }
    }

    pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
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
        // this is kept as a separate function in case other things have to be updated; keeps
        // things clean.
        self.uniform = self.build_view_projection_matrix().into();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = match self.projection {
            Projection::Perspective(fov) => {
                cgmath::perspective(cgmath::Deg(fov), self.aspect, self.znear, self.zfar)
            } // つづ: consider renaming to 'fovy'.
            // つづ: look into how the ortho function actually works.
            Projection::Orthographic => cgmath::ortho(0.0, 1.0, 1.0, 0.0, -1.0, 1.0),
        };

        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}
