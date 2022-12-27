#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
use std::fs;
use std::error::Error;
//use std::{thread, time};
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

pub struct Processor {
    stack: Vec<u16>,
    memory: [u8; MEM_SIZE],
    register: [u8; REG_SIZE],
    screen: [bool; SCREEN_SIZE],
    pc: u16, 
    sp: u16,
    index: u16, 
    dtimer: u8,
    stimer: u8,
    draw: bool,
}

impl Processor {
    pub fn init() -> Self {
        let mut emu = Self {
            memory: [0; MEM_SIZE] ,
            screen: [false; SCREEN_SIZE],
            register: [0; REG_SIZE],
            index: 0,
            pc: START_INDEX,
            sp: 0,
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
    }

    fn readf(filepath: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let file = fs::read(filepath)?; 
        Ok(file)
    }

    pub fn load_rom(&mut self, filepath: &str) {
        let buffer = Processor::readf(filepath).unwrap();

        // load the contents of the rom file into memory starting from 0x200
        for i in 0..buffer.len() {
            self.memory[as_index(START_INDEX) + i] = buffer[i];
        }
    }

    pub fn exec_cycle(&mut self) {
        // fetch two instructions
        let first_instr: u16 = self.memory(self.pc).try_into().unwrap();  
        let second_instr: u16 = self.memory(self.pc).try_into().unwrap(); 

        // Merge them both into one 16 bit instruction
        let opcode: u16 = Processor::into_opcode(first_instr, second_instr);
        
        // Get ready to fetch next instruction
        self.pc += 2;

        // decode and execute
        let x: u8 = ((opcode &  0x0F00) >> 8)
            .try_into()
            .unwrap(); 
        let y: u8 = ((opcode & 0x00F0) >> 4)
            .try_into()
            .unwrap();
        let n:u8 = (opcode & 0x000F)
            .try_into()
            .unwrap();
        let nn:u8 = (opcode & 0x00FF)
            .try_into()
            .unwrap();
        let nnn = opcode & 0x0FFF;
        
        match opcode & 0xF000{
            0x8000 => match n {
                _ => (),
            }
            0x0000 => match n {
                0x0000 => self.clear_screen(),
                0x00EE => self.subroutine_return(),
                _ => (),
            }
            0xA000 => self.set_index(nnn),
            0x1000 => self.jump_to(nnn),
            0x2000 => self.call_subroutine(nnn),
            0x6000 => self.set_vx_to_nn(x, nn),
            0x7000 => self.add_to_vx(x, nn),
            0xD000 => self.op_dxyn(x,y),
            _ => (),
        } 

        // set timers
        if self.stimer > 0 {
            self.stimer -= 1; 
        }
        
        if self.dtimer > 0 {
            self.dtimer -= 1;
        }
    }

    pub fn update_screen(&self) {

    }
}

// All the chip8 instructions
impl Processor {

    fn shift_vx_left(&mut self, x: u8, y: u8) {
        let lost_bit = self.register(x) & 1;

        self.set_reg(x, self.register(y));
        self.set_reg(x, self.register(x) << 1);
        self.set_vf(lost_bit);

    }

    fn shift_vx_right(&mut self, x: u8, y: u8) {
        let lost_bit = self.register(x) & 1;

        self.set_reg(x, self.register(y));
        self.set_reg(x, self.register(x) >> 1);
        self.set_vf(lost_bit);
    }

    fn subtract_vy_vx(&mut self, y: u8, x: u8) {
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

    fn subtract_vx_vy(&mut self, x: u8, y: u8) {
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

    fn add_vy_to_vx(&mut self, x: u8, y: u8) {
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

    fn set_vx_to_vy(&mut self, x: u8, y: u8) {
        self.set_reg(x, self.register(y));
    }

    fn binary_or(&mut self, x: u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);
        self.set_reg(x, vx | vy);
    }

    fn binary_and(&mut self, x:u8, y: u8) {
        let vx = self.register(x);
        let vy = self.register(y);
        self.set_reg(x, vx & vy)
    }

    fn logical_xor(&mut self, x: u8, y: u8) {
        let mut vx = self.register[as_index(x)];
        let vy = self.register[as_index(y)];
        vx ^= vy;
    }

    fn clear_screen(&mut self) {
    }

    fn jump_to(&mut self, nnn: u16) {
        self.pc = nnn;
    }

    fn set_vx_to_nn(&mut self, x: u8, nn: u8) {
        self.register[as_index(x)] = nn;
    }

    fn add_to_vx(&mut self, x: u8, nn: u8) {
        self.register[as_index(x)] += nn;
    }

    fn set_index(&mut self, nnn: u16) {
        self.index = nnn;
    }

    fn op_dxyn(&mut self, x: u8, y: u8) {
       let vx = self.register[usize::from(x)];
       let xy = self.register[usize::from(y)];
    }

    fn subroutine_return(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn call_subroutine(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }
}

// Setters and getters
impl Processor {
    fn set_vf(&mut self, v: u8) {
        self.register[0xF] = v;
    }

    fn set_reg(&mut self, i: u8, v: u8) {
        self.register[as_index(i)] = v; 
    }

    fn register(&self,i: u8) -> u8 {
        self.register[as_index(i)]
    }

    fn memory(&self, i: u16) -> u8 {
        self.memory[as_index(i)]
    }

}

// Bit manipulation methods
impl Processor {
    fn into_opcode(mut a: u16, b: u16) -> u16 {
        a <<= 8;
        a |= b;
        a.into()
    }
}
