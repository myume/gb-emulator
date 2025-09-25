use crate::{
    cpu::Cycles,
    utils::{is_set, set_bit},
};

pub struct Timer {
    div: u8,
    tima: u8,
    tma: u8,

    // TAC
    enable: bool,
    frequency: usize,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            enable: true,
            frequency: 256 * 4,
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

    pub fn tick(&self, cycles: Cycles) {}
}
