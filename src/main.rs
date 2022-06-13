use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;

#[derive(Copy, Clone, Debug)]
enum Block {
    EMPTY,
    PLAYER { color: u32 },
    WALL { color: u32 },
    FLOOR { color: u32 },
}

struct Game {
    player: Player,
    playground: Playground,
}

impl Game {
    fn new() -> Self {
        let definition = Game::read_definition();
        Game {
            player: definition.0,
            playground: definition.1,
        }
    }

    fn read_definition() -> (Player, Playground) {
        let contents = std::fs::read_to_string("map.txt")
            .expect("Unable to read map");
        let mut width = 0;
        let mut index = 0;
        let mut schema = Vec::new();
        let mut count_width = true;
        let mut player_index = 0;
        for code in contents.chars() {
            let block = match code {
                '_' => { Some(Block::EMPTY) }
                '%' => { Some(Block::FLOOR { color: compose_color(255, 0, 0) }) }
                '|' => { Some(Block::WALL { color: compose_color(0, 0, 255) }) }
                '@' => {
                    player_index = index;
                    Some(Block::FLOOR { color: compose_color(255, 0, 0) })
                }
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
        let playground = Playground::new(schema, index / width, width);

        let player = Player {
            position_y: player_index / playground.height,
            position_x: player_index / width,
        };
        (player, playground)
    }

    fn handle_key_press(self: &mut Self, keycode: Keycode) {
        match keycode {
            Keycode::A => { self.player.position_x = self.player.position_x + 1 }
            Keycode::D => { self.player.position_x = self.player.position_x - 1 }
            Keycode::Space => { self.player.position_y = self.player.position_y + 1 }
            _ => {}
        }
    }

    fn tick(self: &Self) {}

    fn render(self: &Self, canvas: &mut WindowCanvas) {
        let canvas_size = canvas.output_size()
            .expect("Unable to extract canvas size");
        let scale = self.playground.scale_factor(canvas_size);
        self.render_playground(&self.playground, canvas, scale);
        self.render_player(&self.player, canvas, scale);
    }

    fn render_playground(self: &Self, playground: &Playground, canvas: &mut WindowCanvas, scale: (u32, u32)) {
        for y in 0..playground.height {
            for x in 0..playground.width {
                let block = playground.block_at(x, y);
                let color = match block {
                    Block::WALL { color } => { Some(color) }
                    Block::FLOOR { color } => { Some(color) }
                    Block::PLAYER { .. } => { None }
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

    fn render_player(self: &Self, player: &Player, canvas: &mut WindowCanvas, scale: (u32, u32)) {
        canvas.set_draw_color(Color::GREEN);
        let rect = Rect::new(
            (player.position_x as u32 * scale.0) as i32,
            (player.position_y as u32 * scale.1 + scale.1) as i32,
            scale.0,
            scale.1,
        );
        canvas.fill_rect(rect).unwrap();
        canvas.draw_rect(rect).unwrap();
    }
}

struct Player {
    position_x: usize,
    position_y: usize,
}

struct Playground {
    schema: Vec<Block>,
    height: usize,
    width: usize,
}

impl Playground {
    fn new(schema: Vec<Block>, height: usize, width: usize) -> Self {
        Playground {
            schema,
            height,
            width,
        }
    }

    fn block_at(self: &Self, x: usize, y: usize) -> &Block {
        &self.schema[y * self.width + x]
    }

    fn scale_factor(self: &Self, size: (u32, u32)) -> (u32, u32) {
        let dh = size.0 / self.width as u32;
        let dw = size.1 / self.height as u32;
        (dh, dw)
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

    let mut game = Game::new();

    while running {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => { running = false }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { running = false }
                Event::KeyDown { keycode, .. } => {
                    if keycode.is_some() {
                        game.handle_key_press(keycode.unwrap());
                    }
                }
                _ => {}
            }
        }
        game.tick();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        game.render(&mut canvas);
        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(1000 / 60));
    }
}
