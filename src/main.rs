extern crate sdl2;

mod cpu;
mod display;
mod keypad;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let display = display::Display::new(video_subsystem);
    let chip_8 = cpu::CPU::new(display);
}
