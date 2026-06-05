use bitflags::bitflags;
use std::cell::Cell;

bitflags! {
    #[derive(Debug, Default, Clone, Copy)]
    pub struct PadButton:u8{
        const A = 1 << 0;
        const B = 1 << 1;
        const START = 1 << 2;
        const SELECT = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
        const LEFT = 1 << 6;
        const RIGHT = 1 << 7;
    }
}

#[derive(Debug, Default)]
pub struct Pad {
    pub status: Cell<PadButton>,
    pub shift_index: Cell<u8>,
    pub strobe_enable: Cell<bool>,
}

impl Pad {
    pub fn press(&self, button: PadButton) {
        self.status.update(|s| s | button);
    }

    pub fn release(&self, button: PadButton) {
        self.status.update(|s| s & !button);
    }
}
