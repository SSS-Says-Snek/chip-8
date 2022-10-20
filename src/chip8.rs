use std::fs;

use rand::{rngs::ThreadRng, Rng};

const START_MEM_ADDR: u16 = 0x200;
const START_FONT_ADDR: u16 = 0x50;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Chip8 {
    registers: Vec<u8>,  // Stores data like mem, but it's on the CPU
    memory: Vec<u8>,     // Chip-8's memory
    index: u16,          // Stores memory addrs
    prog_counter: u16,   // Stores addr of next instruction to execute
    stack_pointer: u8,   // Stores where in the stack we are
    stack: Vec<u16>,     // Keep track of function calls
    sound_timer: u8,     // If != 0, decrements 60 times per sec
    delay_timer: u8,     // Same as sound_timer, but also emit buzz when != 0
    pub keys: Vec<bool>, // Determine if input keys are pressed or not
    pub video: Vec<u8>,  // Stores pixels to display
    opcode: u16,         // Encoded instructions

    rng: ThreadRng, // RNG,
    table: Vec<fn(&mut Self)>,
    table_0: Vec<fn(&mut Self)>,
    table_8: Vec<fn(&mut Self)>,
    table_E: Vec<fn(&mut Self)>,
    table_F: Vec<fn(&mut Self)>,
}

impl Chip8 {
    pub fn new(filename: &str) -> Chip8 {
        let mut table: Vec<fn(&mut Self)> = vec![Chip8::op_null; 0xF + 1];
        let mut table_0: Vec<fn(&mut Self)> = vec![Chip8::op_null; 0xF];
        let mut table_8: Vec<fn(&mut Self)> = vec![Chip8::op_null; 0xF];
        let mut table_E: Vec<fn(&mut Self)> = vec![Chip8::op_null; 0xF];
        let mut table_F: Vec<fn(&mut Self)> = vec![Chip8::op_null; 0x66];

        table_0[0x0] = Chip8::op_00E0;
        table_0[0xE] = Chip8::op_00EE;

        table_8[0x0] = Chip8::op_8xy0;
        table_8[0x1] = Chip8::op_8xy1;
        table_8[0x2] = Chip8::op_8xy2;
        table_8[0x3] = Chip8::op_8xy3;
        table_8[0x4] = Chip8::op_8xy4;
        table_8[0x5] = Chip8::op_8xy5;
        table_8[0x6] = Chip8::op_8xy6;
        table_8[0x7] = Chip8::op_8xy7;
        table_8[0xE] = Chip8::op_8xyE;

        table_E[0x1] = Chip8::op_ExA1;
        table_E[0xE] = Chip8::op_Ex9E;

        table_F[0x07] = Chip8::op_Fx07;
        table_F[0x0A] = Chip8::op_Fx0A;
        table_F[0x15] = Chip8::op_Fx15;
        table_F[0x18] = Chip8::op_Fx18;
        table_F[0x1E] = Chip8::op_Fx1E;
        table_F[0x29] = Chip8::op_Fx29;
        table_F[0x33] = Chip8::op_Fx33;
        table_F[0x55] = Chip8::op_Fx55;
        table_F[0x65] = Chip8::op_Fx65;

        table[0x0] = Chip8::table_0;
        table[0x1] = Chip8::op_1nnn;
        table[0x2] = Chip8::op_2nnn;
        table[0x3] = Chip8::op_3xkk;
        table[0x4] = Chip8::op_4xkk;
        table[0x5] = Chip8::op_5xy0;
        table[0x6] = Chip8::op_6xkk;
        table[0x7] = Chip8::op_7xkk;
        table[0x8] = Chip8::table_8;
        table[0x9] = Chip8::op_9xy0;
        table[0xA] = Chip8::op_Annn;
        table[0xB] = Chip8::op_Bnnn;
        table[0xC] = Chip8::op_Cxkk;
        table[0xD] = Chip8::op_Dxyn;
        table[0xE] = Chip8::table_E;
        table[0xF] = Chip8::table_F;

        let mut chip8 = Chip8 {
            registers: vec![0; 16],
            memory: vec![0; 4096],
            index: 0,
            prog_counter: START_MEM_ADDR,
            stack_pointer: 0,
            stack: vec![0; 16],
            sound_timer: 0,
            delay_timer: 0,
            keys: vec![false; 16],
            video: vec![0; 64 * 32],
            opcode: 0,
            rng: rand::thread_rng(),

            table,
            table_0,
            table_8,
            table_E,
            table_F,
        };

        chip8.load_rom(filename);
        chip8.load_fonts();

        chip8
    }

