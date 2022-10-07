use nannou::prelude::*;
use std::cell::RefCell;

mod data;



struct Model {
    graphics: RefCell<Graphics>,
}

struct Graphics {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
    bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vertex {
    position: (f32, f32, f32),
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Uniforms {
    world: Mat4,
    view: Mat4,
    proj: Mat4,
}

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn main() {
    nannou::app(model).update(update).run();
}

fn model(app: &App) -> Model {
    let w_id = app.new_window().size(1280, 720).view(view).build().unwrap();
    let window = app.window(w_id).unwrap();
    let device = window.device();
    let format = Frame::TEXTURE_FORMAT;
    let msaa_samples = window.msaa_samples();
    let (win_w, win_h) = window.inner_size_pixels();

    let vs_desc = wgpu::include_wgsl!("shaders/vs.wgsl");
    let fs_desc = wgpu::include_wgsl!("shaders/fs.wgsl");
    let vs_mod = device.create_shader_module(&vs_desc);
    let fs_mod = device.create_shader_module(&fs_desc);

    let vertices_bytes = vertices_as_bytes(&data::VERTICES);
    let indices_bytes = indices_as_bytes(&data::INDICES);
    let vertex_usage = wgpu::BufferUsages::VERTEX;
    let index_usage = wgpu::BufferUsages::INDEX;
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: vertices_bytes,
        usage: vertex_usage,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: indices_bytes,
        usage: index_usage,
    });

    let depth_texture = create_depth_texture(device, [win_w, win_h], DEPTH_FORMAT, msaa_samples);
    let depth_texture_view = depth_texture.view().build();

    let uniforms = create_uniforms(0.0, [win_w, win_h]);
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST;
    let uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    let bind_group_layout = create_bind_group_layout(device);
    let bind_group = create_bind_group(device, &bind_group_layout, &uniform_buffer);
    let pipeline_layout = create_pipeline_layout(device, &bind_group_layout);
    let render_pipeline = create_render_pipeline(
        device,
        &pipeline_layout,
        &vs_mod,
        &fs_mod,
        format,
        DEPTH_FORMAT,
        msaa_samples,
    );

    let graphics = RefCell::new(Graphics {
        vertex_buffer,
        index_buffer,
        uniform_buffer,
        depth_texture,
        depth_texture_view,
        bind_group,
        render_pipeline,
    });

    Model { graphics }
}

fn update(_app: &App, _model: &mut Model, _update: Update) {}

fn view(app: &App, model: &Model, frame: Frame) {
    let mut g = model.graphics.borrow_mut();

    let depth_size = g.depth_texture.size();
    let frame_size = frame.texture_size();
    let device = frame.device_queue_pair().device();
    if frame_size != depth_size {
        let depth_format = g.depth_texture.format();
        let sample_count = frame.texture_msaa_samples();
        g.depth_texture = create_depth_texture(device, frame_size, depth_format, sample_count);
        g.depth_texture_view = g.depth_texture.view().build();
    }

    //rotation
    let rotation = app.time;
    let uniforms = create_uniforms(rotation, frame_size);
    let uniforms_size = std::mem::size_of::<Uniforms>() as wgpu::BufferAddress;
    let uniforms_bytes = uniforms_as_bytes(&uniforms);
    let usage = wgpu::BufferUsages::COPY_SRC;
    let new_uniform_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: None,
        contents: uniforms_bytes,
        usage,
    });

    let mut encoder = frame.command_encoder();
    encoder.copy_buffer_to_buffer(&new_uniform_buffer, 0, &g.uniform_buffer, 0, uniforms_size);
    let mut render_pass = wgpu::RenderPassBuilder::new()
        .color_attachment(frame.texture_view(), |color| color)
        .depth_stencil_attachment(&g.depth_texture_view, |depth| depth) //to assist in order of rendering based on depth.
        .begin(&mut encoder);
    render_pass.set_bind_group(0, &g.bind_group, &[]);
    render_pass.set_pipeline(&g.render_pipeline);
    render_pass.set_vertex_buffer(0, g.vertex_buffer.slice(..));
    render_pass.set_index_buffer(g.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
    let index_range = 0..data::INDICES.len() as u32;
    let start_vertex = 0;
    let instance_range = 0..1;
    render_pass.draw_indexed(index_range, start_vertex, instance_range);
}

