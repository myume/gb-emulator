use std::{cell::RefCell, rc::Rc};

use crate::{
    gb::GBButton,
    mmu::InterruptFlag,
    utils::{is_set, reset_bit, set_bit},
};

pub struct Joypad {
    select_buttons: bool,
    select_dpad: bool,

    dpad: u8,
    buttons: u8,

    interrupt_flag: Rc<RefCell<u8>>,
}

impl Joypad {
    pub fn new(interrupt_flag: Rc<RefCell<u8>>) -> Self {
        Joypad {
            select_buttons: false,
            select_dpad: false,
            dpad: 0xFF,
            buttons: 0xFF,

            interrupt_flag,
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

    pub fn on_button_press(&mut self, button: GBButton) {
        match button {
            GBButton::Dpad(joypad_dpad) => self.dpad = reset_bit(self.dpad, joypad_dpad as u8),
            GBButton::Button(joypad_button) => {
                self.buttons = reset_bit(self.buttons, joypad_button as u8)
            }
        }

        let flag = *self.interrupt_flag.borrow();
        *self.interrupt_flag.borrow_mut() = set_bit(flag, InterruptFlag::Joypad as u8);
    }

    pub fn on_button_release(&mut self, button: GBButton) {
        match button {
            GBButton::Dpad(joypad_dpad) => self.dpad = set_bit(self.dpad, joypad_dpad as u8),
            GBButton::Button(joypad_button) => {
                self.buttons = set_bit(self.buttons, joypad_button as u8)
            }
        }

        let flag = *self.interrupt_flag.borrow();
        *self.interrupt_flag.borrow_mut() = set_bit(flag, InterruptFlag::Joypad as u8);
    }
}
