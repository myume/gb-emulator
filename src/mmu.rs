use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::Cartridge,
    cpu::Cycles,
    joypad::Joypad,
    ppu::{OAM_BASE_ADDRESS, OAM_SIZE, PPU},
    serial::Serial,
    timer::Timer,
    utils::compose_bytes,
};

const WRAM_SIZE: usize = 0xE000 - 0xC000;
const HRAM_SIZE: usize = 0xFFFF - 0xFF80;

pub struct MMU {
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
    dma: u8, // OAM DMA source address & start

    stub_audio: [u8; 0xFF26 - 0xFF10 + 1], // TODO: remove this when implemented audio
    pub interrupt_enable: u8,
    pub interrupt_flag: Rc<RefCell<u8>>,

    pub ppu: PPU,
    pub joypad: Joypad,
    pub timer: Timer,
    pub cartridge: Cartridge,
    pub serial: Serial,

    #[cfg(feature = "test")]
    test_ram: [u8; 0xFFFF + 1],
}

#[derive(Debug, Clone, Copy)]
pub enum InterruptFlag {
    Joypad = 4,
    Serial = 3,
    Timer = 2,
    LCD = 1,
    VBlank = 0,
}

impl InterruptFlag {
    pub fn iter() -> impl Iterator<Item = InterruptFlag> {
        [
            InterruptFlag::VBlank,
            InterruptFlag::LCD,
            InterruptFlag::Timer,
            InterruptFlag::Serial,
            InterruptFlag::Joypad,
        ]
        .iter()
        .copied()
    }
}

impl MMU {
    pub fn new(cartridge: Cartridge, print_serial: bool) -> Self {
        let interrupt_flag = Rc::new(RefCell::new(0xE1));
        MMU {
            wram: [0; WRAM_SIZE],
            hram: [0; HRAM_SIZE],
            dma: 0xFF,
            stub_audio: [0; 0xFF26 - 0xFF10 + 1],
            ppu: PPU::new(interrupt_flag.clone()),
            joypad: Joypad::new(interrupt_flag.clone()),
            timer: Timer::new(interrupt_flag.clone()),
            serial: Serial::new(print_serial),
            cartridge,

            interrupt_enable: 0,
            interrupt_flag,

            #[cfg(feature = "test")]
            test_ram: [0; 0xFFFF + 1],
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        #[cfg(feature = "test")]
        return self.test_ram[address as usize];

        match address {
            // cartridge
            0x0000..=0x7FFF => self.cartridge.mbc.read_byte(address),
            // VRAM
            0x8000..=0x9FFF => self.ppu.read_byte(address),
            // External RAM (from cartridge)
            0xA000..=0xBFFF => self.cartridge.mbc.read_byte(address),
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => self.read_byte(address - 0x2000),
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => self.ppu.read_byte(address),
            // Not usable
            0xFEA0..=0xFEFF => 0xFF,
            // Interrupt flag (IF)
            0xFF0F => *self.interrupt_flag.borrow(),
            // LCD control and flags
            0xFF46 => self.dma,
            0xFF40..=0xFF4B => self.ppu.read_byte(address),
            // I/O Registers
            0xFF00 => self.joypad.read(),
            0xFF01..=0xFF02 => self.serial.read_byte(address),
            0xFF04..=0xFF07 => self.timer.read_byte(address),
            0xFF10..=0xFF26 => self.stub_audio[(address - 0xFF10) as usize],
            // HRAM (high RAM)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            // Interrupt Enable register (IE)
            0xFFFF => self.interrupt_enable,
            _ => 0xFF,
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let low = self.read_byte(address);
        let high = self.read_byte(address.wrapping_add(1));
        compose_bytes(high, low)
    }

    pub fn write_byte(&mut self, address: u16, byte: u8) {
        #[cfg(feature = "test")]
        return self.test_ram[address as usize] = byte;

        match address {
            // cartridge
            0x0000..=0x7FFF => self.cartridge.mbc.write_byte(address, byte),
            // VRAM
            0x8000..=0x9FFF => self.ppu.write_byte(address, byte),
            // External RAM (from cartridge)
            0xA000..=0xBFFF => self.cartridge.mbc.write_byte(address, byte),
            // WRAM
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = byte,
            // Echo RAM (prohibited)
            0xE000..=0xFDFF => self.write_byte(address - 0x2000, byte),
            // OAM (Object attribute memory)
            0xFE00..=0xFE9F => self.ppu.write_byte(address, byte),
            // Not usable
            0xFEA0..=0xFEFF => {}
            // OAM DMA Transfer
            0xFF46 => {
                self.dma = byte;
                let source_address = self.dma as u16 * 0x100;
                for i in 0..OAM_SIZE {
                    let copied_byte = self.read_byte(source_address + i as u16);
                    self.write_byte(OAM_BASE_ADDRESS + i as u16, copied_byte);
                }
            }
            // LCD control and flags
            0xFF40..=0xFF4B => self.ppu.write_byte(address, byte),
            // Interrupt flag (IF)
            0xFF0F => *self.interrupt_flag.borrow_mut() = byte,
            // I/O Registers
            0xFF00 => self.joypad.write(byte),
            0xFF01..=0xFF02 => self.serial.write_byte(address, byte),
            0xFF04..=0xFF07 => self.timer.write_byte(address, byte),
            0xFF10..=0xFF26 => self.stub_audio[(address - 0xFF10) as usize] = byte,
            // HRAM (high RAM)
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = byte,
            // Interrupt Enable register (IE)
            0xFFFF => self.interrupt_enable = byte,
            _ => {}
        }
    }

    pub fn write_word(&mut self, address: u16, word: u16) {
        let low = word & 0x00FF;
        let high = word >> 8;
        self.write_byte(address, low as u8);
        self.write_byte(address.wrapping_add(1), high as u8);
    }

    pub fn tick(&mut self, cycles: Cycles) {
        self.ppu.tick(cycles);
        self.timer.tick(cycles);
    }
}
