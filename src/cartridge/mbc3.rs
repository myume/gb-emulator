use crate::cartridge::{MBC, RAM_BANK_SIZE, ROM_BANK_SIZE};

struct RTCRegs {
    s: u8,
    m: u8,
    h: u8,
    dl: u8,
    dh: u8,
}

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,

    rom_bank_number: u8, // 7 bits now
    ram_rtc_enable: bool,
    ram_rtc_select: u8,

    rtc: RTCRegs,
    latch: u8,
}

impl MBC3 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        MBC3 {
            rom,
            ram: vec![0; ram_size],

            rom_bank_number: 0,
            ram_rtc_enable: false,
            ram_rtc_select: 0,

            rtc: RTCRegs {
                s: 0,
                m: 0,
                h: 0,
                dl: 0,
                dh: 0,
            },
            latch: 0xFF,
        }
    }
}

impl MBC for MBC3 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                self.rom
                    [(self.rom_bank_number.max(1) - 1) as usize * ROM_BANK_SIZE + address as usize]
            }

            0xA000..=0xBFFF => {
                if self.ram_rtc_enable {
                    match self.ram_rtc_select {
                        0x00..=0x07 => {
                            self.ram[self.ram_rtc_select as usize * RAM_BANK_SIZE
                                + address as usize
                                - 0xA000]
                        }
                        0x08 => self.rtc.s,
                        0x09 => self.rtc.m,
                        0x0A => self.rtc.h,
                        0x0B => self.rtc.dl,
                        0x0C => self.rtc.dh,
                        _ => panic!("Invalid ram_rtc_select: {:#04X}", self.ram_rtc_select),
                    }
                } else {
                    0xFF
                }
            }

            _ => panic!("Invalid MBC3 address: {:#06X}", address),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_rtc_enable = (byte & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                self.rom_bank_number = byte & 0b1111111;
            }
            0x4000..=0x5FFF => {
                self.ram_rtc_select = byte;
            }
            0x6000..=0x7FFF => {
                if self.latch == 0x00 && byte == 0x01 {}
                self.latch = byte;
            }
            0xA000..=0xBFFF => {
                if self.ram_rtc_enable {
                    match self.ram_rtc_select {
                        0x00..=0x07 => {
                            self.ram[self.ram_rtc_select as usize * RAM_BANK_SIZE
                                + address as usize
                                - 0xA000] = byte
                        }
                        0x08 => self.rtc.s = byte,
                        0x09 => self.rtc.m = byte,
                        0x0A => self.rtc.h = byte,
                        0x0B => self.rtc.dl = byte,
                        0x0C => self.rtc.dh = byte,
                        _ => panic!("Invalid ram_rtc_select: {:#04X}", self.ram_rtc_select),
                    }
                }
            }
            _ => panic!("Invalid MBC3 address: {:#06X}", address),
        }
    }
}
