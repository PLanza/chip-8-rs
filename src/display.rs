use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::VideoSubsystem;

pub struct Display {
    screen: [[bool; 64]; 32],
    canvas: Canvas<sdl2::video::Window>,
}

impl Display {
    pub fn new(video_subsystem: VideoSubsystem) -> Display {
        let window = video_subsystem.window("Test", 640, 320).build().unwrap();
        let mut canvas = window.into_canvas().build().unwrap();

        canvas
            .set_logical_size(64, 32)
            .expect("Error setting canvas logical size");
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Display {
            screen: [[false; 64]; 32],
            canvas,
        }
    }

    pub fn clear(&mut self) {
        self.screen = [[false; 64]; 32];
        self.update_screen();
    }

    pub fn draw_sprite(&mut self, x: usize, y: usize, sprite: &[u8]) -> bool {
        let mut collision = false;
        'rows: for (i, byte) in sprite.iter().enumerate() {
            if i + y > 32 {
                break 'rows;
            }

            'columns: for j in 0..8 {
                if j + x > 64 {
                    break 'columns;
                }

                if ((byte >> (7 - j)) & 1) != 0 {
                    collision |= self.screen[i + y][j + x];
                    self.screen[i + y][j + x] ^= true;
                }
            }
        }

        self.update_screen();
        collision
    }

    fn update_screen(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.canvas.set_draw_color(if self.screen[y][x] {
                    Color::RGB(255, 255, 255)
                } else {
                    Color::RGB(0, 0, 0)
                });

                self.canvas
                    .draw_point(sdl2::rect::Point::new(x as i32, y as i32))
                    .expect("Error drawing point");
            }
        }

        self.canvas.present();
    }
}
