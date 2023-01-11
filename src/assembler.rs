use crate::emulator;

pub struct Assembler {
    l_byte: u8,
    h_byte: u8,
}

pub struct DecodedInstruction {
    x: u8,
    y: u8,
    n: u8,
    nn: u8, 
    nnn: u16,
    id: u16,
}

impl DecodedInstruction {
    fn new(id: u16, x: u8, y: u8, n: u8, nn: u8, nnn: u16) -> Self {
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
    return l_byte;
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
    // Decodes an instruction, gets all the relevant information, then returns it as type
    // DecodedInstruction
    pub fn decode(&self, emu: &mut emulator::Emulator) -> DecodedInstruction {
        let opcode: u16 = merge_bytes(self.l_byte.into(), self.h_byte.into());
        emu.fetch_next_opcode();

        let x: u8 = ((opcode & 0x0F00) >> 8).try_into().unwrap();
        let y: u8 = ((opcode & 0x00F0) >> 4).try_into().unwrap();
        let n: u8 = (opcode & 0x000F).try_into().unwrap();
        let nn: u8 = (opcode & 0x00FF).try_into().unwrap();
        let nnn: u16 = opcode & 0x0FFF;
        let id: u16 = opcode & 0xF000;

        DecodedInstruction::new(id, x, y, n, nn, nnn)
    }

    pub fn execute(&self, opcode: &DecodedInstruction, emu: &mut emulator::Emulator) {
        match opcode.id {
            0x8000 => match opcode.n {
                0x0000 => emu.set_vx_to_vy(opcode.x, opcode.y),
                0x0001 => emu.binary_or(opcode.x, opcode.y),
                0x0002 => emu.binary_and(opcode.x, opcode.y),
                0x0003 => emu.logical_xor(opcode.x, opcode.y),
                0x0004 => emu.add_vy_to_vx(opcode.x, opcode.y),
                0x0005 => emu.subtract_vx_vy(opcode.x, opcode.y),
                0x0007 => emu.subtract_vy_vx(opcode.y, opcode.x),
                0x0006 => emu.shift_vx_right(opcode.x, opcode.y),
                0x000E => emu.shift_vx_left(opcode.x, opcode.y),
                _ => (),
            },
            0x0000 => match opcode.n {
                0x0000 => emu.clear_screen(),
                0x00EE => emu.subroutine_return(),
                _ => (),
            },
            0xA000 => emu.set_index(opcode.nnn),
            0x1000 => emu.jump_to(opcode.nnn),
            0x2000 => emu.call_subroutine(opcode.nnn),
            0x6000 => emu.set_vx_to_nn(opcode.x, opcode.nn),
            0x7000 => emu.add_to_vx(opcode.x, opcode.nn),
            0xD000 => emu.update_pixels(opcode.x, opcode.y),
            _ => (),
        }
    }
}


