use crate::emulator;

pub struct Assembler {
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

impl Assembler {
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

impl Assembler {
    // Merges two bytes together to get an instruction
    // Fetches the next opcode
    // Extracts all the relevant information from the instruction 
    // Creates a new Instruction object
    pub fn decode(&self, emu: &mut emulator::Emulator) -> Instruction {
        let opcode: u16 = merge_bytes(self.l_byte.into(), self.h_byte.into());

        emu.fetch_next_opcode();

        let x: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
        let y: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
        let n: u8 = (opcode & 0x000F).try_into().unwrap();
        let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
        let nnn: u16 = opcode & 0x0FFF;
        let id: u16 = opcode & 0xF000;

        Instruction::from(id, x, y, n, nn, nnn)
    }

    pub fn execute(&self, emu: &mut emulator::Emulator, instruction: &Instruction) {
        match instruction.id {
            0x8000 => match instruction.n {
                0x0000 => emu.set_vx_to_vy(instruction.x, instruction.y),
                0x0001 => emu.binary_or(instruction.x, instruction.y),
                0x0002 => emu.binary_and(instruction.x, instruction.y),
                0x0003 => emu.logical_xor(instruction.x, instruction.y),
                0x0004 => emu.add_vy_to_vx(instruction.x, instruction.y),
                0x0005 => emu.subtract_vx_vy(instruction.x, instruction.y),
                0x0007 => emu.subtract_vy_vx(instruction.y, instruction.x),
                0x0006 => emu.shift_vx_right(instruction.x, instruction.y),
                0x000E => emu.shift_vx_left(instruction.x, instruction.y),
                _ => (),
            },
            0x0000 => match instruction.n {
                0x0000 => emu.clear_screen(),
                0x00EE => emu.subroutine_return(),
                _ => (),
            },
            0xA000 => emu.set_index(instruction.nnn),
            0x1000 => emu.jump_to(instruction.nnn),
            0x2000 => emu.call_subroutine(instruction.nnn),
            0x6000 => emu.set_vx_to_nn(instruction.x, instruction.nn),
            0x7000 => emu.add_to_vx(instruction.x, instruction.nn),
            0xD000 => emu.update_pixels(instruction.x, instruction.y),
            _ => (),
        }
    }
}