fn create_uniforms(time: f32, [w, h]: [u32; 2]) -> Uniforms {
    let rotation = Mat4::from_rotation_y(time*3.0);
    let aspect_ratio = w as f32 / h as f32;
    let fov_y = std::f32::consts::FRAC_PI_2;
    let near = 0.01;
    let far = 100.0;
    let proj = Mat4::perspective_rh_gl(fov_y, aspect_ratio, near, far);
    let time2 = time*3.0;
    let eye = pt3(time2.cos()*2.0+0.3, time.sin()*3.0+1.0, time2.cos()+1.0);
    let target = Point3::ZERO;
    let up = Vec3::Z;
    let view = Mat4::look_at_rh(eye, target, up);
    let scale = Mat4::from_scale(Vec3::splat(time.sin()*0.01+0.015)); 
    Uniforms {
        world: rotation,
        view: (view * scale).into(),
        proj: proj.into(),
    }
}

fn create_depth_texture(
    device: &wgpu::Device,
    size: [u32; 2],
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::Texture {
    wgpu::TextureBuilder::new()
        .size(size)
        .format(depth_format)
        .usage(wgpu::TextureUsages::RENDER_ATTACHMENT)
        .sample_count(sample_count)
        .build(device)
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    wgpu::BindGroupLayoutBuilder::new()
        .uniform_buffer(wgpu::ShaderStages::VERTEX, false)
        .build(device)
}

fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    wgpu::BindGroupBuilder::new()
        .buffer::<Uniforms>(uniform_buffer, 0..1)
        .build(device, layout)
}

fn create_pipeline_layout(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::PipelineLayout {
    let desc = wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    };
    device.create_pipeline_layout(&desc)
}

fn create_render_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    vs_mod: &wgpu::ShaderModule,
    fs_mod: &wgpu::ShaderModule,
    dst_format: wgpu::TextureFormat,
    depth_format: wgpu::TextureFormat,
    sample_count: u32,
) -> wgpu::RenderPipeline {
    wgpu::RenderPipelineBuilder::from_layout(layout, vs_mod)
        .fragment_shader(&fs_mod)
        .color_format(dst_format)
        .color_blend(wgpu::BlendComponent::REPLACE)
        .alpha_blend(wgpu::BlendComponent::REPLACE)
        .add_vertex_buffer::<Vertex>(&wgpu::vertex_attr_array![0 => Float32x3])
        .depth_format(depth_format)
        .sample_count(sample_count)
        .build(device)
}

fn vertices_as_bytes(data: &[Vertex]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn indices_as_bytes(data: &[u16]) -> &[u8] {
    unsafe { wgpu::bytes::from_slice(data) }
}

fn uniforms_as_bytes(uniforms: &Uniforms) -> &[u8] {
    unsafe { wgpu::bytes::from(uniforms) }
}

/*fn main() {
    #[derive(Debug)]
    struct Slot {
        item: String,
        count: u8,
    }

    #[derive(Debug)]
    struct Player {
        username: String,
        user_id: u8,
        character: CharacterType,
        inventory: [[Slot; 6]; 3],
    }

    #[derive(Debug)]
    enum CharacterType {
        Hikaru(u8), //u8 for skin
        Junk(u8),
        Koyori(u8),
        Sihu(u8),
    }

    enum Packet {
        Movement(f32, f32, f32), //X, Y, and Z impulse
        Attack(u8),              //damage amount
        Taunt(u8),               //taunt id
        Interact,
        Kms,
        InventorySwap(Slot, Slot),
    }

    let mut player1 = Player {
        username: String::from("Axy0C"),
        user_id: 0,
        character: CharacterType::Koyori(0),
        inventory: array_init::array_init(|_| {
            array_init::array_init(|_| Slot {
                item: String::new(),
                count: 0,
            })
        }),
    };

    player1.inventory[0][0].item = String::from("sword");
    player1.inventory[0][1].item = String::from("bow");
    player1.inventory[0][2].item = String::from("arrow");
    player1.inventory[0][2].count = 16;

    player1.inventory[1][0].item = String::from("grape");
    player1.inventory[1][0].count = 42;
    player1.inventory[1][1].item = String::from("apple juice");
    player1.inventory[1][2].item = String::from("apple pie");

    player1.inventory[2][0].item = String::from("dildo");
    player1.inventory[2][1].item = String::from("knife");
    player1.inventory[2][2].item = String::from("bandage");
    player1.inventory[2][2].count = 3;
}*/
