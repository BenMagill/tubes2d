use pixels::{Error, Pixels, SurfaceTexture};
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};
use tao::dpi::LogicalSize;
use tao::event::{Event, KeyEvent, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::keyboard::KeyCode;
use tao::window::WindowBuilder;

static WIDTH: u32 = 100;
static HEIGHT: u32 = 100;

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn zero() -> Point {
        Point { x: 0.0, y: 0.0 }
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
}

impl Tube {
    fn new(speed: f32, direction: Facing, current_pos: Point, turn_chance: f32) -> Tube {
        Tube {
            speed,
            direction,
            current_pos,
            turn_chance,
            last_pos: current_pos,
            rng: thread_rng(),
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
        } else if self.current_pos.x > WIDTH as f32 && self.direction == Facing::E {
            self.direction = Facing::W;
        } else if self.current_pos.y < 1.0 && self.direction == Facing::N {
            self.direction = Facing::S;
        } else if self.current_pos.y > HEIGHT as f32 && self.direction == Facing::S {
            self.direction = Facing::N;
        }
    }

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
}

fn main() -> Result<(), Error> {
    // Idea
    // Each tube will move around randomly
    //  they will each have different speeds and chance of turning
    //  colour will be random
    //
    // The direction of the tube and current coordinates need to be remembered, as well as start
    // and each edge point so the line cant be drawn
    //
    // IDK
    //  how to allow the tubes to go above and below smoothly
    //  how indepth to go: do i just work on generating what the image would be, or the rendering
    //  as well

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(100, 100, surface_texture)?
    };

    let mut tube = Tube::new(0.7, Facing::E, Point::zero(), 0.06);

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
                //if let Err(err) = pixels.resize_buffer(size.width, size.height) {
                //println!("Can't resize");
                //*control_flow = ControlFlow::Exit;
                //}
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    println!("Couldnt resize");
                    *control_flow = ControlFlow::Exit;
                }
            }

            _ => {}
        },

        Event::MainEventsCleared => {
            tube.incrememnt();
            window.request_redraw();
        }

        Event::RedrawRequested(_) => {
            // Test draw code
            let frame = pixels.frame_mut();

            //let diff = tube.last_pos - tube.current_pos;
            //if diff.x == 0.0 {
            //} else {
            //}
            // TODO: replace with code actually drawing a line, as processing every pixel is slow
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
                let x = i % 100;
                let y = i / 100;

                let x1 = tube.last_pos.x as usize;
                let y1 = tube.last_pos.y as usize;
                let x2 = tube.current_pos.x as usize;
                let y2 = tube.current_pos.y as usize;

                let on_line = (x1 <= x && x <= x2 && y1 <= y && y <= y2)
                    || (x1 >= x && x >= x2 && y1 >= y && y >= y2);

                if on_line {
                    pixel.copy_from_slice(&[0x48, 0xb2, 0xe8, 0xff]);
                }
            }

            if let Err(err) = pixels.render() {
                println!("render error");
                *control_flow = ControlFlow::Exit;
            }
        }

        _ => {}
    });
}
