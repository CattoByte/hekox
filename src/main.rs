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

    use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer}; //Buffer... things
    use vulkano::command_buffer::ClearColorImageInfo;
    use vulkano::command_buffer::CopyImageToBufferInfo;
    use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo}; //Command buffer things
    use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet}; //I have no idea what I am doing
    use vulkano::device::physical::PhysicalDevice; //Physical device object
    use vulkano::device::{Device, DeviceCreateInfo, Features, QueueCreateInfo}; //Device object and functions
    use vulkano::format::ClearColorValue;
    use vulkano::format::Format;
    use vulkano::image::{ImageDimensions, StorageImage};
    use vulkano::instance::{Instance, InstanceCreateInfo}; //Instance object and functions
    use vulkano::pipeline::ComputePipeline; //Computational... things
    use vulkano::pipeline::Pipeline; //Pipeline uhhh
    use vulkano::pipeline::PipelineBindPoint;
    use vulkano::sync::{self, GpuFuture}; //Submission and synchronization

    let instance =
        Instance::new(InstanceCreateInfo::default()).expect("Failed to create Vulkan instance!");
    let physical = PhysicalDevice::enumerate(&instance)
        .next()
        .expect("No devices available!");

    for family in physical.queue_families() {
        println!(
            "Found a queue family with {:?} queue(s)!",
            family.queues_count()
        );
    }

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

    let data: i32 = 12;
    let buffer = CpuAccessibleBuffer::from_data(device.clone(), BufferUsage::all(), false, data)
        .expect("Failed to create buffer!");

    let source_content: Vec<i32> = (0..64).collect();
    let source =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, source_content)
            .expect("Failed to create buffer!");

    let destination_content: Vec<i32> = (0..64).map(|_| 0).collect();
    let destination = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::all(),
        false,
        destination_content,
    )
    .expect("Failed to create buffer!");

    let mut builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .copy_buffer(CopyBufferInfo::buffers(source.clone(), destination.clone()))
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = sync::now(device.clone())
        .then_execute(queue.clone(), command_buffer)
        .unwrap()
        .then_signal_fence_and_flush()
        .unwrap();

    future.wait(None).unwrap();

    let src_content = source.read().unwrap();
    let destination_content = destination.read().unwrap();
    assert_eq!(&*src_content, &*destination_content);

    let data_iter = 0..65536;
    let data_buffer =
        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), false, data_iter)
            .expect("Failed to create buffer!");

    mod cs {
        vulkano_shaders::shader! {
        ty: "compute",
        src: "
            #version 450
            layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

            layout(set = 0, binding = 0) buffer Data {
                uint data[];
            } buf;

            void main() {
                uint idx = gl_GlobalInvocationID.x;
                buf.data[idx] *= 12;
            }"
        }
    }

    let shader = cs::
        load(device.clone(  )   )  .expect("Failed to create shader module!");
}