    pub fn cycle(&mut self) {
        self.opcode = (((self.memory[self.prog_counter as usize] as u16) << 8)
            | self.memory[self.prog_counter as usize + 1] as u16) as u16;
        // print!("{:4x} ", self.opcode);

        self.prog_counter += 2;

        self.table[(self.opcode >> 12) as usize](self);

        if self.delay_timer > 0 {
            self.delay_timer -= 1
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1
        }
    }

    fn load_rom(&mut self, filename: &str) {
        let data = fs::read(filename).expect("Unable to read file!");

        for i in 0..data.len() {
            self.memory[START_MEM_ADDR as usize + i] = data[i];
        }

        // println!("{:?}", self.memory);
    }

    fn load_fonts(&mut self) {
        for i in 0..FONTSET.len() {
            self.memory[START_FONT_ADDR as usize + i] = FONTSET[i];
        }
    }

    fn table_0(&mut self) {
        self.table_0[(self.opcode & 0x000F) as usize](self);
    }

    fn table_8(&mut self) {
        self.table_8[(self.opcode & 0x000F) as usize](self);
    }

    fn table_E(&mut self) {
        self.table_E[(self.opcode & 0x000F) as usize](self);
    }

    fn table_F(&mut self) {
        self.table_F[(self.opcode & 0x00FF) as usize](self);
    }

    fn op_00E0(&mut self) {
        self.video.iter_mut().for_each(|m| *m = 0);
    }

    fn op_00EE(&mut self) {
        self.stack_pointer -= 1;
        self.prog_counter = self.stack[self.stack_pointer as usize];
    }

