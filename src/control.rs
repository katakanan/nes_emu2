use super::nes::Nes;
use super::pad;
use chrono::{Datelike, Local, Timelike};
use gilrs::{Axis, Button as GamepadButton, Event, EventType, Gilrs};
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

pub struct GamepadInput {
    gilrs: Option<Gilrs>,
    left_stick_x: pad::PadButton,
    left_stick_y: pad::PadButton,
}

impl GamepadInput {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => {
                let gamepads: Vec<_> = gilrs
                    .gamepads()
                    .map(|(id, gamepad)| (id, gamepad.name().to_string()))
                    .collect();
                if gamepads.is_empty() {
                    eprintln!("Gamepad input enabled: no gamepads detected");
                } else {
                    for (id, name) in &gamepads {
                        eprintln!("Gamepad detected: {:?} {}", id, name);
                    }
                }
                Some(gilrs)
            }
            Err(err) => {
                eprintln!("Gamepad input disabled: {}", err);
                None
            }
        };

        Self {
            gilrs,
            left_stick_x: pad::PadButton::empty(),
            left_stick_y: pad::PadButton::empty(),
        }
    }

    pub fn update(&mut self, nes: &Nes) {
        if self.gilrs.is_none() {
            return;
        }

        loop {
            let Some(Event { event, .. }) =
                self.gilrs.as_mut().and_then(|gilrs| gilrs.next_event())
            else {
                break;
            };
            match event {
                EventType::ButtonPressed(button, _) => {
                    if let Some(pad_button) = Self::map_button(button) {
                        Self::trace_event("press", pad_button);
                        nes.pad.press(pad_button);
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(pad_button) = Self::map_button(button) {
                        Self::trace_event("release", pad_button);
                        nes.pad.release(pad_button);
                    }
                }
                EventType::AxisChanged(axis, value, _) => {
                    self.update_axis(nes, axis, value);
                }
                EventType::Disconnected => {
                    nes.pad.release(self.left_stick_x | self.left_stick_y);
                    self.left_stick_x = pad::PadButton::empty();
                    self.left_stick_y = pad::PadButton::empty();
                }
                _ => {}
            }
        }
    }

    fn map_button(button: GamepadButton) -> Option<pad::PadButton> {
        match button {
            GamepadButton::South => Some(pad::PadButton::A),
            GamepadButton::East => Some(pad::PadButton::B),
            GamepadButton::Start => Some(pad::PadButton::START),
            GamepadButton::Select => Some(pad::PadButton::SELECT),
            GamepadButton::DPadUp => Some(pad::PadButton::UP),
            GamepadButton::DPadDown => Some(pad::PadButton::DOWN),
            GamepadButton::DPadLeft => Some(pad::PadButton::LEFT),
            GamepadButton::DPadRight => Some(pad::PadButton::RIGHT),
            _ => None,
        }
    }

    fn update_axis(&mut self, nes: &Nes, axis: Axis, value: f32) {
        let threshold = 0.5;
        match axis {
            Axis::LeftStickX => {
                let next = if value <= -threshold {
                    pad::PadButton::LEFT
                } else if value >= threshold {
                    pad::PadButton::RIGHT
                } else {
                    pad::PadButton::empty()
                };
                Self::replace_button_state(nes, &mut self.left_stick_x, next);
            }
            Axis::LeftStickY => {
                let next = if value <= -threshold {
                    pad::PadButton::DOWN
                } else if value >= threshold {
                    pad::PadButton::UP
                } else {
                    pad::PadButton::empty()
                };
                Self::replace_button_state(nes, &mut self.left_stick_y, next);
            }
            _ => {}
        }
    }

    fn replace_button_state(nes: &Nes, current: &mut pad::PadButton, next: pad::PadButton) {
        if *current == next {
            return;
        }

        nes.pad.release(*current);
        nes.pad.press(next);
        *current = next;
    }

    fn trace_event(action: &str, button: pad::PadButton) {
        if std::env::var_os("NES_TRACE_GAMEPAD").is_some() {
            eprintln!("Gamepad {} {:?}", action, button);
        }
    }
}
