mod renderer;
use renderer::object;
use renderer::resource;
use renderer::texture;
use renderer::ui;
use winit::application::ApplicationHandler;
use winit::event_loop;
use winit::event_loop::ActiveEventLoop;
use winit::event_loop::ControlFlow;
use winit::window::WindowId;
use std::{sync::Arc, time::Instant};

use cgmath::num_traits::ToPrimitive;
use game_loop::game_loop;
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::EventLoop,
    window::Window,
};

const UPDATES_PER_SECOND: u32 = 96;
const MAX_FRAME_TIME: f64 = 0.1;

struct Game {
    renderer_state: renderer::State,
    do_bricks_have_an_inherent_colour: u64,
    second_store: f32,
}

impl Game {
    pub fn new(window: &Arc<Window>) -> Self {
        // while initializing the variable below, i saw the 'Result<Window, OsError>' tip and
        // temporarily assumed that 'Window' was referring to the Windows OS.
        // i involuntarily frowned a little. oops.
        let renderer_state = pollster::block_on(renderer::State::new(window.clone()));
        let do_bricks_have_an_inherent_colour = 0;
        let second_store = 0.0;
        
        let mut tree_under_fire = Game {
            renderer_state,
            do_bricks_have_an_inherent_colour,
            second_store,
        };

        let device = &tree_under_fire.renderer_state.device;
        let queue = &tree_under_fire.renderer_state.queue;

        let mut objects: Vec<object::Object> = Vec::new();
        let tex_layout = texture::Texture::layout(device);
        let model_bytes = include_bytes!("./models/junk.glb");
        let model =
            resource::load_model_bytes("junk", model_bytes, device, queue, &tex_layout).unwrap();
        objects.push(object::Object::new(
            "junk".to_string(),
            device,
            model,
            None,
            None,
            Some((2.0, 2.0, 2.0).into()),
            None,
        ));
        tree_under_fire.renderer_state.objects = objects;

        let mut ui_elements: Vec<ui::Element> = Vec::new();
        let test_image = include_bytes!("./textures/test.png");
        let test_texture = texture::Texture::from_image_bytes(
            Some(&"test texture".to_string()),
            &device,
            &queue,
            test_image,
        )
        .unwrap();
        ui_elements.push(ui::Element::new(
            "test".to_string(),
            &device,
            test_texture,
            Some((0.0, 0.0, 3.0).into()),
            None,
            Some((1.25, 1.25)),
        ));
        tree_under_fire.renderer_state.ui_elements = ui_elements;

        tree_under_fire
    }

    pub fn update(&mut self) {
        use cgmath::prelude::*;
        use cgmath::Deg;

        self.do_bricks_have_an_inherent_colour += 1;
        let tweaked_bricks = self.do_bricks_have_an_inherent_colour as f32 / 90.0; //* ((self.do_bricks_have_an_inherent_colour as f32 / 180.0).sin() / 3.0 + 2.0);

        if self.second_store == 0.0 {
            self.renderer_state.objects[0].position.z = 2.0;
            self.renderer_state.objects[0].position.y = (tweaked_bricks * 4.0).sin().abs() - 2.25;
            self.renderer_state.objects[0].rotation = cgmath::Quaternion::from(cgmath::Euler {
                x: Deg(self.second_store),
                y: Deg(180.0 + self.second_store),
                z: Deg(((tweaked_bricks * 2.0).sin()) * 50.0),
            });
        } else {
            self.renderer_state.objects[0].position.y = -2.0;
            self.renderer_state.objects[0].position.z = 0.0;
            self.renderer_state.objects[0].rotation = cgmath::Quaternion::from(cgmath::Euler {
                x: Deg(self.second_store),
                y: Deg(180.0 + tweaked_bricks.sin()),
                z: Deg(self.second_store * 2.0),
            });
        }

        if self.do_bricks_have_an_inherent_colour % 360 < 90 {
            self.second_store += 4.0;
        } else {
            self.second_store = 0.0;
        }
        /*
        self.renderer_state.objects[0].rotation =
            cgmath::Quaternion::from_angle_x(cgmath::Deg(tweaked_bricks * 50.0))
                * cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks * 70.0))
                * cgmath::Quaternion::from_angle_z(cgmath::Deg(tweaked_bricks * 90.0));

        self.renderer_state.objects[1].position.x = (tweaked_bricks * 5.0).sin() * 1.5;
        self.renderer_state.objects[1].position.y = (tweaked_bricks * 7.0).sin() * 1.5;
        self.renderer_state.objects[1].position.z = (tweaked_bricks * 5.0).cos() * 1.5;
        self.renderer_state.objects[1].rotation =
            cgmath::Quaternion::from_angle_z(cgmath::Deg(tweaked_bricks * 300.0))
                * cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks * 180.0));

        self.renderer_state.objects[2].position.x = (tweaked_bricks * 4.0).sin() * 2.0;
        self.renderer_state.objects[2].position.y = (tweaked_bricks * 5.0).sin() * 2.0;
        self.renderer_state.objects[2].position.z = (tweaked_bricks * 3.0).cos() * 2.0;
        self.renderer_state.objects[2].rotation =
            cgmath::Quaternion::from_angle_z(cgmath::Deg(tweaked_bricks * 200.0))
                * cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks * 100.0));

        self.renderer_state.objects[3].position.x = (tweaked_bricks * 2.0).sin() * 2.5;
        self.renderer_state.objects[3].position.y = (tweaked_bricks * 3.0).sin() * 2.5;
        self.renderer_state.objects[3].position.z = (tweaked_bricks).cos() * 2.5;
        self.renderer_state.objects[3].rotation =
            cgmath::Quaternion::from_angle_z(cgmath::Deg(tweaked_bricks * 500.0))
                * cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks * 300.0));

        self.renderer_state.objects[4].position.x = (tweaked_bricks * 2.0).sin() * 4.0;
        self.renderer_state.objects[4].position.y = (tweaked_bricks * 3.0).sin() * 2.5;
        self.renderer_state.objects[4].position.z = (tweaked_bricks).cos() * 4.0;
        self.renderer_state.objects[4].rotation =
            cgmath::Quaternion::from_angle_z(cgmath::Deg(tweaked_bricks * 750.0))
                * cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks * 200.0));*/

        // there's a bug in the orthographic code that causes the z to affect the size of the ui
        // elements
        // self.renderer_state.ui_elements[0].position.z = tweaked_bricks.sin();
        self.renderer_state.ui_elements[0].rotation =
            cgmath::Quaternion::from_angle_y(cgmath::Deg(tweaked_bricks.sin() * 50.0));
    }

