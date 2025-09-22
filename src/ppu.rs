use crate::cpu::Cycles;

const OAM_SIZE: usize = 0xFEA0 - 0xFE00;
const VRAM_SIZE: usize = 0xA000 - 0x8000;

enum LCDCBits {
    LCDEnable = 7,
    WindowTileMap = 6,
    WindowEnable = 5,
    BgWindowTiles = 4,
    BgTileMap = 3,
    OBJSize = 2,
    OBJEnable = 1,
    BgWindowEnable = 0,
}

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

    dma: u8, // OAM DMA source address & start

    lcdc: u8, // LCD control

    ly: u8,   // LCD Y coordinate [read-only]
    lyc: u8,  // LY compare
    stat: u8, // LCD status

    // Background viewport Y position, X position
    scy: u8,
    scx: u8,

    // Window Y position, X position plus 7
    wy: u8,
    wx: u8,

    bgp: u8, // BG palette data

    // OBJ palette 0, 1 data
    obp0: u8,
    obp1: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode_clock: 0,
            mode: PPUMode::Mode2,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],
            dma: 0,

            lcdc: 0,

            ly: 0,
            lyc: 0,
            stat: 0,

            scy: 0,
            scx: 0,

            wy: 0,
            wx: 0,

            bgp: 0,
            obp0: 0,
            obp1: 0,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            0xFF40 => self.lcdc,
            0xFF41 => self.stat,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("Invalid PPU Address: 0x{:X}", address),
        }
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize] = byte,
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = byte,
            0xFF40 => self.lcdc = byte,
            0xFF41 => self.stat = byte,
            0xFF42 => self.scy = byte,
            0xFF43 => self.scx = byte,
            0xFF44 => self.ly = byte,
            0xFF45 => self.lyc = byte,
            0xFF46 => self.dma = byte,
            0xFF47 => self.bgp = byte,
            0xFF48 => self.obp0 = byte,
            0xFF49 => self.obp1 = byte,
            0xFF4A => self.wy = byte,
            0xFF4B => self.wx = byte,
            _ => panic!("Invalid PPU Address: 0x{:X}", address),
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
