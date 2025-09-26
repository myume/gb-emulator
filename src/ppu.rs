use crate::{cpu::Cycles, utils::is_set};

const OAM_BASE_ADDRESS: u16 = 0xFE00;
const OAM_END_ADDRESS: u16 = 0xFE9F;
const OAM_SIZE: usize = OAM_END_ADDRESS as usize - OAM_BASE_ADDRESS as usize + 1;

const VRAM_BASE_ADDRESS: u16 = 0x8000;
const VRAM_END_ADDRESS: u16 = 0x9FFF;
const VRAM_SIZE: usize = VRAM_END_ADDRESS as usize - VRAM_BASE_ADDRESS as usize + 1;

enum SpriteFlags {
    Priority = 7,
    YFlip = 6,
    XFlip = 5,
    DMGPalette = 4,
}

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

const OAM_CYCLE_LENGTH: usize = 80;
const VRAM_CYCLE_LENGTH: usize = 172;
const HBLANK_CYCLE_LENGTH: usize = 204;
const VBLANK_CYCLE_LENGTH: usize = 456;

const TOTAL_SCANLINES: usize = 154;
const GB_SCREEN_HEIGHT: usize = 144;
const GB_SCREEN_WIDTH: usize = 160;
const BASE_TILE_WIDTH: usize = 8;
const TILE_MAP_WIDTH: usize = 32;

const BYTES_PER_TILE: usize = 16;
const BYTES_PER_LINE: usize = 2;
const BYTES_PER_SPRITE: usize = 4;

type Color = u32; // RGBA8888 format
type Palette = u8;