    /*pub fn handle_event(&mut self, event: &Event<()>) -> bool {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == self.renderer_state.window().id() => {
                match event {
                    WindowEvent::CloseRequested => return false,
                    WindowEvent::Resized(physical_size) => {
                        self.renderer_state.resize(*physical_size);
                    }
                    // つづ: re-evaluate whether new_inner_size (the second variable) or the 'field
                    // ignorer' should go last.
                    // okay turns out you have to put the 'field ignorer' last.
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        self.renderer_state.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        true
    }*/
    pub fn render(&mut self, instant: &Instant) -> Result<(), wgpu::SurfaceError> {
        let elapsed = instant.elapsed().as_secs_f32();
        self.renderer_state.update(elapsed);
        self.renderer_state.render()
    }
}

// this is my solution to avoiding the extensive use of '.unwrap()' for everything regarding the
// renderer and window.
// つづ: look into pollster and see if this can be simplified, although i doubt it'll need it. i
// have no idea whether it is useful, but its focus on futures makes it worth investigating a
// little.
//
// on second thought, i could just pass whatever function requires the renderer an unwrapped
// version... huh.
// still worth doing this so i have some sort of application interface.
// if stuff gets really bad within internal functions, a simple assignment like 'window =
// self.window.unwrap();' should suffice. 
struct Application {
    window: Option<Arc<Window>>,
    boot: Instant,
    game: Option<Game>,
}

impl Application {
    pub fn new(boot: Instant) -> Application {
        Application {
            window: None,
            boot,
            game: None,
        }
    }
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() && self.game.is_none() {
            self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));
            self.game = Some(Game::new(&self.window.clone().unwrap()));
        } else if self.window.is_some() | self.game.is_some() {
            // つづ: throw or something.
            println!("somehow there is either a window or a game, but not both.");
            event_loop.exit();
        }
        println!("application triggered a resume.");
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        println!("window event triggered.");
        if id != self.window.as_ref().unwrap().id() {
            println!("not this window!");
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("close requested.");
                event_loop.exit();
            },

            WindowEvent::RedrawRequested => {
                if let Some(ref mut dense_forest) = self.game {
                    dense_forest.update();
                    dense_forest.render(&self.boot);
                    println!("rendered.");
                }
            }
            _ => {
                println!("i don't know how to handle the '{:?}' event.", event);
            }
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll); // つづ: test effects without this line.

    let mut application = Application::new(Instant::now());
    if let Err(e) = event_loop.run_app(&mut application) {
        eprintln!("event loop threw: {e}");
    }
/*
    game_loop(
        event_loop,
        window,
        game,
        UPDATES_PER_SECOND,
        MAX_FRAME_TIME,
        |g| {
            g.game.update();
        },
        move |g| {
            match g.game.render(&boot) {
                Ok(_) => {}
                //Err(wgpu::SurfaceError::Lost) => g.game.renderer_state.resize(g.game.renderer_state.get_size()), // reconfigure if lost
                Err(wgpu::SurfaceError::OutOfMemory) => g.exit(),
                Err(e) => eprintln!("{:?}", e),
            }
        },
        |g, event| {
            if g.game.handle_event(event) == false {
                g.exit()
            };
        },
    );*/
}
