use crate::{cpu::Cycles, utils::is_set};

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

#[derive(Clone, Copy)]
enum PPUMode {
    HBlank = 0, // Mode0
    VBlank = 1, // Mode1
    OAM = 2,    // Mode2
    VRAM = 3,   // Mode3
}

const OAM_LENGTH: usize = 80;
const VRAM_LENGTH: usize = 172;
const HBLANK_LENGTH: usize = 204;
const VBLANK_LENGTH: usize = 456;

const TOTAL_SCANLINES: usize = 154;
const GB_SCREEN_HEIGHT: usize = 144;
const GB_SCREEN_WIDTH: usize = 160;
const BG_TILE_WIDTH: usize = 8;
const TILE_MAP_WIDTH: usize = 32;
const BYTES_PER_TILE: usize = 2;

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

    frame: [u8; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            mode_clock: 0,
            mode: PPUMode::OAM,
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

            frame: [0; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],
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
            PPUMode::OAM => {
                if self.mode_clock >= OAM_LENGTH {
                    self.mode_clock %= OAM_LENGTH;
                    self.change_mode(PPUMode::VRAM);
                }
            }
            PPUMode::VRAM => {
                if self.mode_clock >= VRAM_LENGTH {
                    self.mode_clock %= VRAM_LENGTH;
                    self.change_mode(PPUMode::HBlank);
                }
            }
            PPUMode::HBlank => {
                if self.mode_clock >= HBLANK_LENGTH {
                    self.mode_clock %= HBLANK_LENGTH;

                    self.draw_scanline();

                    self.ly += 1;
                    if self.ly as usize == GB_SCREEN_HEIGHT {
                        self.change_mode(PPUMode::VBlank);
                    } else {
                        self.change_mode(PPUMode::OAM);
                    }
                }
            }
            PPUMode::VBlank => {
                if self.mode_clock >= VBLANK_LENGTH {
                    self.mode_clock %= VBLANK_LENGTH;
                    self.ly += 1;

                    if self.ly as usize == TOTAL_SCANLINES {
                        self.change_mode(PPUMode::OAM);
                        self.ly = 0;
                    }
                }
            }
        }
    }

    fn change_mode(&mut self, new_mode: PPUMode) {
        self.stat = self.stat & 0b11111100 | new_mode as u8;
        self.mode = new_mode;
    }

    fn draw_scanline(&mut self) {
        self.draw_bg();
        self.draw_window();
        self.draw_sprites();
    }

    fn draw_sprites(&mut self) {
        if !is_set(self.lcdc, LCDCBits::OBJEnable as u8) {
            return;
        }
    }

    fn draw_bg(&mut self) {
        let bg_map: u16 = if is_set(self.lcdc, LCDCBits::BgTileMap as u8) {
            0x9C00
        } else {
            0x9800
        };

        let tile_data_flag = is_set(self.lcdc, LCDCBits::BgWindowTiles as u8);
        let bg_data: u16 = if tile_data_flag { 0x8000 } else { 0x9000 };

        let tile_y = (self.scy + self.ly) as usize / BG_TILE_WIDTH;

        let mut i = 0;
        while i < GB_SCREEN_WIDTH {
            let x = (self.scx as usize + i) % 256;
            let tile_x = x / BG_TILE_WIDTH;

            let tile_index = tile_y * TILE_MAP_WIDTH + tile_x;
            let bg_index = self.read_byte(bg_map + tile_index as u16);

            let bg_address = bg_data
                + if tile_data_flag {
                    // 8000 method
                    bg_index as u16
                } else {
                    // 8800 method
                    bg_index as i8 as i16 as u16
                } * BYTES_PER_TILE as u16; // mutiple index by number of bytes per tile to get correct address

            let pixels =
                PPU::compose_pixels(self.read_byte(bg_address), self.read_byte(bg_address + 1));

            let start_x_offset = tile_x % BG_TILE_WIDTH;
            let pixels_to_draw = (BG_TILE_WIDTH - start_x_offset).min(GB_SCREEN_WIDTH - i);

            self.draw_pixels(
                self.ly as usize * GB_SCREEN_WIDTH + i,
                pixels,
                start_x_offset,
                pixels_to_draw,
            );

            i += pixels_to_draw;
        }
    }

    fn compose_pixels(first: u8, second: u8) -> u16 {
        let mut res = 0;
        for i in 0..8 {
            let left = is_set(first, i) as u16;
            let right = is_set(second, i) as u16;

            res |= (left as u16) << 2 * i;
            res |= (right as u16) << 2 * i + 1;
        }

        res
    }

    fn draw_window(&mut self) {
        if !is_set(self.lcdc, LCDCBits::WindowEnable as u8) {
            return;
        }
    }

    fn draw_pixels(
        &mut self,
        frame_base: usize,
        pixels: u16,
        pixels_start_offset: usize,
        pixels_to_draw: usize,
    ) {
        for i in pixels_start_offset..pixels_start_offset + pixels_to_draw {
            let shift = 2 * (BG_TILE_WIDTH - i - 1);
            self.frame[frame_base + i] = ((pixels & 0b11 << shift) >> shift) as u8;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_compose_pixels() {
        assert_eq!(PPU::compose_pixels(0x3C, 0x7E), 0b0010111111111000);
        assert_eq!(PPU::compose_pixels(0x42, 0x42), 0b0011000000001100);
        assert_eq!(PPU::compose_pixels(0x7E, 0x5E), 0b0011011111111100);
    }

    #[test]
    fn test_draw_pixels() {
        let mut ppu = PPU::new();
        ppu.draw_pixels(0, 0b0010111111111000, 0, 8);

        assert_eq!(
            ppu.frame[0..8],
            [0b00, 0b10, 0b11, 0b11, 0b11, 0b11, 0b10, 0b00]
        );

        let mut ppu = PPU::new();
        ppu.draw_pixels(0, 0b0010111111111000, 0, 2);

        assert_eq!(ppu.frame[0..2], [0b00, 0b10]);
    }
}
