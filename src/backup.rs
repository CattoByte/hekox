fn main() {
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

    dbg!(&player1);

    use bytemuck::{Pod, Zeroable};
    use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
    use vulkano::command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, CopyImageToBufferInfo, RenderPassBeginInfo,
        SubpassContents,
    };
    use vulkano::device::{physical::PhysicalDevice, Device, DeviceCreateInfo, QueueCreateInfo};
    use vulkano::format::Format;
    use vulkano::image::{view::ImageView, ImageDimensions, StorageImage};
    use vulkano::instance::{Instance, InstanceCreateInfo};
    use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
    use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
    use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
    use vulkano::pipeline::GraphicsPipeline;
    use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, Subpass};
    use vulkano::sync::{self, GpuFuture};
    use vulkano_win::VkSurfaceBuild;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::{Window, WindowBuilder};

    let instance =
        Instance::new(InstanceCreateInfo {
            enabled_extensions: vulkano_win::required_extensions(),
            ..Default::default()}).expect("Failed to create Vulkan instance!");
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No devices available!");
    //for family in physical.queue_families() {println!("Found a queue family with {:?} queue(s)!", family.queues_count());}
    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics())
        .expect("Couldn't find a graphical queue family, what kind of device is this..?");

    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo::family(queue_family)],
            ..Default::default()
        },
    )
    .expect("Failed to create device!");

    let queue = queues.next().unwrap();

    let image = StorageImage::new(
        device.clone(),
        ImageDimensions::Dim2d {
            width: 1024,
            height: 1024,
            array_layers: 1,
        },
        Format::R8G8B8A8_UNORM,
        Some(queue.family()),
    )
    .unwrap();

    let buf = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        (0..1024 * 1024 * 4).map(|_| 0u8),
    )
    .expect("Failed to create buffer!");

    #[repr(C)]
    #[derive(Default, Copy, Clone, Zeroable, Pod)]
    struct Vertex {
        position: [f32; 2],
    }
    vulkano::impl_vertex!(Vertex, position);

    let vertex1 = Vertex {
        position: [-0.5, -0.5],
    };
    let vertex2 = Vertex {
        position: [0.0, 0.5],
    };
    let vertex3 = Vertex {
        position: [0.5, -0.25],
    };

    let vertex_buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vec![vertex1, vertex2, vertex3].into_iter(),
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(device.clone(),
        attachments: {
            color: {
                load: Clear,
                store: Store,
                format: Format::R8G8B8A8_UNORM,
                samples: 1,
            }
        },
        pass: {
            color: [color],
            depth_stencil: {}
        }
    )
    .unwrap();

    let view = ImageView::new_default(image.clone()).unwrap();
    let framebuffer = Framebuffer::new(
        render_pass.clone(),
        FramebufferCreateInfo {
            attachments: vec![view],
            ..Default::default()
        },
    )
    .unwrap();
    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
            #version 450

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }"
        }
    }
    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
            #version 450

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }"
        }
    }

    let vs = vs::load(device.clone()).expect("Failed to create shader module!");
    let fs = fs::load(device.clone()).expect("Failed to create shader module!");

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [1024.0, 1024.0],
        depth_range: 0.0..1.0,
    };

    let pipeline = GraphicsPipeline::start()
        .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
        .vertex_shader(vs.entry_point("main").unwrap(), ())
        .input_assembly_state(InputAssemblyState::new())
        .viewport_state(ViewportState::viewport_fixed_scissor_irrelevant([viewport]))
        .fragment_shader(fs.entry_point("main").unwrap(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap();

    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassContents::Inline,
        )
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .draw(3, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap()
        .copy_image_to_buffer(CopyImageToBufferInfo::image_buffer(image, buf.clone()))
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();
    future.wait(None).unwrap();

    let event_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&event_loop, instance.clone()).unwrap();

    event_loop.run(|event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            },
            _ => ()
        }
    });

}
