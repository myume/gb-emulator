use crate::cartridge::{MBC, RAM_BANK_SIZE, ROM_BANK_SIZE};

enum MBC1BankingMode {
    Simple = 0,
    Advanced = 1,
}

pub struct MBC1 {
    ram_enable: bool,
    rom_bank_number: u8,           // 5 bits
    ram_bank_number: u8,           // 2 bits
    banking_mode: MBC1BankingMode, // 1 bit
    rom: Vec<u8>,
    ram: Vec<u8>,
}

impl MBC1 {
    pub fn new(rom: Vec<u8>, ram_size: usize) -> Self {
        MBC1 {
            rom,
            ram: vec![0; ram_size],
            ram_enable: false,
            rom_bank_number: 0,
            ram_bank_number: 0,
            banking_mode: MBC1BankingMode::Simple,
        }
    }
}

impl MBC for MBC1 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => self.rom[address as usize],
            0x4000..=0x7FFF => {
                self.rom[self.rom_bank_number as usize * ROM_BANK_SIZE + address as usize]
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let address = match self.banking_mode {
                        MBC1BankingMode::Simple => address as usize - 0xA000,
                        MBC1BankingMode::Advanced => {
                            self.ram_bank_number as usize * RAM_BANK_SIZE + address as usize
                                - 0xA000
                        }
                    };

                    self.ram[address]
                } else {
                    0xFF
                }
            }
            _ => panic!("Invalid MBC1 Address: {:#06X}", address),
        }
    }

    fn write_byte(&mut self, address: u16, byte: u8) {
        match address {
            0x0000..=0x1FFF => self.ram_enable = (byte & 0x0F) == 0x0A,
            0x2000..=0x3FFF => {
                self.rom_bank_number = byte & 0b11111;
            }
            0x4000..=0x5FFF => {
                self.ram_bank_number = byte & 0x3;
            }
            0x6000..=0x7FFF => {
                self.banking_mode = if byte & 1 == 1 {
                    MBC1BankingMode::Advanced
                } else {
                    MBC1BankingMode::Simple
                };
            }
            0xA000..=0xBFFF => {
                if self.ram_enable {
                    let address = match self.banking_mode {
                        MBC1BankingMode::Simple => address as usize - 0xA000,
                        MBC1BankingMode::Advanced => {
                            self.ram_bank_number as usize * RAM_BANK_SIZE + address as usize
                                - 0xA000
                        }
                    };

                    if address < self.ram.len() {
                        self.ram[address] = byte;
                    }
                }
            }
            _ => panic!("Invalid MBC1 Address: {:#06X}", address),
        }
    }
}
