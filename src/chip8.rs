#![allow(dead_code)]
#![allow(unused_variables)]
use std::fs;
use std::error::Error;
use std::{thread, time};
use crate::useful::u16_index;
use crate::useful::u8_index;

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
    sp: u16,
    index: u16, 
    dtimer: u8,
    stimer: u8,
    draw: bool,
}

impl Emulator {
    pub fn setup() -> Self {
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
        let buffer = Emulator::readf(filepath).unwrap();

        // load the contents of the rom file into memory starting from 0x200
        for i in 0..buffer.len() {
            self.memory[u16_index(&START_INDEX) + i] = buffer[i];
        }
    }

    pub fn exec_cycle(&mut self) {
        // fetch two instructions
        let first_instr: u16 = self.memory[u16_index(&self.pc)].into();  
        let second_instr: u16 = self.memory[u16_index(&self.pc) + 1].into(); 

        // Merge them both into one 16 bit instruction
        let mut opcode: u16 = first_instr; 
        opcode <<= 8;  
        opcode |= second_instr; 
        
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
            0x0000 => match opcode & 0x000F {
                0x0000 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                _ => (),
            }
            0xA000 => self.op_annn(nnn),
            0x1000 => self.op_1nnn(nnn),
            0x2000 => self.op_2nnn(nnn),
            0x6000 => self.op_6xnn(x, nn),
            0x7000 => self.op_7xnn(x, nn),
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
impl Emulator {
    fn op_00e0(&mut self) {
        println!("Clearing screen!");
        thread::sleep(time::Duration::from_millis(109));
    }

    fn op_1nnn(&mut self, nnn: u16) {
        println!("Setting program counter to address {}", nnn);
        self.pc = nnn;
    }

    fn op_6xnn(&mut self, x: u8, nn: u8) {
        println!("Setting register {}", x); 
        self.register[u8_index(&x)] = nn;
        thread::sleep(time::Duration::from_millis(27));
    }

    fn op_7xnn(&mut self, x: u8, nn: u8) {
        println!("Adding to register {}", x);
        self.register[u8_index(&x)] += nn;
        thread::sleep(time::Duration::from_millis(45))
    }

    fn op_annn(&mut self, nnn: u16) {
        println!("Setting index register to {}", nnn);
        self.index = nnn;
    }

    fn op_dxyn(&mut self, x: u8, y: u8) {
        println!("Drawing... (not implemented yet)");
       let vx = self.register[usize::from(x)];
       let xy = self.register[usize::from(y)];
    }

    fn op_00ee(&mut self) {
        self.pc = self.stack.pop().unwrap();
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }
}
