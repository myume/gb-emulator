use crate::cpu::Cycles;

const OAM_SIZE: usize = 0xFEA0 - 0xFE00;
const VRAM_SIZE: usize = 0xA000 - 0x8000;

enum PPUMode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

pub struct PPU {
    mode_clock: usize,
    mode: PPUMode,

    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

    ly: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode_clock: 0,
            mode: PPUMode::Mode2,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],
            ly: 0,
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

    pub fn tick(&mut self, cycles: Cycles) {
        self.mode_clock = self.mode_clock + cycles;

        match self.mode {
            PPUMode::Mode0 => todo!(),
            PPUMode::Mode1 => todo!(),
            PPUMode::Mode2 => todo!(),
            PPUMode::Mode3 => todo!(),
        }
    }
}
