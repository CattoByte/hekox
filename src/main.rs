mod renderer;
use std::{sync::Arc, time::Instant};

use cgmath::num_traits::ToPrimitive;
use game_loop::game_loop;
use winit::{
    event::{self, Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

const UPDATES_PER_SECOND: u32 = 96;
const MAX_FRAME_TIME: f64 = 0.1;

struct Game {
    renderer_state: renderer::State,
    do_bricks_have_an_inherent_colour: u64,
}

impl Game {
    pub fn update(&mut self) {
        self.do_bricks_have_an_inherent_colour += 1;
    }

    pub fn handle_event(&mut self, event: &Event<()>) -> bool {
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
    }
    pub fn render(&mut self, instant: &Instant) -> Result<(), wgpu::SurfaceError> {
        let elapsed = instant.elapsed().as_secs_f32();
        self.renderer_state.update(elapsed);
        self.renderer_state.render()
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let window = Arc::new(window);
    let boot = Instant::now();

    let game = Game {
        renderer_state: pollster::block_on(renderer::State::new(window.clone())),
        do_bricks_have_an_inherent_colour: 0,
    };

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
    );
}
