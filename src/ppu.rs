static OAM_SIZE: usize = 0xFEA0 - 0xFE00;
static VRAM_SIZE: usize = 0xA000 - 0x8000;

pub struct PPU {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            _ => panic!("Invalid PPU Address"),
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = byte,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = byte,
            _ => panic!("Invalid PPU Address"),
        }
    }
}
