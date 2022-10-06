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
    use blue_engine::{
        header::{Engine, ObjectSettings, WindowDescriptor},
        primitive_shapes::cube,
    };
    let mut engine = Engine::new(WindowDescriptor::default()).expect("win");

    cube("Cube", &mut engine).unwrap();
    engine
        .objects
        .get_mut("Cube")
        .unwrap()
        .set_color(0f32, 0f32, 1f32, 1f32)
        .unwrap();

    let radius = 5f32;
    let start = std::time::SystemTime::now();
    engine
        .update_loop(move |_, _, _, _, camera| {
            let camx = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camy = start.elapsed().unwrap().as_secs_f32().sin() * radius;
            let camz = start.elapsed().unwrap().as_secs_f32().cos() * radius;
            camera
                .set_position(camx, camy, camz)
                .expect("Couldn't update the camera eye");
        })
        .expect("Error during update loop");
}
