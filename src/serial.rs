use crate::cpu::Cycles;

pub struct Serial {
    data: u8,    // serial transfer data
    control: u8, // serial transfer control
}

impl Serial {
    pub fn new() -> Self {
        Serial {
            data: 0,
            control: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.data,
            0xFF02 => self.control,
            _ => panic!("Invalid Serial address 0x{:X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0xFF01 => self.data = byte,
            0xFF02 => self.control = byte,
            _ => panic!("Invalid Serial address 0x{:X}", address),
        }
    }

    pub fn tick(&self, cycles: Cycles) {}
}
