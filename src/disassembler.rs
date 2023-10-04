use crate::emulator;

pub struct Disassembler {
    l_byte: u8,
    h_byte: u8,
}

pub struct Instruction {
    x: u8,
    y: u8,
    n: u8,
    nn: u8, 
    nnn: u16,
    id: u16,
}

enum Opcode {
    JUMP = 0x1000,
    CALL_SUBROUTINE = 0x2000,
    SUBROUTINE_RETURN = 0x00EE,
    CLEAR_SCREEN = 0x00E0,
    SET_INDEX = 0xA000,
    SET = 0x6000,
    ADD = 0x7000,
    LOGICAL = 0x8000,
    DISPLAY = 0xD000,
}

enum Logical {
    SET = 0x0000,
    OR = 0x0001,
    AND = 0x0002,
    XOR = 0x0003,
    ADD = 0x0004,
    SUB_FIVE = 0x0005, // Sets VX to VX - VY 
    SUB_SEVEN = 0x0007, // Sets VX to VY - VX
    SHIFT_RIGHT = 0x0006, 
    SHIFT_LEFT = 0x000E,
}

impl Instruction {
    fn from(id: u16, x: u8, y: u8, n: u8, nn: u8, nnn: u16) -> Self {
        Self {
            x,
            y,
            n, 
            nn,
            nnn,
            id,
        }
    }
}

fn merge_bytes(mut l_byte: u16, h_byte: u16) -> u16 {
    l_byte <<=8;
    l_byte|= h_byte; 
    l_byte
}

impl Diassembler {
    pub fn new() -> Self {
        Self {
            l_byte: 0,
            h_byte: 0,
        }
    }

    pub fn fetch_opcode(&mut self, emu: &emulator::Emulator) {
        self.l_byte = emu.memory(emu.pc());
        self.h_byte = emu.memory(emu.pc() + 1);
    }
}

impl Disassembler {
    // Merges two bytes together to get an instruction
    // Fetches the next opcode
    // Extracts all the relevant information from the instruction 
    // Creates a new Instruction object
    pub fn decode(&self, emu: &mut emulator::Emulator) -> Instruction {
        let opcode: u16 = merge_bytes(self.l_byte.into(), self.h_byte.into());

        emu.fetch_next_opcode();

        let x: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap(); // x is used to look up registers VX-VF
        let y: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap(); // y is used to look up registers VX-VF
        let n: u8 = (opcode & 0x000F).try_into().unwrap(); // a 4-bit number
        let nn: u8 = (opcode & 0x00FF).try_into().unwrap(); // an 8-bit immediate number
        let nnn: u16 = opcode & 0x0FFF; // a 12-bit immediate memory address
        let id: u16 = opcode & 0xF000; // the opcode to execute 

        Instruction::from(id, x, y, n, nn, nnn)
    }

    pub fn execute(&self, emu: &mut emulator::Emulator, instruction: &Instruction) {
        match instruction.id {
            Opcode::LOGICAL => match instruction.n {
                Logical::SET => emu.set_vx_to_vy(instruction.x, instruction.y),
                Logical::OR => emu.binary_or(instruction.x, instruction.y),
                Logical::AND => emu.binary_and(instruction.x, instruction.y),
                Logical::XOR => emu.logical_xor(instruction.x, instruction.y),
                Logical::ADD => emu.add_vy_to_vx(instruction.x, instruction.y),
                Logical::SUB_FIVE => emu.subtract_vx_vy(instruction.x, instruction.y),
                Logical::SUB_SEVEN => emu.subtract_vy_vx(instruction.y, instruction.x),
                Logical::SHIFT_RIGHT => emu.shift_vx_right(instruction.x, instruction.y),
                Logical::SHIFT_LEFT => emu.shift_vx_left(instruction.x, instruction.y),
                _ => (),
            },
            0x0000 => match instruction.n {
                Opcode::CLEAR_SCREEN => emu.clear_screen(),
                Opcode::SUBROUTINE_RETURN => emu.subroutine_return(),
                _ => (),
            },
            Opcode::SET_INDEX => emu.set_index(instruction.nnn),
            Opcode::JUMP => emu.jump_to(instruction.nnn),
            Opcode::CALL_SUBROUTINE => emu.call_subroutine(instruction.nnn),
            Opcode::SET => emu.set_vx_to_nn(instruction.x, instruction.nn),
            Opcode::ADD => emu.add_to_vx(instruction.x, instruction.nn),
            Opcode::DISPLAY => emu.update_pixels(instruction.x, instruction.y),
            _ => (),
        }
    }
}


