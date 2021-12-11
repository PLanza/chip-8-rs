use rand;

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
    //display,
    //keypad,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            ram: [0; 4096],
            pc: 0,
            i: 0,
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            registers: [0; 16],
            opcode: 0,
        }
    }

    pub fn execute_cycle(&mut self) {
        self.opcode = (self.ram[self.pc] << 8) as u16 | self.ram[self.pc + 1] as u16;
        self.pc += 2;

        // Decode
        self.execute();
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

    // Clear screen
    fn CLS(&mut self) {
        // clear display
    }

    // Return
    fn RET(&mut self) {
        self.sp -= 1;
        self.pc = (self.stack[self.sp]) as usize;
    }

    // Jump
    fn JP(&mut self) {
        let addr = (self.opcode & 0x0FFF) as usize;
        self.pc = addr;
    }

    // Call
    fn CALL(&mut self) {
        let addr = (self.opcode & 0x0FFF) as usize;
        self.stack[self.sp] = self.pc as u16;
        self.sp += 1;
        self.pc = addr
    }

    // Skip if equal to immediate
    fn SEI(&mut self) {
        let x = (self.opcode & 0x0F00) as usize;
        let imm = (self.opcode & 0x00FF) as u8;

        if self.registers[x] == imm {
            self.pc += 2;
        }
    }

    // Skip if not equal to immediate
    fn SNEI(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let imm = (self.opcode & 0x00FF) as u8;

        if self.registers[x] != imm {
            self.pc += 2;
        }
    }

    // Skip if equal registers
    fn SER(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        if self.registers[x] == self.registers[y] {
            self.pc += 2;
        }
    }

    // Load immediate
    fn LDI(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let imm = (self.opcode & 0x00FF) as u8;

        self.registers[x] = imm;
    }

    // Add immediate
    fn ADDI(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let imm = self.opcode & 0x00FF;

        let tmp: u16 = self.registers[x] as u16 + imm;
        self.registers[x] = (tmp & 0x00FF) as u8;
    }

    // Load register
    fn LDR(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        self.registers[x] = self.registers[y];
    }

    // OR registers
    fn OR(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        self.registers[x] |= self.registers[y];
    }

    // AND registers
    fn AND(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        self.registers[x] &= self.registers[y];
    }

    // XOR registers
    fn XOR(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        self.registers[x] ^= self.registers[y];
    }

    // Add registers
    fn ADDR(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        let tmp: u16 = self.registers[x] as u16 + self.registers[y] as u16;
        self.registers[x] = (tmp & 0x00FF) as u8;
        self.registers[0xF] = (tmp & 0x0F00 >> 8) as u8; // Carry bit
    }

    // Subtract registers x - y
    fn SUB(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        // NOT borrow
        if self.registers[x] > self.registers[y] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        };

        self.registers[x] -= self.registers[y];
    }

    // Shift register x right 1 bit storing carry
    fn SHR(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let _y = (self.opcode & 0x00F0 >> 4) as usize; // Ignored

        self.registers[0xF] = self.registers[x] & 0x01;
        self.registers[x] >>= 1;
    }

    // Subtract registers y - x
    fn SUBN(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        // NOT borrow
        if self.registers[y] > self.registers[x] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[x] = self.registers[y] - self.registers[x];
    }

    // Shift register x left 1 bit storing carry
    fn SHL(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let _y = (self.opcode & 0x00F0 >> 4) as usize; // Ignored

        self.registers[0xF] = self.registers[x] >> 7;
        self.registers[x] <<= 1;
    }

    // Skip if not equal registers
    fn SNER(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let y = (self.opcode & 0x00F0 >> 4) as usize;

        if self.registers[x] != self.registers[y] {
            self.pc += 2;
        }
    }

    // Load into index register
    fn LDIN(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.i = addr;
    }

    // Jump to address plus the value of V0
    fn JPA(&mut self) {
        let addr = (self.opcode & 0x0FFF) as usize;
        self.pc = addr + self.registers[0] as usize;
    }

    // Genereate random number from 0 to 255, AND it with last two bits
    fn RND(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let imm = (self.opcode & 0x00FF) as u8;

        self.registers[x] = rand::random::<u8>() & imm;
    }

    // Draw sprite
    fn DRW(&mut self) {}

    // Skip next instruction if key pressed
    fn SKP(&mut self) {}

    // Skip next instruction if key not pressed
    fn SKNP(&mut self) {}

    // Load delay timer to register
    fn LDDT(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        self.registers[x] = self.delay_timer
    }

    // Store key value to register
    fn STK(&mut self) {}

    // Set delay timer to register's value
    fn SDT(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        self.delay_timer = self.registers[x];
    }

    // Set sound timer to register's value
    fn SST(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        self.sound_timer = self.registers[x];
    }

    // Add index register and another
    fn ADDIN(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        self.i += self.registers[x] as u16;
    }

    // Store memory location of font into index
    fn SFT(&mut self) {}

    // Store the binary-coded decimal conversion from register into memory
    fn BCD(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;
        let val = self.registers[x];

        self.ram[self.i as usize] = val / 100;
        self.ram[(self.i + 1) as usize] = (val % 100) / 10;
        self.ram[(self.i + 2) as usize] = (val % 100) % 10;
    }

    // Store registers 0..x into memory
    fn ST(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;

        for i in (0..=x) {
            self.ram[self.i as usize + i] = self.registers[i];
        }
    }

    // Load memory into registers 0..x
    fn LD(&mut self) {
        let x = (self.opcode & 0x0F00 >> 8) as usize;

        for i in (0..=x) {
            self.registers[i] = self.ram[self.i as usize + i];
        }
    }

    fn illegal_op(&mut self) {
        println!("The opcode `{}` is not a legal instruction.", self.opcode);
        self.pc += 2;
    }
}

static FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
];
