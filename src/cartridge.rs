pub struct Cartridge {
    pub mbc: Box<dyn MBC>,
}

pub trait MBC {}

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

impl MBC for NoMBC {}
