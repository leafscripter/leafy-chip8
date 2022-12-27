extern crate sdl2;
//use sdl2::event::Event;
//use sdl2::keyboard::Keycode;
//use sdl2::video::Window;
//use sdl2::pixels::Color;
//use sdl2::render::Canvas;
//use sdl2::rect::Rect;

//use std::time::Duration;
use chip8::emulator::Processor; 
//use chip8::display::Display;

fn main() {
    // Setup a render system

    // Setup emulator
    let mut emu = Processor::init(); 
    emu.load_rom("/home/kaysar/Documents/ibm_logo.ch8");

    loop {
    
        emu.exec_cycle();
        
        if emu.draw() == true {
            // Update screen
        }

    }

}
