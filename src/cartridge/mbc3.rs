use std::time::Instant;

use crate::{
    cartridge::{MBC, RAM_BANK_SIZE, ROM_BANK_SIZE},
    utils::{is_set, reset_bit, set_bit},
};

struct RTCRegs {
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_counter_low: u8,
    day_counter_high: u8,
}

pub struct MBC3 {
    rom: Vec<u8>,
    ram: Vec<u8>,

    rom_bank_number: u8, // 7 bits now
    ram_rtc_enable: bool,
    ram_rtc_select: u8,

    rtc: RTCRegs,
    latch: u8,

    init_time: Instant,
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
                seconds: 0,
                minutes: 0,
                hours: 0,
                day_counter_low: 0,
                day_counter_high: 0,
            },
            latch: 0xFF,
            init_time: Instant::now(),
        }
    }

    fn refresh_clock(&mut self) {
        // timer is stopped
        if is_set(self.rtc.day_counter_high, 6) {
            return;
        }

        self.rtc.day_counter_high = set_bit(self.rtc.day_counter_high, 6);

        let duration = self.init_time.elapsed();

        let secs = duration.as_secs();
        self.rtc.seconds = (secs % 60) as u8;
        let mins = secs / 60;
        self.rtc.minutes = (mins % 60) as u8;
        let hours = mins / 60;
        self.rtc.hours = (hours % 24) as u8;
        let days = hours / 24;
        self.rtc.day_counter_low = (days & 0xFF) as u8;

        if days & 0x0100 != 0 {
            self.rtc.day_counter_high = set_bit(self.rtc.day_counter_high, 0);
        }
        if days > 0x1FF {
            self.rtc.day_counter_high = set_bit(self.rtc.day_counter_high, 7);
        }

        self.rtc.day_counter_high = reset_bit(self.rtc.day_counter_high, 6);
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
                        0x08 => self.rtc.seconds,
                        0x09 => self.rtc.minutes,
                        0x0A => self.rtc.hours,
                        0x0B => self.rtc.day_counter_low,
                        0x0C => self.rtc.day_counter_high,
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
                if self.latch == 0x00 && byte == 0x01 {
                    self.refresh_clock();
                }
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
                        0x08..=0x0C => {
                            self.rtc.day_counter_high = set_bit(self.rtc.day_counter_high, 6);
                            match self.ram_rtc_select {
                                0x08 => self.rtc.seconds = byte,
                                0x09 => self.rtc.minutes = byte,
                                0x0A => self.rtc.hours = byte,
                                0x0B => self.rtc.day_counter_low = byte,
                                0x0C => self.rtc.day_counter_high = byte,
                                _ => panic!("Invalid ram_rtc_select: {:#04X}", self.ram_rtc_select),
                            }
                            self.rtc.day_counter_high = reset_bit(self.rtc.day_counter_high, 6);
                        }
                        _ => panic!("Invalid ram_rtc_select: {:#04X}", self.ram_rtc_select),
                    }
                }
            }
            _ => panic!("Invalid MBC3 address: {:#06X}", address),
        }
    }
}
