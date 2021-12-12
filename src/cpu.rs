#![allow(non_snake_case)]

use crate::display;
use crate::keypad;
use rand;
use std::fs::File;

pub struct CPU {
    ram: [u8; 4096],
    pc: usize,
    i: u16,
    stack: [u16; 16],
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    registers: [u8; 16],
    opcode: u16,
    pub keypad: keypad::Keypad,
    display: display::Display,
}

static FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];

impl CPU {
    pub fn new(display: display::Display) -> CPU {
        let mut cpu = CPU {
            ram: [0; 4096],
            pc: 0x200,
            i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            opcode: 0,
            keypad: keypad::Keypad::new(),
            display,
        };

        // load font
        for i in 0..80 {
            cpu.ram[i + 0x50] = FONTSET[i];
        }

        cpu
    }

    pub fn load_program(&mut self, file: File) {
        let reader = std::io::Read::bytes(file);

        for (i, byte) in reader.enumerate() {
            match byte {
                Ok(value) => self.ram[0x200 + i] = value,
                Err(_) => {}
            }
            //print!("({:x}: {:x}) ", 0x200 + i, self.ram[0x200 + i]);
        }
    }

    pub fn execute_cycle(&mut self) {
        self.opcode = (self.ram[self.pc] as u16) << 8 | self.ram[self.pc + 1] as u16;
        println!("{:x} - {:x}", self.pc, self.opcode);
        self.pc += 2;

        // Decode
        self.execute();

        // Apparently this should only be done 60 times per second
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            // play sound
            self.sound_timer -= 1;
        }
    }

    #[allow(dead_code)]
    pub fn print_registers(&self) {
        for i in 0..self.registers.len() {
            print!("V{}: {:x} ", i, self.registers[i]);
        }
        println!(
            "\ni: {:x}, delay timer: {:x}, sound timer: {:x}\n",
            self.i, self.delay_timer, self.sound_timer
        );
    }

    fn execute(&mut self) {
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode {
                    0x00E0 => self.CLS(),
                    0x00EE => self.RET(),
                    _ => self.illegal_op(),
                };
            }
            0x1000 => self.JP(),
            0x2000 => self.CALL(),
            0x3000 => self.SEI(),
            0x4000 => self.SNEI(),
            0x5000 => self.SER(),
            0x6000 => self.LDI(),
            0x7000 => self.ADDI(),
            0x8000 => match self.opcode & 0x000F {
                0x0 => self.LDR(),
                0x1 => self.OR(),
                0x2 => self.AND(),
                0x3 => self.XOR(),
                0x4 => self.ADDR(),
                0x5 => self.SUB(),
                0x6 => self.SHR(),
                0x7 => self.SUBN(),
                0xE => self.SHL(),
                _ => self.illegal_op(),
            },
            0x9000 => {
                if self.opcode & 0x000F == 0 {
                    self.SNER()
                } else {
                    self.illegal_op()
                }
            }
            0xA000 => self.LDIN(),
            0xB000 => self.JPA(),
            0xC000 => self.RND(),
            0xD000 => self.DRW(),
            0xE000 => match self.opcode & 0x00FF {
                0x9E => self.SKP(),
                0xA1 => self.SKNP(),
                _ => self.illegal_op(),
            },
            0xF000 => match self.opcode & 0x00FF {
                0x07 => self.LDDT(),
                0x0A => self.STK(),
                0x15 => self.SDT(),
                0x18 => self.SST(),
                0x1E => self.ADDIN(),
                0x29 => self.SFT(),
                0x33 => self.BCD(),
                0x55 => self.ST(),
                0x65 => self.LD(),
                _ => self.illegal_op(),
            },
            _ => self.illegal_op(),
        }
    }

    fn x(&mut self) -> usize {
        (self.opcode as usize & 0x0F00) >> 8
    }
    fn y(&mut self) -> usize {
        (self.opcode as usize & 0x00F0) >> 4
    }
    fn imm(&mut self) -> u8 {
        (self.opcode & 0x00FF) as u8
    }
    fn addr(&mut self) -> usize {
        (self.opcode & 0x0FFF) as usize
    }

    // Clear screen
    fn CLS(&mut self) {
        self.display.clear();
    }

    // Return
    fn RET(&mut self) {
        self.sp -= 1;
        self.pc = (self.stack[self.sp]) as usize;
    }

    // Jump
    fn JP(&mut self) {
        self.pc = self.addr();
    }

    // Call
    fn CALL(&mut self) {
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = self.addr();
    }

    // Skip if equal to immediate
    fn SEI(&mut self) {
        if self.registers[self.x()] == self.imm() {
            self.pc += 2;
        }
    }

    // Skip if not equal to immediate
    fn SNEI(&mut self) {
        if self.registers[self.x()] != self.imm() {
            self.pc += 2;
        }
    }

    // Skip if equal registers
    fn SER(&mut self) {
        if self.registers[self.x()] == self.registers[self.y()] {
            self.pc += 2;
        }
    }

    // Load immediate
    fn LDI(&mut self) {
        self.registers[self.x()] = self.imm();
    }

    // Add immediate
    fn ADDI(&mut self) {
        let tmp = self.registers[self.x()] as u16 + self.imm() as u16;
        self.registers[self.x()] = (tmp & 0x00FF) as u8;
    }

    // Load register
    fn LDR(&mut self) {
        self.registers[self.x()] = self.registers[self.y()];
    }

    // OR registers
    fn OR(&mut self) {
        self.registers[self.x()] |= self.registers[self.y()];
    }

    // AND registers
    fn AND(&mut self) {
        self.registers[self.x()] &= self.registers[self.y()];
    }

    // XOR registers
    fn XOR(&mut self) {
        self.registers[self.x()] ^= self.registers[self.y()];
    }

    // Add registers
    fn ADDR(&mut self) {
        let tmp = self.registers[self.x()] as u16 + self.registers[self.y()] as u16;

        self.registers[self.x()] = (tmp & 0x00FF) as u8;
        self.registers[0xF] = ((tmp & 0x0F00) >> 8) as u8; // Carry bit
    }

    // Subtract registers x - y
    fn SUB(&mut self) {
        // NOT borrow
        if self.registers[self.x()] > self.registers[self.y()] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        };

        self.registers[self.x()] =
            (self.registers[self.x()] as i16 - self.registers[self.y()] as i16) as u8;
    }

    // Shift register x right 1 bit storing carry
    fn SHR(&mut self) {
        self.registers[0xF] = self.registers[self.x()] & 0x01;
        self.registers[self.x()] >>= 1;
    }

    // Subtract registers y - x
    fn SUBN(&mut self) {
        // NOT borrow
        if self.registers[self.y()] > self.registers[self.x()] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[self.x()] =
            (self.registers[self.y()] as i16 - self.registers[self.x()] as i16) as u8;
    }

    // Shift register x left 1 bit storing carry
    fn SHL(&mut self) {
        self.registers[0xF] = self.registers[self.x()] >> 7;
        self.registers[self.x()] <<= 1;
    }

    // Skip if not equal registers
    fn SNER(&mut self) {
        if self.registers[self.x()] != self.registers[self.y()] {
            self.pc += 2;
        }
    }

    // Load into index register
    fn LDIN(&mut self) {
        self.i = self.addr() as u16;
    }

    // Jump to address plus the value of V0
    fn JPA(&mut self) {
        self.pc = self.addr() + self.registers[0] as usize;
    }

    // Genereate random number from 0 to 255, AND it with last two bits
    fn RND(&mut self) {
        self.registers[self.x()] = rand::random::<u8>() & self.imm();
    }

    // Draw sprite
    fn DRW(&mut self) {
        let (from, to) = (self.i as usize, (self.i + (self.opcode & 0x000F)) as usize);
        let (x, y) = (self.registers[self.x()] % 64, self.registers[self.y()] % 32);

        let sprite = &self.ram[from..to];

        self.registers[0xF] = self.display.draw_sprite(x as usize, y as usize, sprite) as u8;
    }

    // Skip next instruction if key pressed
    fn SKP(&mut self) {
        let key = self.x();
        if self.keypad.is_pressed(key) {
            self.pc += 2;
        }
    }

    // Skip next instruction if key not pressed
    fn SKNP(&mut self) {
        let key = self.x();
        if !self.keypad.is_pressed(key) {
            self.pc += 2;
        }
    }

    // Load delay timer to register
    fn LDDT(&mut self) {
        self.registers[self.x()] = self.delay_timer
    }

    // Blocking store pressed key value to register
    fn STK(&mut self) {
        match self.keypad.wait_for_keypress() {
            Some(val) => self.registers[self.x()] = val as u8,
            None => self.pc -= 2,
        }
    }

    // Set delay timer to register's value
    fn SDT(&mut self) {
        self.delay_timer = self.registers[self.x()];
    }

    // Set sound timer to register's value
    fn SST(&mut self) {
        self.sound_timer = self.registers[self.x()];
    }

    // Add index register and another
    fn ADDIN(&mut self) {
        self.i += self.registers[self.x()] as u16;
    }

    // Store memory location of font into index
    fn SFT(&mut self) {
        self.i = (0x50 + self.registers[self.x()] * 5) as u16;
    }

    // Store the binary-coded decimal conversion from register into memory
    fn BCD(&mut self) {
        let val = self.registers[self.x()];

        self.ram[self.i as usize] = val / 100;
        self.ram[(self.i + 1) as usize] = (val % 100) / 10;
        self.ram[(self.i + 2) as usize] = (val % 100) % 10;
    }

    // Store registers 0..x into memory
    fn ST(&mut self) {
        for i in 0..=self.x() {
            self.ram[self.i as usize + i] = self.registers[i];
        }
    }

    // Load memory into registers 0..x
    fn LD(&mut self) {
        for i in 0..=self.x() {
            self.registers[i] = self.ram[self.i as usize + i];
        }
    }

    fn illegal_op(&mut self) {
        println!("The opcode `{}` is not a legal instruction.", self.opcode);
        self.pc += 2;
    }
}
