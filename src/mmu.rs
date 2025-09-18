use crate::{cartridge::Cartridge, ppu::PPU, utils::compose_bytes};

static WRAM_SIZE: usize = 0xE000 - 0xC000;
static HRAM_SIZE: usize = 0xFFFF - 0xFF80;
static OAM_SIZE: usize = 0xFEA0 - 0xFE00;

pub struct MMU {
    stub_ram: [u8; 0xFFFF], // TODO: remove after everything is implemented

    wram: [u8; WRAM_SIZE],
    oam: [u8; OAM_SIZE],
    hram: [u8; HRAM_SIZE],
    pub interrupt_enable: u8,
    pub interrupt_flag: u8,

    ppu: PPU,
    cartridge: Cartridge,
}

impl MMU {
    pub fn new(cartridge: Cartridge) -> Self {
        MMU {
            stub_ram: [0; 0xFFFF],

            wram: [0; WRAM_SIZE],
            oam: [0; OAM_SIZE],
            hram: [0; HRAM_SIZE],
            interrupt_enable: 0,
            interrupt_flag: 0,
            ppu: PPU::new(),
            cartridge,
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // cartridge
            0x0000..=0x7FFF => self.stub_ram[address as usize],
            // VRAM
            0x8000..=0x9FFF => self.stub_ram[address as usize],
            // External RAM (from cartridge)
            0xA000..=0xBFFF => self.stub_ram[address as usize],
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => self.stub_ram[address as usize],
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            // Not usable
            0xFEA0..=0xFEFF => self.stub_ram[address as usize],
            // Interrupt flag (IF)
            0xFF0F => self.interrupt_flag,
            // I/O Registers
            0xFF00..=0xFF7F => self.stub_ram[address as usize],
            // HRAM (high RAM)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            // Interrupt Enable register (IE)
            0xFFFF => self.interrupt_enable,
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address.wrapping_add(1));
        compose_bytes(high, low)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            // cartridge
            0x0000..=0x7FFF => self.stub_ram[address as usize] = byte,
            // VRAM
            0x8000..=0x9FFF => self.stub_ram[address as usize] = byte,
            // External RAM (from cartridge)
            0xA000..=0xBFFF => self.stub_ram[address as usize] = byte,
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = byte,
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => self.stub_ram[address as usize] = byte,
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize] = byte,
            // Not usable
            0xFEA0..=0xFEFF => self.stub_ram[address as usize] = byte,
            // Interrupt flag (IF)
            0xFF0F => self.interrupt_flag = byte,
            // I/O Registers
            0xFF00..=0xFF7F => self.stub_ram[address as usize] = byte,
            // HRAM (high RAM)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = byte,
            // Interrupt Enable register (IE)
            0xFFFF => self.interrupt_enable = byte,
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let low = word & 0x00FF;
        let high = word >> 8;
        self.write_byte(address, low as u8);
        self.write_byte(address.wrapping_add(1), high as u8);
    }
}
