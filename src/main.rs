#![feature(
    coroutines,
    coroutine_trait,
    never_type,
    exhaustive_patterns,
    cell_update
)]

extern crate bitflags;
extern crate gfx_device_gl;
extern crate image;
extern crate nes_rom_reader;
extern crate num_derive;
extern crate num_traits;
extern crate piston_window;

use piston_window::{Button, Event, Key, Loop, PressEvent, ReleaseEvent};
use std::fs::File;
use std::io::{self, Read, Result, Write};
use std::ops::{Coroutine, CoroutineState};
use std::pin::Pin;
use view::NESView;

mod addressings;
mod control;
mod cparams;
mod cpu;
mod nes;
mod opcode;
mod operations;
mod pad;
mod ppu;
mod view;

#[macro_use]
mod misc;

use misc::FPS;
use nes::*;

use crate::control::{GamepadInput, KeyboardInput};
use crate::ppu::RenderStep;

fn main() -> Result<()> {
    let rom_dir = if cfg!(feature = "nestest") {
        "../nes-roms/nestest.nes".to_string()
    } else {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            eprintln!("Usage: {} <path-to-rom>", args[0]);
            std::process::exit(1);
        }
        args[1].clone()
    };

    let nes = Nes::start(&rom_dir);
    let mut file = File::create("nestest.log")?;

    if cfg!(feature = "nestest") {
        nes.cpu.pc.set(0xC000);
        nes.cpu.p.set(cpu::Status::from_bits_truncate(0x24));
    }

    let mut nes_run = nes.run();
    let mut view = NESView::new(&nes);

    let mut fps = FPS::new();
    let mut gamepad_input = GamepadInput::new();

    while let Some(event) = view.window.next() {
        gamepad_input.update(&nes);

        match event {
            Event::Loop(Loop::Render(_)) => {
                loop {
                    match Pin::new(&mut nes_run).resume(()) {
                        CoroutineState::Yielded(NesStep::Ppu(step @ ppu::PpuStep::Vblank)) => {
                            // println!("{:?}", step);
                            break;
                        }
                        CoroutineState::Yielded(NesStep::Ppu(
                            step @ ppu::PpuStep::Cycle(renderstep, _),
                        )) => {
                            // println!("{:?}", step);
                            if let RenderStep::Cycle(_frame, _x, y) = renderstep {
                                match y {
                                    241 => {
                                        // _ = io::stdin().read(&mut [0u8; 2]);
                                    }
                                    _ => {}
                                }
                            }
                        }
                        CoroutineState::Yielded(NesStep::Cpu(step @ cpu::CpuStep::Cycle)) => {
                            // println!("{:?}", step);
                        }
                        CoroutineState::Yielded(NesStep::Cpu(step @ cpu::CpuStep::Op(_, _, _))) => {
                            // println!("{:?}", step);
                        }
                        CoroutineState::Yielded(NesStep::Cpu(
                            step @ cpu::CpuStep::DebugLine(_),
                        )) => {
                            // println!("{:?}", step);
                            // println!("{:?}", debug_line);
                            if cfg!(feature = "nestest") {
                                if let cpu::CpuStep::DebugLine(debug_line) = step {
                                    if debug_line[..4] == "0001".to_string() {
                                        println!("End of Cpu Nestest");
                                        std::process::exit(0);
                                    }
                                    writeln!(file, "{}", debug_line)?;
                                }
                            }
                        }
                    }
                }

                // println!("update view!");
                view.set_title(fps.update());
                view.update(&nes.ppu.img.borrow());
                view.draw_2d(&event);

                // std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Event::Input(_, _) => {
                if let Some(Button::Keyboard(key)) = event.press_args() {
                    KeyboardInput::keypress(&nes, key);
                }
                if let Some(Button::Keyboard(key)) = event.release_args() {
                    KeyboardInput::keyrelease(&nes, key);
                }
            }
            _ => {}
        }
    }

    println!("Hello World");
    Ok(())
}
