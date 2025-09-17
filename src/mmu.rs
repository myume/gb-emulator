use crate::utils::compose_bytes;

pub struct MMU {}

impl MMU {
    pub fn new() -> Self {
        MMU {}
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // cartridge
            0x0000..=0x7FFF => {
                todo!()
            }
            // VRAM
            0x8000..=0x9FFF => {
                todo!()
            }
            // External RAM (from cartridge)
            0xA000..=0xBFFF => {
                todo!()
            }
            // WRAM
            0xC000..=0xDFFF => {
                todo!()
            }
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => {
                todo!()
            }
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => {
                todo!()
            }
            // Not usable
            0xFEA0..=0xFEFF => {
                todo!()
            }
            // I/O Registers
            0xFF00..=0xFF7F => {
                todo!()
            }
            // HRAM (high RAM)
            0xFF80..=0xFFFE => {
                todo!()
            }
            // Interrupt Enable register (IE)
            0xFFFF => {
                todo!()
            }
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address.wrapping_add(1));
        compose_bytes(high, low)
    }

    pub fn write_byte(&self, address: u16, value: u8) {
        match address {
            // cartridge
            0x0000..=0x7FFF => {
                todo!()
            }
            // VRAM
            0x8000..=0x9FFF => {
                todo!()
            }
            // External RAM (from cartridge)
            0xA000..=0xBFFF => {
                todo!()
            }
            // WRAM
            0xC000..=0xDFFF => {
                todo!()
            }
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => {
                todo!()
            }
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => {
                todo!()
            }
            // Not usable
            0xFEA0..=0xFEFF => {
                todo!()
            }
            // I/O Registers
            0xFF00..=0xFF7F => {
                todo!()
            }
            // HRAM (high RAM)
            0xFF80..=0xFFFE => {
                todo!()
            }
            // Interrupt Enable register (IE)
            0xFFFF => {
                todo!()
            }
        }
    }

    pub fn write_word(&self, address: u16, value: u16) {
        let low = value | 0x00FF;
        let high = value | 0xFF00;
        self.write_byte(address, low as u8);
        self.write_byte(address.wrapping_add(1), high as u8);
    }
}
