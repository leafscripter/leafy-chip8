#![allow(unused_assignments)]
extern crate sdl2;

//use chip8::decoder
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::surface::Surface;
use chip8::emulator::Emulator; 
use chip8::assembler::Assembler;
use chip8::assembler::DecodedInstruction;
use std::fs;

fn main() {
    // Just a test array 
    let graphics: [u8; 64 * 32] = [0; 64 * 32];

    // Setup a render system
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("Leafy-Chip8", 640, 320).build().unwrap();

    let mut event_pump = sdl.event_pump().unwrap();

    // Setup emulator
    let mut emu = Emulator::new();
    let mut assembler = Assembler::new();
    let mut decoded_instruction: DecodedInstruction;
    emu.load_rom("/home/kaysar/Documents/ibm_logo.ch8");
    //load_rom(&mut emu, "/home/kaysar/Documents/ibm_logo.ch8");

    'running:loop {
        // Handle window events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                }, 
                _ => (),
            }
        }
        // Emulate a Chip8 cycle
        assembler.fetch_opcode(&emu);
        decoded_instruction = assembler.decode(&mut emu);
        assembler.execute(&decoded_instruction, &mut emu);

        if emu.draw() == true {
            // Update screen
        }

    }

}
