pub struct Cartridge {
    pub mbc: Box<dyn MBC>,
}

pub trait MBC {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, byte: u8);
}

pub struct NoMBC {
    rom: [u8; 0x8000],
    ram: [u8; 0xC000 - 0xA000],
}

impl NoMBC {
    pub fn new() -> Self {
        NoMBC {
            rom: [0; 0x8000],
            ram: [0; 0xC000 - 0xA000],
        }
    }
}

impl MBC for NoMBC {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // ROM
            0x0000..=0x7FFF => self.rom[address as usize],
            // RAM
            0xA000..=0xBFFF => self.ram[(address - 0xA000) as usize],
            _ => panic!("Illegal address for MBC"),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // ROM
            0x0000..=0x7FFF => self.rom[address as usize] = byte,
            // RAM
            0xA000..=0xBFFF => self.ram[(address - 0xA000) as usize] = byte,
            _ => panic!("Illegal address for MBC"),
        }
    }
}
