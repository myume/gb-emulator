use std::{cell::RefCell, rc::Rc};

use crate::{
    cpu::Cycles,
    mmu::InterruptFlag,
    utils::{is_set, reset_bit, set_bit},
};

pub const OAM_BASE_ADDRESS: u16 = 0xFE00;
const OAM_END_ADDRESS: u16 = 0xFE9F;
pub const OAM_SIZE: usize = OAM_END_ADDRESS as usize - OAM_BASE_ADDRESS as usize + 1;

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

enum STATFlags {
    LYCSelect = 6,
    Mode2Select = 5,
    Mode1Select = 4,
    Mode0Select = 3,
    LycEqLy = 2,
    PPUMode = 1, // bits 0 and 1
}

#[derive(Clone, Copy, PartialEq)]
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
pub const GB_SCREEN_HEIGHT: usize = 144;
pub const GB_SCREEN_WIDTH: usize = 160;
const BASE_TILE_WIDTH: usize = 8;
const TILE_MAP_WIDTH: usize = 32;

const BYTES_PER_TILE: usize = 16;
const BYTES_PER_LINE: usize = 2;
const BYTES_PER_SPRITE: usize = 4;

type Color = [u8; 4]; // RGBA8888 format
type Palette = u8;

const MONOCHROME_PALETTE: [Color; 4] = [
    [0xFF, 0xFF, 0xFF, 0xFF], // white
    [0xAA, 0xAA, 0xAA, 0xFF], // light gray
    [0x55, 0x55, 0x55, 0xFF], // dark gray
    [0x00, 0x00, 0x00, 0xFF], // black
];

pub struct PPU {
    mode_clock: usize,
    mode: PPUMode,

    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],

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
    window_line_counter: u8,

    bgp: Palette, // BG palette data

    // OBJ palette 0, 1 data
    obp0: Palette,
    obp1: Palette,

    frame: [Color; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],
    display: [Color; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],

    palette: [Color; 4],
    interrupt_flag: Rc<RefCell<u8>>,
}

impl PPU {
    pub fn new(interrupt_flag: Rc<RefCell<u8>>) -> Self {
        PPU {
            mode_clock: 0,
            mode: PPUMode::OAM,
            oam: [0; OAM_SIZE],
            vram: [0; VRAM_SIZE],

            lcdc: 0,

            ly: 0,
            lyc: 0,
            stat: 0,

            scy: 0,
            scx: 0,

            wy: 0,
            wx: 0,
            window_line_counter: 0,

            bgp: 0,
            obp0: 0,
            obp1: 0,

            frame: [MONOCHROME_PALETTE[0]; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],
            display: [MONOCHROME_PALETTE[0]; GB_SCREEN_HEIGHT * GB_SCREEN_WIDTH],
            palette: MONOCHROME_PALETTE,
            interrupt_flag,
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
            0xFF47 => self.bgp = byte,
            0xFF48 => self.obp0 = byte,
            0xFF49 => self.obp1 = byte,
            0xFF4A => self.wy = byte,
            0xFF4B => self.wx = byte,
            _ => panic!("Invalid PPU Address: {:#06X}", address),
        }
    }

    fn set_ly(&mut self, val: u8) {
        self.ly = val;

        if self.ly == self.lyc {
            self.stat = set_bit(self.stat, STATFlags::LycEqLy as u8);
            if is_set(self.stat, STATFlags::LYCSelect as u8) {
                let flag = *self.interrupt_flag.borrow();
                *self.interrupt_flag.borrow_mut() = set_bit(flag, InterruptFlag::LCD as u8);
            }
        } else {
            self.stat = reset_bit(self.stat, STATFlags::LycEqLy as u8);
        }
    }

