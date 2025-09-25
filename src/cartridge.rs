use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

pub struct Cartridge {
    pub title: String,
    pub bytes: usize,

    pub mbc: Box<dyn MBC>,
}

impl Cartridge {
    pub fn load_cartridge(path: &Path) -> io::Result<Cartridge> {
        let mut f = File::open(path)?;
        let mut data = Vec::new();

        let bytes = f.read_to_end(&mut data)?;

        let title = String::from_utf8_lossy(&data[0x0134..0x0143]).to_string();

        let cart_type = data[0x147];
        let rom_size = data[0x148];
        let ram_size = data[0x149];

        let mbc = match cart_type {
            0x00 => NoMBC::new(),
            _ => panic!("Unsupported cartridge type: {:#04X}", cart_type),
        };

        Ok(Cartridge {
            title,
            bytes,
            mbc: Box::new(mbc),
        })
    }
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
