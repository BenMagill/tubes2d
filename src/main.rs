use std::cell::RefCell;

use pixels::{Error, Pixels, SurfaceTexture};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use tao::event::{Event, KeyEvent, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::keyboard::KeyCode;
use tao::window::WindowBuilder;

static WIDTH: usize = 100;
static HEIGHT: usize = 100;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn zero() -> Point {
        Point { x: 0.0, y: 0.0 }
    }

    fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }
}

impl std::ops::Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Self::Output {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Facing {
    N,
    S,
    E,
    W,
}

#[derive(Copy, Clone)]
enum Rotate {
    L,
    R,
}
fn rotate(facing: Facing, direction: Rotate) -> Facing {
    match direction {
        Rotate::L => match facing {
            Facing::N => Facing::W,
            Facing::E => Facing::N,
            Facing::S => Facing::E,
            Facing::W => Facing::S,
        },
        Rotate::R => match facing {
            Facing::N => Facing::E,
            Facing::E => Facing::S,
            Facing::S => Facing::W,
            Facing::W => Facing::N,
        },
    }
}

struct Tube {
    speed: f32,
    direction: Facing,
    current_pos: Point,
    last_pos: Point,
    turn_chance: f32,
    rng: ThreadRng,
    colour: [u8; 4],
}

impl Tube {
    fn new(
        speed: f32,
        direction: Facing,
        current_pos: Point,
        turn_chance: f32,
        colour: [u8; 4],
    ) -> Tube {
        Tube {
            speed,
            direction,
            current_pos,
            turn_chance,
            last_pos: current_pos,
            rng: thread_rng(),
            colour,
        }
    }
    fn move_forward(&mut self) {
        match self.direction {
            Facing::N => self.current_pos.y = self.current_pos.y - self.speed,
            Facing::S => self.current_pos.y = self.current_pos.y + self.speed,
            Facing::E => self.current_pos.x = self.current_pos.x + self.speed,
            Facing::W => self.current_pos.x = self.current_pos.x - self.speed,
        };
    }

    fn incrememnt(&mut self) {
        self.last_pos = self.current_pos;

        self.move_forward();

        self.attempt_turn();

        self.fix_out_of_bounds();
    }

    fn fix_out_of_bounds(&mut self) {
        if self.current_pos.x < 1.0 && self.direction == Facing::W {
            self.direction = Facing::E;
        } else if self.current_pos.x > (WIDTH - 1) as f32 && self.direction == Facing::E {
            self.direction = Facing::W;
        } else if self.current_pos.y < 1.0 && self.direction == Facing::N {
            self.direction = Facing::S;
        } else if self.current_pos.y > HEIGHT as f32 && self.direction == Facing::S {
            self.direction = Facing::N;
        }
    }

    // If hits edge, turn it the other way so doesn't go out
    fn attempt_turn(&mut self) {
        let n: f32 = self.rng.gen();
        if n < self.turn_chance {
            self.direction = rotate(
                self.direction,
                if self.rng.gen::<bool>() {
                    Rotate::L
                } else {
                    Rotate::R
                },
            );
        }
    }

    fn draw(&mut self, frame: &mut [u8]) {
        // With the assumption velocity always < 1, dont need to draw a line just a single
        // pixel per movement
        let x = self.current_pos.x as usize;
        let y = self.current_pos.y as usize;
        let i = (y * HEIGHT) + x;

        // Ignore out of bounds issues
        if (i * 4) < WIDTH * HEIGHT * 4 {
            frame[(i * 4)..(i * 4) + 4].copy_from_slice(&self.colour);
        }
    }
}

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_resizable(false)
        .with_title("Tubes2d")
        .build(&event_loop)
        .unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut tubes = vec![];
    tubes.push(RefCell::new(Tube::new(
        0.7,
        Facing::E,
        Point::zero(),
        0.06,
        [0x48, 0xb2, 0xe8, 0xff],
    )));
    tubes.push(RefCell::new(Tube::new(
        0.5,
        Facing::E,
        Point::new(WIDTH as f32, HEIGHT as f32),
        0.03,
        [0x5e, 0x48, 0xe8, 0xff],
    )));
    tubes.push(RefCell::new(Tube::new(
        0.2,
        Facing::E,
        Point::new(0.0, HEIGHT as f32),
        0.01,
        [0x57, 0xEB, 0xB3, 0xff],
    )));
    //tubes.push(RefCell::new(Tube::new(
    //1.0,
    //Facing::E,
    //Point::new(100.0, 50.0),
    //0.04,
    //[0xC2, 0x46, 0xE3, 0xff],
    //)));

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: KeyCode::Escape,
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            WindowEvent::Resized(size) => {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    println!("Couldnt resize");
                    *control_flow = ControlFlow::Exit;
                }
            }

            _ => {}
        },

        Event::MainEventsCleared => {
            for tube in &tubes {
                tube.borrow_mut().incrememnt();
            }
            window.request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Test draw code
            let frame = pixels.frame_mut();

            for tube in &tubes {
                let mut tube = tube.borrow_mut();
                tube.draw(frame);
            }
            if let Err(err) = pixels.render() {
                println!("render error");
                *control_flow = ControlFlow::Exit;
            }
        }

        _ => {}
    });
}
