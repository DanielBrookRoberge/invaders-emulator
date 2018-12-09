extern crate sdl2;
extern crate virtual_cpu_8080;
extern crate virtual_cpu_core;
mod machine;
mod screen;

use std::fs::File;
use std::io::prelude::*;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use machine::{Keys, SpaceInvadersMachine};
use screen::Screen;
use virtual_cpu_8080::cpu::emulate_instruction;
use virtual_cpu_8080::State8080;
use virtual_cpu_core::Memory;

static CYCLES_PER_FRAME: usize = 2_000_000 / 120;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let mut event_pump = sdl_context.event_pump()?;
    let mut screen = Screen::new(&sdl_context)?;

    let mut state = State8080::new();
    let mut machine = SpaceInvadersMachine::new();
    machine.press_key(Keys::Coin);

    let rom = File::open("./invaders").unwrap();
    let rom_bytes: Vec<u8> = rom.bytes().map(|x| x.unwrap()).collect();
    state.m.load(0x0000, &rom_bytes);

    let mut interrupt = 1;
    loop {
        let mut instruction_count = 0;
        while instruction_count < CYCLES_PER_FRAME {
            instruction_count += emulate_instruction(&mut state, &mut machine);
        }

        state.trigger_interrupt(interrupt);
        interrupt = if interrupt == 1 { 2 } else { 1 };

        if interrupt == 1 {
            let framebuffer = state.m.view(0x2400, 0x3fff);

            screen.clear();
            for y in 0..224 {
                let line = &framebuffer[(32 * y)..(32 * y + 32)];
                for (x, px) in line.iter().enumerate() {
                    for b in 0..8 {
                        if px & (1 << b) != 0 {
                            screen.draw(y as i16, 256 - (8 * x + b) as i16, 255)?;
                        }
                    }
                }
            }
            screen.canvas.present();
        }
        for event in event_pump.poll_iter() {
            match event {
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => return Ok(()),
                    Keycode::A => machine.press_key(Keys::Left2),
                    Keycode::D => machine.press_key(Keys::Right2),
                    Keycode::Left => machine.press_key(Keys::Left1),
                    Keycode::Right => machine.press_key(Keys::Right1),
                    Keycode::Space => machine.press_key(Keys::Shoot1),
                    Keycode::W => machine.press_key(Keys::Shoot2),
                    Keycode::C => machine.release_key(Keys::Coin),
                    Keycode::Num1 => machine.press_key(Keys::Start1),
                    Keycode::Num2 => machine.press_key(Keys::Start2),
                    _ => (),
                },
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::A => machine.release_key(Keys::Left2),
                    Keycode::D => machine.release_key(Keys::Right2),
                    Keycode::Left => machine.release_key(Keys::Left1),
                    Keycode::Right => machine.release_key(Keys::Right1),
                    Keycode::Space => machine.release_key(Keys::Shoot1),
                    Keycode::W => machine.release_key(Keys::Shoot2),
                    Keycode::C => machine.press_key(Keys::Coin),
                    Keycode::Num1 => machine.release_key(Keys::Start1),
                    Keycode::Num2 => machine.release_key(Keys::Start2),
                    _ => (),
                },
                _ => {}
            }
        }
    }
}
