use super::nes::Nes;
use super::pad;
use chrono::{Datelike, Local, Timelike};
use piston_window::keyboard::Key;

pub struct KeyboardInput;

impl KeyboardInput {
    pub fn keypress(nes: &Nes, key: Key) {
        println!("{:?}", key);
        match key {
            Key::G => {
                nes.ppu.grid_on.update(|x| !x);
            }
            Key::P => {
                let img = &nes.ppu.img.borrow();
                let dt = Local::now();
                let filename = format!(
                    "{}_{:02}_{:02}_{:02}_{:02}_{:02}.png",
                    dt.year(),
                    dt.month(),
                    dt.day(),
                    dt.hour(),
                    dt.minute(),
                    dt.second()
                );
                match img.save(filename) {
                    Ok(()) => {}
                    Err(_e) => {}
                }
            }
            Key::Q => {
                let debug_num = nes.ppu.debug_num.get() + 1;
                println!("{}", debug_num);
                nes.ppu.debug_num.set(debug_num);
            }

            Key::NumPad0 => nes.pad.press(pad::PadButton::A),
            Key::NumPad5 => nes.pad.press(pad::PadButton::B),
            Key::NumPad4 => nes.pad.press(pad::PadButton::LEFT),
            Key::NumPad6 => nes.pad.press(pad::PadButton::RIGHT),
            Key::NumPad8 => nes.pad.press(pad::PadButton::UP),
            Key::NumPad2 => nes.pad.press(pad::PadButton::DOWN),
            Key::NumPad7 => nes.pad.press(pad::PadButton::START),
            Key::NumPad1 => nes.pad.press(pad::PadButton::SELECT),

            Key::J => nes.pad.press(pad::PadButton::A),
            Key::K => nes.pad.press(pad::PadButton::B),
            Key::A => nes.pad.press(pad::PadButton::LEFT),
            Key::D => nes.pad.press(pad::PadButton::RIGHT),
            Key::W => nes.pad.press(pad::PadButton::UP),
            Key::S => nes.pad.press(pad::PadButton::DOWN),
            Key::Space => nes.pad.press(pad::PadButton::START),
            Key::H => nes.pad.press(pad::PadButton::SELECT),

            _ => {}
        }
    }

    pub fn keyrelease(nes: &Nes, key: Key) {
        match key {
            Key::NumPad0 => nes.pad.release(pad::PadButton::A),
            Key::NumPad5 => nes.pad.release(pad::PadButton::B),
            Key::NumPad4 => nes.pad.release(pad::PadButton::LEFT),
            Key::NumPad6 => nes.pad.release(pad::PadButton::RIGHT),
            Key::NumPad8 => nes.pad.release(pad::PadButton::UP),
            Key::NumPad2 => nes.pad.release(pad::PadButton::DOWN),
            Key::NumPad7 => nes.pad.release(pad::PadButton::START),
            Key::NumPad1 => nes.pad.release(pad::PadButton::SELECT),

            Key::J => nes.pad.release(pad::PadButton::A),
            Key::K => nes.pad.release(pad::PadButton::B),
            Key::A => nes.pad.release(pad::PadButton::LEFT),
            Key::D => nes.pad.release(pad::PadButton::RIGHT),
            Key::W => nes.pad.release(pad::PadButton::UP),
            Key::S => nes.pad.release(pad::PadButton::DOWN),
            Key::Space => nes.pad.release(pad::PadButton::START),
            Key::H => nes.pad.release(pad::PadButton::SELECT),
            _ => {}
        }
    }
}
