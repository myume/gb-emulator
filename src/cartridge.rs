use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use crate::cartridge::mbc1::MBC1;
use crate::cartridge::mbc3::MBC3;

mod mbc1;
mod mbc3;

pub const ROM_BANK_SIZE: usize = 0x4000;
pub const RAM_BANK_SIZE: usize = 0x2000;

pub struct Cartridge {
    pub title: String,

    pub mbc: Box<dyn MBC>,
}

impl Cartridge {
    pub fn load_cartridge(path: &Path) -> io::Result<Cartridge> {
        let mut f = File::open(path)?;
        let mut rom = Vec::new();

        f.read_to_end(&mut rom)?;

        let title = String::from_utf8_lossy(&rom[0x0134..0x0143]).to_string();

        // if rom[0x143] == 0xC0 {
        //     panic!("CGB cartridge not supported");
        // }

        let cart_type = rom[0x147];
        // let rom_size = ROM_BANK_SIZE * 2 * (1 << rom[0x148]);
        let ram_size = RAM_BANK_SIZE
            * match rom[0x149] {
                0x00 => 0,
                0x01 => unreachable!("Unused RAM size"),
                0x02 => 1,
                0x03 => 4,
                0x04 => 16,
                0x05 => 64,
                _ => unreachable!("Invalid RAM size"),
            };

        let mbc: Box<dyn MBC> = match cart_type {
            0x00 => {
                let mut mbc = NoMBC::new();
                mbc.load_rom(rom.as_slice());
                Box::new(mbc)
            }
            0x01..=0x03 => Box::new(MBC1::new(rom, ram_size)),
            0x0F..=0x13 => Box::new(MBC3::new(rom, ram_size)),
            _ => panic!("Unsupported cartridge type: {:#04X}", cart_type),
        };

        #[cfg(not(feature = "gb_doctor"))]
        println!("Loaded ROM: {}", title);

        Ok(Cartridge { title, mbc })
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

    fn load_rom(&mut self, data: &[u8]) {
        self.rom.copy_from_slice(data);
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
