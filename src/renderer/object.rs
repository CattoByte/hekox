use std::ops::Range;

use cgmath::EuclideanSpace;

use super::model;

pub struct Object {
    pub label: String,
    pub model: model::Model,
    pub position: cgmath::Point3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
    pub scale: (f32, f32, f32),
    pub instances: Range<u32>,
}

impl Object {
    pub fn new(
        label: String,
        model: model::Model,
        position: Option<cgmath::Point3<f32>>,
        rotation: Option<cgmath::Quaternion<f32>>,
        scale: Option<(f32, f32, f32)>,
        instances: Option<Range<u32>>,
    ) -> Self {
        Self {
            label,
            model,
            position: position.unwrap_or((0.0, 0.0, 0.0).into()),
            rotation: rotation.unwrap_or((0.0, 0.0, 0.0, 1.0).into()),
            scale: scale.unwrap_or((1.0, 1.0, 1.0)),
            instances: instances.unwrap_or(Range { start: 0, end: 1 }),
        }
    }
}

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
    pub fn update(&mut self, object: &Object) {
        self.transformation = (
            cgmath::Matrix4::from_translation(object.position.to_vec())
            * cgmath::Matrix4::from(object.rotation)
            * cgmath::Matrix4::from_nonuniform_scale(
                object.scale.0,
                object.scale.1,
                object.scale.2,
            )).into();
    }
}
