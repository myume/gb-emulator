use std::io::{self, Write};

use crate::utils::is_set;

pub struct Serial {
    data: u8,    // serial transfer data
    control: u8, // serial transfer control
    print_serial: bool,
}

impl Serial {
    pub fn new(print_serial: bool) -> Self {
        Serial {
            data: 0,
            control: 0,
            print_serial,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => panic!("Invalid Serial address {:#06X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xFF01 => self.data = byte,
            0xFF02 => {
                self.control = byte;
                if is_set(self.control, 7) && self.print_serial {
                    print!("{}", self.data as char);
                    io::stdout().flush().unwrap();
                }
            }
            _ => panic!("Invalid Serial address {:#06X}", address),
        }
    }
}
