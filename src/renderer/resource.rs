use wgpu::util::DeviceExt;

use super::model;
use super::texture;

pub fn load_model_bytes(
    label: &str,
    data: &[u8],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
) -> Result<model::Model, String> {
    // i'm on my own now...
    let (document, buffers, images) = gltf::import_slice(data).unwrap();
    let mut materials = Vec::new();
    for material in document.materials() {
        let buffer_index = material
            .pbr_metallic_roughness()
            .base_color_texture()
            .unwrap()
            .texture()
            .source()
            .index();
        let dimensions = (images[buffer_index].width, images[buffer_index].height);

        let label = material.name().unwrap_or("Unnamed material").to_string();

        let texture = texture::Texture::from_raw_data(
            Some(&format!("{} texture", label)),
            device,
            queue,
            &images[buffer_index].pixels,
            dimensions,
        )
        .unwrap();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} bind group", label)),
            layout,
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

        materials.push(model::Material {
            label,
            texture,
            bind_group,
        });
    }

    let mut meshes = Vec::new();
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let mut vertices = Vec::new();
            if let Some(attribute) = reader.read_positions() {
                for position in attribute {
                    vertices.push(model::ModelVertex {
                        position,
                        tex_coords: Default::default(),
                    });
                }
            }
            if let Some(attribute) = reader.read_tex_coords(0).map(|x| x.into_f32()) {
                let mut field_iterator = 0;
                for coords in attribute {
                    vertices[field_iterator].tex_coords = coords;
                    field_iterator += 1;
                }
            }

            let mut indices = Vec::new();
            if let Some(raw_indices) = reader.read_indices() {
                indices.append(&mut raw_indices.into_u32().collect::<Vec<u32>>());
            }

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

            meshes.push(model::Mesh {
                label: label.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: indices.len() as u32,
                material: primitive.material().index().unwrap(),
            });
        }
    }

    return Ok(model::Model { meshes, materials });
}
