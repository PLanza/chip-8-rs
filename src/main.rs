extern crate sdl2;

mod cpu;
mod display;
mod keypad;

use sdl2::event::Event;
use std::env;
use std::fs::File;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let display = display::Display::new(video_subsystem);
    let mut event_pump = sdl.event_pump().unwrap();

    let mut chip_8 = cpu::CPU::new(display);

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing Argument: Path to program");
        return;
    } else if args.len() > 2 {
        println!("Too many arguments");
        return;
    }

    match File::open(&args[1]) {
        Ok(file) => chip_8.load_program(file),
        Err(e) => {
            println!("Encountered error: {:?}", e);
            return;
        }
    }

    'main: loop {
        // inner loop to catch multiple simultaneous key presses
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown { keycode: key, .. } => match key {
                    Some(sdl2::keyboard::Keycode::Space) => {
                        chip_8.print_registers();
                        chip_8.execute_cycle();
                    }
                    key => chip_8.keypad.press(key.unwrap(), true),
                },
                Event::KeyUp { keycode: key, .. } => chip_8.keypad.press(key.unwrap(), false),
                _ => {}
            }
        }
    }
}
