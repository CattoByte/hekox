use std::io::{BufReader, Cursor};
use gltf::Gltf;
use wgpu::util::DeviceExt;

use crate::renderer::model;
use crate::renderer::texture;

pub fn load_model(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue, layout: &wgpu::BindGroupLayout) -> model::Model {
    let text = std::fs::read_to_string(file_name).unwrap();
    let cursor = Cursor::new(text);
    let reader = BufReader::new(cursor);
    let gltf = Gltf::from_reader(reader).unwrap();

    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        if let gltf::buffer::Source::Uri(uri) = buffer.source() {
            let path = std::path::Path::new("/home/cattobyte/ドキュメント/rust/hekox/src/models/junk/").join(file_name);
            let bin = std::fs::read(path).unwrap();
            buffer_data.push(bin);
        } else {
            panic!("this isn't a file blob!");
        }
    }
    /*for buffer in gltf.buffers() {
        if let gltf::buffer::Source::Bin = buffer.source() {
            buffer_data.push(gltf.blob.as_deref().unwrap());
        } else {
            panic!("this isn't a binary blob!");
        }
    }*/

    let mut materials = Vec::new();
    for material in gltf.materials() {
        let texture_info = material.pbr_metallic_roughness().base_color_texture();
        let texture_source = texture_info.map(|texture| {
            texture.texture().source().source()
        }).expect("texture source error (figure it out lol).");

        let texture = if let gltf::image::Source::Uri {uri,  mime_type } = texture_source {
            let path = std::path::Path::new("/home/cattobyte/ドキュメント/rust/hekox/src/models/junk/").join(uri);
            let data = std::fs::read(path).unwrap();
            texture::Texture::from_bytes(
                file_name,
                device,
                queue,
                &data,
            ).expect("failed to load texture")
        } else {
            panic!("uri source texture");
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&format!("{} material bind group", material.name().unwrap_or("unnamed material")).to_string()),
                //format!("{} material bind group", material.name()
                  //         .unwrap_or("unnamed material")).to_string(),
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
            name: format!("{} material", material.name().unwrap_or("unnamed material").to_string()),
            texture,
            bind_group,
        });
    }

    let mut meshes = Vec::new();
    for scene in gltf.scenes() {
        for node in scene.nodes() {
            let mesh = node.mesh().expect("no node!");

            let primitives = mesh.primitives();
            primitives.for_each(|i| {
                let reader = i.reader(|buffer| Some(&buffer_data[buffer.index()]));

                let mut vertices = Vec::new();
                if let Some(vertex_attribute) = reader.read_positions() {
                    vertex_attribute.for_each(|vertex| {
                        vertices.push(model::ModelVertex {
                            position: vertex,
                            tex_coords: Default::default(),
                        })
                    });
                }
                if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|i| i.into_f32()) {
                    let mut tex_coord_index = 0;
                    tex_coord_attribute.for_each(|tex_coord| {
                        vertices[tex_coord_index].tex_coords = tex_coord;
                        tex_coord_index += 1;
                    });
                }

                let mut indices = Vec::new();
                if let Some(indices_raw) = reader.read_indices() {
                    indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
                }

                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} vertex buffer", file_name)),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some(&format!("{} index buffer", file_name)),
                    contents: bytemuck::cast_slice(&indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                meshes.push(model::Mesh {
                    name: file_name.to_string(),
                    vertex_buffer,
                    index_buffer,
                    num_elements: indices.len() as u32,
                    material: 0,
                });
            });
        }
    }

    model::Model { meshes, materials }
}
