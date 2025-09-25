use crate::utils::is_set;

pub enum JoypadDpad {
    Up = 2,
    Down = 3,
    Left = 1,
    Right = 0,
}

pub enum JoypadButtons {
    Start = 3,
    Select = 4,
    A = 0,
    B = 1,
}

pub struct Joypad {
    select_buttons: bool,
    select_dpad: bool,

    dpad: u8,
    buttons: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            select_buttons: false,
            select_dpad: false,
            dpad: 0xFF,
            buttons: 0xFF,
        }
    }

    pub fn read(&self) -> u8 {
        if self.select_dpad {
            0xE0 | (self.dpad & 0x0F)
        } else if self.select_buttons {
            0xD0 | (self.buttons & 0x0F)
        } else {
            0xFF
        }
    }

    pub fn write(&mut self, byte: u8) {
        self.select_buttons = !is_set(byte, 5);
        self.select_dpad = !is_set(byte, 4);
    }
}