    pub fn tick(&mut self, cycles: Cycles) {
        if !is_set(self.lcdc, LCDCBits::LCDEnable as u8) {
            return;
        }
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

                    self.set_ly(self.ly + 1);
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
                    self.set_ly(self.ly + 1);

                    if self.ly as usize == TOTAL_SCANLINES {
                        self.set_ly(0);
                        self.change_mode(PPUMode::OAM);
                    }
                }
            }
        }
    }

    fn change_mode(&mut self, new_mode: PPUMode) {
        self.stat = self.stat & 0b11111100;
        if is_set(self.lcdc, LCDCBits::LCDEnable as u8) {
            self.stat |= new_mode as u8;
        }
        self.mode = new_mode;

        let flag = *self.interrupt_flag.borrow();
        // Request VBlank interrupt
        if new_mode == PPUMode::VBlank {
            self.display.copy_from_slice(&self.frame);
            self.window_line_counter = 0;
            *self.interrupt_flag.borrow_mut() = set_bit(flag, InterruptFlag::VBlank as u8);
        }

        if self.mode != PPUMode::VRAM && is_set(self.stat, self.mode as u8 + 3) {
            *self.interrupt_flag.borrow_mut() = set_bit(flag, InterruptFlag::LCD as u8);
        }
    }

    fn draw_scanline(&mut self) {
        if is_set(self.lcdc, LCDCBits::BgWindowEnable as u8) {
            self.draw_bg();
            if is_set(self.lcdc, LCDCBits::WindowEnable as u8) {
                self.draw_window();
            }
        }
        if is_set(self.lcdc, LCDCBits::OBJEnable as u8) {
            self.draw_sprites();
        }
    }

    fn draw_sprites(&mut self) {
        let relative_ly = self.ly + 16;

        let obj_size = if is_set(self.lcdc, LCDCBits::OBJSize as u8) {
            16
        } else {
            8
        };

        let mut num_sprites = 0;
        for sprite_address in (OAM_BASE_ADDRESS..=OAM_END_ADDRESS).step_by(BYTES_PER_SPRITE) {
            if num_sprites == 10 {
                break;
            }
            let y = self.read_byte(sprite_address);
            let x = self.read_byte(sprite_address + 1);

            let x_start = if x >= BASE_TILE_WIDTH as u8 {
                x - BASE_TILE_WIDTH as u8
            } else {
                0
            };

            // when sprite is visible
            if y <= relative_ly
                && relative_ly < y + obj_size
                && x > 0
                && x_start < GB_SCREEN_WIDTH as u8
            {
                num_sprites += 1;
                let tile_index = self.read_byte(sprite_address + 2);
                let tile_index = if obj_size == 16 {
                    // Bit 0 of tile index for 8x16 objects should be ignored
                    tile_index & 0xFE
                } else {
                    tile_index
                };
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
                let line_offset = line_within_tile as u16 * BYTES_PER_LINE as u16;
                let tile_offset = tile_index as u16 * BYTES_PER_TILE as u16;

                let address = VRAM_BASE_ADDRESS + tile_offset + line_offset;

                let mut p1 = self.read_byte(address);
                let mut p2 = self.read_byte(address + 1);

                if xflip {
                    p1 = p1.reverse_bits();
                    p2 = p2.reverse_bits();
                }

                let pixels = PPU::compose_pixels(p1, p2);

                let frame_base = self.ly as usize * GB_SCREEN_WIDTH + x_start as usize;
                self.draw_pixels(
                    frame_base,
                    pixels,
                    if x < BASE_TILE_WIDTH as u8 {
                        BASE_TILE_WIDTH - x as usize
                    } else {
                        0
                    },
                    (x.min(GB_SCREEN_WIDTH as u8) - x_start) as usize,
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
        let tile_data: u16 = if tile_data_flag {
            VRAM_BASE_ADDRESS
        } else {
            0x9000
        };

        let tile_y = ((self.scy as usize + self.ly as usize) % 256) as usize / BASE_TILE_WIDTH;
        let tile_pixel_offset_y =
            ((self.scy as usize + self.ly as usize) % 256) as u16 % BASE_TILE_WIDTH as u16;

        let mut pixels_drawn = 0;
        while pixels_drawn < GB_SCREEN_WIDTH {
            let x = (self.scx as usize + pixels_drawn) % 256;
            let tile_x = x / BASE_TILE_WIDTH;

            let tile_index = tile_y * TILE_MAP_WIDTH + tile_x;
            let bg_index = self.read_byte(bg_map + tile_index as u16);

            let tile_address = if tile_data_flag {
                // 8000 method
                tile_data + bg_index as u16 * BYTES_PER_TILE as u16
            } else {
                // 8800 method
                tile_data.wrapping_add((bg_index as i8 as i16 * BYTES_PER_TILE as i16) as u16)
            };

            let pixels = PPU::compose_pixels(
                self.read_byte(tile_address + tile_pixel_offset_y * BYTES_PER_LINE as u16),
                self.read_byte(tile_address + tile_pixel_offset_y * BYTES_PER_LINE as u16 + 1),
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
        if self.wy > self.ly || self.wx > GB_SCREEN_WIDTH as u8 || self.wy > GB_SCREEN_HEIGHT as u8
        {
            return; // line isn't in window area yet
        }

        let window_map: u16 = if is_set(self.lcdc, LCDCBits::WindowTileMap as u8) {
            0x9C00
        } else {
            0x9800
        };

        let tile_data_flag = is_set(self.lcdc, LCDCBits::BgWindowTiles as u8);
        let tile_data: u16 = if tile_data_flag {
            VRAM_BASE_ADDRESS
        } else {
            0x9000
        };

        let tile_y = self.window_line_counter as usize / BASE_TILE_WIDTH;
        let tile_pixel_offset_y = (self.ly - self.wy) as u16 % BASE_TILE_WIDTH as u16;

        let mut x = if self.wx >= 7 {
            self.wx as usize - 7
        } else {
            0
        };
        while x < GB_SCREEN_WIDTH {
            let tile_x = if self.wx >= 7 {
                x + 7 - self.wx as usize
            } else {
                x
            } / BASE_TILE_WIDTH;

            let tile_index = tile_y * TILE_MAP_WIDTH + tile_x;
            let tile_data_index = self.read_byte(window_map + tile_index as u16);

            let tile_address = if tile_data_flag {
                // 8000 method
                tile_data + tile_data_index as u16 * BYTES_PER_TILE as u16
            } else {
                // 8800 method
                tile_data
                    .wrapping_add((tile_data_index as i8 as i16 * BYTES_PER_TILE as i16) as u16)
            };

            let pixels = PPU::compose_pixels(
                self.read_byte(tile_address + tile_pixel_offset_y * BYTES_PER_LINE as u16),
                self.read_byte(tile_address + tile_pixel_offset_y * BYTES_PER_LINE as u16 + 1),
            );

            let start_x_offset = x % BASE_TILE_WIDTH;
            let pixels_to_draw = (BASE_TILE_WIDTH - start_x_offset).min(GB_SCREEN_WIDTH - x);

            self.draw_pixels(
                self.ly as usize * GB_SCREEN_WIDTH + x,
                pixels,
                start_x_offset,
                pixels_to_draw,
                self.bgp,
                None,
            );

            x += pixels_to_draw;
        }
        self.window_line_counter += 1;
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

            let color = self.get_color_from_palette(palette, color_index);
            let pixel_address = frame_base + i - pixels_start_offset;
            if priority.is_some() {
                if color_index != 0 {
                    self.frame[pixel_address] = color;
                }
            } else {
                self.frame[pixel_address] = color;
            }
        }
    }

    fn get_color_from_palette(&self, palette: Palette, color_index: u8) -> Color {
        let color_id = (palette >> (color_index * 2) & 0b11) as u8;
        self.palette[color_id as usize]
    }

    pub fn pixel_data(&self) -> &[u8] {
        self.display.as_flattened()
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
        let intflag = Rc::new(RefCell::new(0));
        let mut ppu = PPU::new(intflag.clone());
        let palettte = 0b11100100;
        ppu.draw_pixels(0, 0b0010111111111000, 0, 8, palettte, None);

        assert_eq!(
            ppu.frame[0..8],
            [0b00, 0b10, 0b11, 0b11, 0b11, 0b11, 0b10, 0b00].map(|id| MONOCHROME_PALETTE[id])
        );

        let mut ppu = PPU::new(intflag.clone());
        ppu.draw_pixels(0, 0b0010111111111000, 0, 2, palettte, None);

        assert_eq!(
            ppu.frame[0..2],
            [0b00, 0b10].map(|id| MONOCHROME_PALETTE[id])
        );

        let mut ppu = PPU::new(intflag.clone());
        ppu.draw_pixels(0, 0b0010111111111000, 2, 2, palettte, None);

        assert_eq!(
            ppu.frame[0..2],
            [0b11, 0b11].map(|id| MONOCHROME_PALETTE[id])
        );
    }
}
