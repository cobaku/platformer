use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

#[derive(Copy, Clone, Debug)]
enum Block {
    EMPTY,
    WALL { color: u32 },
    FLOOR { color: u32 },
}

struct Playground {
    schema: Vec<Block>,
    height: usize,
    width: usize,
}

impl Playground {
    fn new(height: usize, width: usize) -> Self {
        let mut schema = Vec::new();
        schema.resize(height * width, Block::EMPTY);
        Playground {
            schema,
            height,
            width,
        }
    }

    fn read() -> Self {
        let contents = std::fs::read_to_string("map.txt")
            .expect("Unable to read map");
        let mut width = 0;
        let mut index = 0;
        let mut schema = Vec::new();
        let mut count_width = true;
        for code in contents.chars() {
            let block = match code {
                '_' => { Some(Block::EMPTY) }
                '%' => { Some(Block::FLOOR { color: compose_color(255, 0, 0) }) }
                '|' => { Some(Block::WALL { color: compose_color(0, 0, 255) }) }
                '\n' => {
                    if count_width {
                        width = index;
                        count_width = false;
                    }
                    None
                }
                _ => { None }
            };
            index = index + 1;
            if block.is_some() {
                schema.push(block.unwrap());
            }
        }
        Playground {
            schema,
            height: (index / width),
            width,
        }
    }

    fn tick(self: &Self) {}

    fn block_at(self: &Self, x: usize, y: usize) -> &Block {
        &self.schema[y * self.width + x]
    }

    fn scale_factor(self: &Self, size: (u32, u32)) -> (u32, u32) {
        let dh = size.0 / self.width as u32;
        let dw = size.1 / self.height as u32;
        (dh, dw)
    }
}

fn render(canvas: &mut WindowCanvas, playground: &Playground) {
    let canvas_size = canvas.output_size()
        .expect("Unable to extract canvas size");
    let scale = playground.scale_factor(canvas_size);
    for y in 0..playground.height {
        for x in 0..playground.width {
            let block = playground.block_at(x, y);
            let color = match block {
                Block::WALL { color } => {
                    Some(color)
                }
                Block::FLOOR { color } => {
                    Some(color)
                }
                Block::EMPTY => { None }
            };
            if color.is_none() {
                continue;
            }
            let actual_color = color.unwrap();
            let split = split_rgb(*actual_color);
            let sdl_color = Color::from(split);
            canvas.set_draw_color(sdl_color);
            let rect = Rect::new(
                (x as u32 * scale.0) as i32,
                (y as u32 * scale.1) as i32,
                scale.0,
                scale.1,
            );
            canvas.fill_rect(rect).unwrap();
            canvas.draw_rect(rect).unwrap();
        }
    }
}

fn split_rgb(color: u32) -> (u8, u8, u8) {
    (((color >> 8 * 2) & 0xFF) as u8,
     ((color >> 8 * 1) & 0xFF) as u8,
     ((color >> 8 * 0) & 0xFF) as u8)
}

fn compose_color(r: u32, g: u32, b: u32) -> u32 {
    let mut rgb = r;
    rgb = (rgb << 8) + g;
    rgb = (rgb << 8) + b;
    rgb as u32
}

fn main() {
    const WINDOW_HEIGHT: usize = 600;
    const WINDOW_WIDTH: usize = 800;

    let sdl_context = sdl2::init()
        .expect("Unable to init SDL");
    let video = sdl_context.video()
        .expect("Unable to init SDL video subsystem");
    let window = video.window(
        &"Dummy platformer on Rust",
        WINDOW_WIDTH as u32,
        WINDOW_HEIGHT as u32,
    )
        .position_centered()
        .build()
        .expect("Unable to create window for application");

    let mut running = true;

    let mut events = sdl_context.event_pump()
        .expect("Unable to extract SDL event listener");

    let mut canvas = window.into_canvas()
        .present_vsync()
        .accelerated()
        .build()
        .expect("Unable to create canvas");

    let playground = Playground::read();

    while running {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => { running = false }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { running = false }
                _ => {}
            }
        }
        playground.tick();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        render(&mut canvas, &playground);
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
}