    fn op_1nnn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.prog_counter = addr;
    }

    fn op_2nnn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.stack[self.stack_pointer as usize] = self.prog_counter;
        self.stack_pointer += 1;
        self.prog_counter = addr;
    }

    fn op_3xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        if self.registers[vx] == byte {
            self.prog_counter += 2;
        }
    }

    fn op_4xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        if self.registers[vx] != byte {
            self.prog_counter += 2;
        }
    }

    fn op_5xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] == self.registers[vy] {
            self.prog_counter += 2;
        }
    }

    fn op_6xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        self.registers[vx] = byte;
    }

    fn op_7xkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = (self.opcode & 0x00FF) as u8;

        self.registers[vx] = (self.registers[vx] as u16 + byte as u16) as u8;
    }

    fn op_8xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vy];
    }

    fn op_8xy1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] |= self.registers[vy];
        // self.registers[0xF] = 0;
    }

    fn op_8xy2(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] &= self.registers[vy];
        // self.registers[0xF] = 0;
    }

    fn op_8xy3(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] ^= self.registers[vy];
        // self.registers[0xF] = 0;
    }

    fn op_8xy4(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        let sum = (self.registers[vx] as usize + self.registers[vy] as usize) as usize;

        if sum > 255 {
            self.registers[0xF] = 1
        } else {
            self.registers[0xF] = 0
        }

        self.registers[vx] = sum as u8;
    }

    fn op_8xy5(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] > self.registers[vy] {
            self.registers[0xF] = 1
        } else {
            self.registers[0xF] = 0
        }

        self.registers[vx] = self.registers[vx].wrapping_sub(self.registers[vy]);
    }

    fn op_8xy6(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vy];

        self.registers[0xF] = self.registers[vx] & 0x1;

        self.registers[vx] >>= 1;
    }

    fn op_8xy7(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vy] > self.registers[vx] {
            self.registers[0xF] = 1
        } else {
            self.registers[0xF] = 0
        }

        self.registers[vx] = self.registers[vy].wrapping_sub(self.registers[vx]);
    }

    fn op_8xyE(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        self.registers[vx] = self.registers[vy];

        self.registers[0xF] = (self.registers[vx] & 0x80) >> 7;

        self.registers[vx] <<= 1;
    }

    fn op_9xy0(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;

        if self.registers[vx] != self.registers[vy] {
            self.prog_counter += 2;
        }
    }

    fn op_Annn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.index = addr;
    }

    fn op_Bnnn(&mut self) {
        let addr = self.opcode & 0x0FFF;

        self.prog_counter = self.registers[0x0] as u16 + addr;
    }

    fn op_Cxkk(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let byte = self.opcode & 0x00FF;

        self.registers[vx] = (self.rng.gen_range(0..=255) & byte) as u8;
    }

    fn op_Dxyn(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let vy = ((self.opcode & 0x00F0) >> 4) as usize;
        let height = (self.opcode & 0x000F) as usize;

        let (x_pos, y_pos) = ((self.registers[vx]) as usize, (self.registers[vy]) as usize);

        for row in 0..height {
            let sprite_byte = self.memory[self.index as usize + row];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0x80 >> col);
                let screen_pixel =
                    &mut self.video[((y_pos + row) % 32) * 64 + ((x_pos + col) % 64)];

                if sprite_pixel != 0 {
                    // print!("{},{} ", sprite_pixel, screen_pixel);
                    if *screen_pixel == 0xFF {
                        self.registers[0xF] = 1;
                    }

                    *screen_pixel ^= 0xFF;
                    // print!("{} ", screen_pixel);
                }
            }
        }
    }

    fn op_Ex9E(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[vx] as usize;

        if self.keys[key] {
            self.prog_counter += 2;
        }
    }

    fn op_ExA1(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let key = self.registers[vx] as usize;

        if !self.keys[key] {
            self.prog_counter += 2;
        }
    }

    fn op_Fx07(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.registers[vx] = self.delay_timer;
    }

    fn op_Fx0A(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let mut pressed = false;

        for i in 0..16 {
            if self.keys[i] {
                self.registers[vx] = i as u8;
                pressed = true;
                break;
            }
        }

        if !pressed {
            // Basically reruns the current instruction until key is pressed
            self.prog_counter -= 2;
        }
    }

    fn op_Fx15(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.delay_timer = self.registers[vx];
    }

    fn op_Fx18(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.sound_timer = self.registers[vx];
    }

    fn op_Fx1E(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        self.index += self.registers[vx] as u16;
    }

    fn op_Fx29(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let digit = self.registers[vx] as u16;

        self.index = START_FONT_ADDR + 5 * digit;
    }

    fn op_Fx33(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;
        let mut value = self.registers[vx];
        let idx = self.index as usize;

        self.memory[idx + 2] = value % 10;
        value /= 10;
        self.memory[idx + 1] = value % 10;
        value /= 10;
        self.memory[idx] = value % 10;
    }

    fn op_Fx55(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        for i in 0..=vx {
            self.memory[self.index as usize + i] = self.registers[i];
        }

        self.index += vx as u16 + 1;
    }

    fn op_Fx65(&mut self) {
        let vx = ((self.opcode & 0x0F00) >> 8) as usize;

        for i in 0..=vx {
            self.registers[i] = self.memory[self.index as usize + i];
        }

        self.index += vx as u16 + 1;
    }

    fn op_null(&mut self) {}
}
