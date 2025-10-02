use std::{cell::RefCell, rc::Rc};

use crate::{
    cpu::Cycles,
    mmu::InterruptFlag,
    utils::{is_set, set_bit},
};

pub struct Timer {
    clock: u16,
    tima_clock: usize,

    div: u8,
    tima: u8,
    tma: u8,

    // TAC
    enable: bool,
    frequency: usize,
    interrupt_flag: Rc<RefCell<u8>>,
}

impl Timer {
    pub fn new(interrupt_flag: Rc<RefCell<u8>>) -> Self {
        Timer {
            clock: 0,
            tima_clock: 0,
            div: 0,
            tima: 0,
            tma: 0,
            enable: true,
            frequency: 256 * 4,
            interrupt_flag,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => self.div,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => {
                let clock_select = match self.frequency {
                    1024 => 0,
                    16 => 1,
                    64 => 2,
                    256 => 3,
                    _ => unreachable!(),
                };

                if self.enable {
                    set_bit(clock_select, 2)
                } else {
                    clock_select
                }
            }
            _ => panic!("Invalid Timer address {:#06X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xFF04 => self.div = 0x00,
            0xFF05 => self.tima = byte,
            0xFF06 => self.tma = byte,
            0xFF07 => {
                self.enable = is_set(byte, 2);
                self.frequency = match byte & 0x03 {
                    0 => 1024,
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => unreachable!(),
                }
            }
            _ => panic!("Invalid Timer address {:#06X}", address),
        }
    }

    fn increment_clock(&mut self, amount: usize) {
        self.clock = self.clock.wrapping_add(amount as u16);
        self.div = (self.clock >> 8) as u8;
    }

    pub fn tick(&mut self, cycles: Cycles) {
        self.increment_clock(cycles);

        if self.enable {
            self.tima_clock = self.tima_clock.wrapping_add(cycles);
            if self.tima_clock == self.frequency {
                self.tima_clock %= self.frequency;

                if self.tima.wrapping_add(1) < self.tima {
                    self.tima = self.tma;
                    *self.interrupt_flag.borrow_mut() =
                        set_bit(*self.interrupt_flag.borrow(), InterruptFlag::Timer as u8);
                } else {
                    self.tima = self.tima.wrapping_add(1);
                }
            }
        }
    }
}
