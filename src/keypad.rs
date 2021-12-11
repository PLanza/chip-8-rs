use sdl2::keyboard::Keycode;

pub struct Keypad {
    keys: [bool; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad { keys: [false; 16] }
    }

    pub fn is_pressed(&mut self, key: usize) -> bool {
        self.keys[key]
    }

    pub fn press(&mut self, key: Keycode, state: bool) {
        match key {
            Keycode::Num1 => self.keys[0x1] = state,
            Keycode::Num2 => self.keys[0x2] = state,
            Keycode::Num3 => self.keys[0x3] = state,
            Keycode::Num4 => self.keys[0xC] = state,
            Keycode::Q => self.keys[0x4] = state,
            Keycode::W => self.keys[0x5] = state,
            Keycode::E => self.keys[0x6] = state,
            Keycode::R => self.keys[0xD] = state,
            Keycode::A => self.keys[0x7] = state,
            Keycode::S => self.keys[0x8] = state,
            Keycode::D => self.keys[0x9] = state,
            Keycode::F => self.keys[0xE] = state,
            Keycode::Z => self.keys[0xA] = state,
            Keycode::X => self.keys[0x0] = state,
            Keycode::C => self.keys[0xB] = state,
            Keycode::V => self.keys[0xF] = state,
            _ => {}
        }
    }

    pub fn wait_for_keypress(&mut self) -> usize {
        'spin: loop {
            for i in 0..16 {
                if self.keys[i] {
                    break 'spin i;
                }
            }
        }
    }
}
