use std::fs;
use std::error::Error;
use crate::helpers::as_index;

const FONT_SPRITES: [u8; 80]  = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0 
    0x20, 0x60, 0x20, 0x20, 0x70, //1 
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3 
    0x90, 0x90, 0xF0, 0x10, 0x10, //4 
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5 
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6 
    0xF0, 0x10, 0x20, 0x10, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B 
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C 
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80, //F
];

const SCREEN_SIZE: usize = 64 * 32;
const MEM_SIZE:usize = 4096;
const REG_SIZE:usize = 16;
const START_INDEX:u16 = 0x200;

pub struct Emulator {
    stack: Vec<u16>,
    memory: [u8; MEM_SIZE],
    register: [u8; REG_SIZE],
    screen: [bool; SCREEN_SIZE],
    pc: u16, 
    index: u16, 
    dtimer: u8,
    stimer: u8,
    draw: bool,
}

impl Emulator {
    pub fn new() -> Self {
        let mut emu = Self {
            memory: [0; MEM_SIZE] ,
            screen: [false; SCREEN_SIZE],
            register: [0; REG_SIZE],
            index: 0,
            pc: START_INDEX,
            stack: Vec::new(),
            dtimer: 0,
            stimer: 0,
            draw: false,
        };

        // Loading the font sprites into memory
        for i in 0..FONT_SPRITES.len(){
            emu.memory[i] = FONT_SPRITES[i];
        }

        emu
    }
}

impl Emulator {
    pub fn draw(&self) -> bool {
        self.draw
    }

    pub fn display(&self) -> [bool; SCREEN_SIZE] {
        self.screen 
    }

    pub fn reset(&mut self) {
        self.memory = [0; MEM_SIZE];
        self.screen = [false; SCREEN_SIZE];
        self.register = [0; REG_SIZE];
        self.index = 0;
        self.pc = START_INDEX;
        self.stack = Vec::new();
        self.dtimer = 0;
        self.stimer = 0;
        self.draw = false;
    }

    // Loads ch8 file from path and loads it into memory starting from index 512 
    pub fn load_rom(&mut self, fpath: &str) {
        let buf = fs::read(fpath).expect("ROM could not be found");

        for i in 0..buf.len() {
            self.memory[as_index(START_INDEX) + i] = buf[i];
        }
    }

    pub fn fetch_next_opcode(&mut self) {
        self.pc += 2;
    }
}

// All the chip8 instructions
impl Emulator {

    pub fn shift_vx_left(&mut self, x: u8, y: u8) {
        let lost_bit = self.register(x) & 1;
        println!("lost bit: {}", lost_bit);

        self.set_reg(x, self.register(y));
        self.set_reg(x, self.register(x) << 1);
        self.set_vf(lost_bit);

    }

    pub fn shift_vx_right(&mut self, x: u8, y: u8) {
        let lost_bit = self.register(x) & 1;

        self.set_reg(x, self.register(y));
        self.set_reg(x, self.register(x) >> 1);
        self.set_vf(lost_bit);
    }

    pub fn subtract_vy_vx(&mut self, y: u8, x: u8) {
        let vx = self.register(x);
        let vy = self.register(y);

        self.set_vf(1);
        // If the result underflows set carry flag to 0 
        if vx > vy {
            self.set_vf(0);
        } else {
            self.set_reg(x, vx - vy);
        }
    }

    pub fn subtract_vx_vy(&mut self, x: u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);

        self.set_vf(1);
        // If the result underflows set carry flag to 0 
        if vy > vx {
            self.set_vf(0)
        } else {
            self.set_reg(x, vx - vy);
        }
    }

    pub fn add_vy_to_vx(&mut self, x: u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);

        // If it overflows then set carry flag to 0
        let result = vx.checked_add(vy);
        match result {
            Some(sum) => {
                self.set_reg(x, sum);
                self.set_vf(0);
            },
            None => self.set_vf(1),
        }
    }

    pub fn set_vx_to_vy(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.register(y));
    }

    pub fn binary_or(&mut self, x: u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);
        self.set_reg(x, vx | vy);
    }

    pub fn binary_and(&mut self, x:u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);
        self.set_reg(x, vx & vy)
    }

    pub fn logical_xor(&mut self, x: u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);
        self.set_reg(x, vx ^ vy);
    }

    pub fn clear_screen(&mut self) {
    }

    pub fn jump_to(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    pub fn set_vx_to_nn(&mut self, x: u8, nn: u8) {
        self.set_reg(x, nn);
    }

    pub fn add_to_vx(&mut self, x: u8, nn: u8) {
        let vx = self.register(x);
        let result = vx.checked_add(nn);
       
        // Don't add nn to vx if it causes the register to overflow
        match result {
            Some(sum) => {
                self.set_reg(x, sum);
            },

            None => (),
        }
    }

    pub fn set_index(&mut self, nnn: u16) {
        self.index = nnn;
    }

    pub fn update_pixels(&mut self, x: u8, y: u8) {
       let vx = self.register(x);
       let vy = self.register(y);
       let x = vx % 64;
       let y = vy % 32;
       
       self.set_vf(0);

       for i in 0..self.screen.len() {
           // Get the Nth byte of sprite data
           println!("{}", self.screen(i));
       }

    }

    pub fn subroutine_return(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    pub fn call_subroutine(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }
}

// Setters and getters
impl Emulator {
    pub fn set_vf(&mut self, v: u8) {
        self.register[0xF] = v;
    }

    pub fn set_reg(&mut self, i: u8, v: u8) {
        self.register[as_index(i)] = v; 
    }

    pub fn add_reg(&mut self, i: u8, v: u8) {
        self.register[as_index(i)] += v;
    }

    pub fn register(&self,i: u8) -> u8 {
        self.register[as_index(i)]
    }

    pub fn memory(&self, i: u16) -> u8 {
        self.memory[as_index(i)]
    }

    pub fn screen(&self, i: usize) -> bool {
        self.screen[i]
    }

    pub fn memory_add(&mut self, i: u8, v: u8) {
        self.memory[as_index(i)] = v;
    }

    pub fn pc(&self) -> u16 {
        self.pc
    }

}