const MONOCHROME_PALETTE: [Color; 4] = [0xFFFFFFFF, 0xAAAAAAFF, 0x555555FF, 0x000000FF];

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

    bgp: Palette, // BG palette data

    // OBJ palette 0, 1 data
    obp0: Palette,
    obp1: Palette,

    frame: [Color; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],

    palette: [Color; 4],
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
            palette: MONOCHROME_PALETTE,
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
            0xFF44 => {
                #[cfg(feature = "gb_doctor")]
                return 0x90;
                self.ly
            }
            0xFF45 => self.lyc,
            0xFF46 => self.dma,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => panic!("Invalid PPU Address: {:#06X}", address),
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
            _ => panic!("Invalid PPU Address: {:#06X}", address),
        }
    }

    pub fn tick(&mut self, cycles: Cycles) {
        self.mode_clock = self.mode_clock + cycles;

        match self.mode {
            PPUMode::OAM => {
                if self.mode_clock >= OAM_CYCLE_LENGTH {
                    self.mode_clock %= OAM_CYCLE_LENGTH;
                    self.change_mode(PPUMode::VRAM);
                }
            }
            PPUMode::VRAM => {
                if self.mode_clock >= VRAM_CYCLE_LENGTH {
                    self.mode_clock %= VRAM_CYCLE_LENGTH;
                    self.change_mode(PPUMode::HBlank);
                }
            }
            PPUMode::HBlank => {
                if self.mode_clock >= HBLANK_CYCLE_LENGTH {
                    self.mode_clock %= HBLANK_CYCLE_LENGTH;

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
                if self.mode_clock >= VBLANK_CYCLE_LENGTH {
                    self.mode_clock %= VBLANK_CYCLE_LENGTH;
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

        let relative_ly = self.ly + 16;

        let obj_size = if is_set(self.lcdc, LCDCBits::OBJSize as u8) {
            16
        } else {
            8
        };

        for sprite_address in (OAM_BASE_ADDRESS..=OAM_END_ADDRESS).step_by(BYTES_PER_SPRITE) {
            let y = self.read_byte(sprite_address);
            let x = self.read_byte(sprite_address + 1);

            let x_start = if x >= BASE_TILE_WIDTH as u8 {
                x - BASE_TILE_WIDTH as u8
            } else {
                0
            };
            // should be 0 when x is 0 or when x >= 168.
            let pixels_to_draw = x.max((GB_SCREEN_WIDTH + BASE_TILE_WIDTH) as u8) - x_start;

            if y <= relative_ly && relative_ly < y + obj_size && pixels_to_draw > 0 {
                let tile_index = self.read_byte(sprite_address + 2);
                let sprite_flags = self.read_byte(sprite_address + 3);

                let yflip = is_set(sprite_flags, SpriteFlags::YFlip as u8);
                let xflip = is_set(sprite_flags, SpriteFlags::XFlip as u8);

                let priority = is_set(sprite_flags, SpriteFlags::Priority as u8);

                let palette = if is_set(sprite_flags, SpriteFlags::DMGPalette as u8) {
                    self.obp1
                } else {
                    self.obp0
                };

                let mut line_within_tile = relative_ly - y;
                if yflip {
                    line_within_tile = obj_size - line_within_tile - 1;
                }
                let line_offset = (line_within_tile * BYTES_PER_LINE as u8) as u16;
                let tile_offset = (tile_index * BYTES_PER_TILE as u8) as u16;

                let address = VRAM_BASE_ADDRESS + tile_offset + line_offset;

                let mut pixels =
                    PPU::compose_pixels(self.read_byte(address), self.read_byte(address + 1));
                if xflip {
                    pixels = pixels.reverse_bits();
                }

                let frame_base = self.ly as usize * GB_SCREEN_WIDTH + x_start as usize;

                self.draw_pixels(
                    frame_base,
                    pixels,
                    x_start.into(),
                    pixels_to_draw.into(),
                    palette,
                    Some(priority),
                );
            }
        }
    }

    fn draw_bg(&mut self) {
        let bg_map: u16 = if is_set(self.lcdc, LCDCBits::BgTileMap as u8) {
            0x9C00
        } else {
            0x9800
        };

        let tile_data_flag = is_set(self.lcdc, LCDCBits::BgWindowTiles as u8);
        let bg_data: u16 = if tile_data_flag {
            VRAM_BASE_ADDRESS
        } else {
            0x9000
        };

        let tile_y = (self.scy as usize + self.ly as usize % 256) as usize / BASE_TILE_WIDTH;
        let tile_pixel_offset_y =
            (self.scy as usize + self.ly as usize % 256) as u16 % BASE_TILE_WIDTH as u16;

        let mut pixels_drawn = 0;
        while pixels_drawn < GB_SCREEN_WIDTH {
            let x = (self.scx as usize + pixels_drawn) % 256;
            let tile_x = x / BASE_TILE_WIDTH;

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

            let pixels = PPU::compose_pixels(
                self.read_byte(bg_address + tile_pixel_offset_y * BYTES_PER_LINE as u16),
                self.read_byte(bg_address + tile_pixel_offset_y * BYTES_PER_LINE as u16 + 1),
            );

            let start_x_offset = x % BASE_TILE_WIDTH;
            let pixels_to_draw =
                (BASE_TILE_WIDTH - start_x_offset).min(GB_SCREEN_WIDTH - pixels_drawn);

            self.draw_pixels(
                self.ly as usize * GB_SCREEN_WIDTH + pixels_drawn,
                pixels,
                start_x_offset,
                pixels_to_draw,
                self.bgp,
                None,
            );

            pixels_drawn += pixels_to_draw;
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
        palette: Palette,
        priority: Option<bool>,
    ) {
        for i in pixels_start_offset..pixels_start_offset + pixels_to_draw {
            let shift = 2 * (BASE_TILE_WIDTH - i - 1);
            let color_index = (pixels >> shift & 0b11) as u8;

            if Some(true) == priority
                && self.frame[frame_base + i] != self.get_color_from_palette(self.bgp, 0)
            {
                continue;
            }

            self.frame[frame_base + i] = self.get_color_from_palette(palette, color_index);
        }
    }

    fn get_color_from_palette(&self, palette: Palette, color_index: u8) -> Color {
        let color_id = (palette >> (color_index * 2) & 0b11) as u8;
        self.palette[color_id as usize]
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
        let palettte = 0b11100100;
        ppu.draw_pixels(0, 0b0010111111111000, 0, 8, palettte, None);

        assert_eq!(
            ppu.frame[0..8],
            [0b00, 0b10, 0b11, 0b11, 0b11, 0b11, 0b10, 0b00].map(|id| MONOCHROME_PALETTE[id])
        );

        let mut ppu = PPU::new();
        ppu.draw_pixels(0, 0b0010111111111000, 0, 2, palettte, None);

        assert_eq!(
            ppu.frame[0..2],
            [0b00, 0b10].map(|id| MONOCHROME_PALETTE[id])
        );
    }
}
