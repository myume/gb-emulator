use clap::Parser;

use crate::{cartridge::Cartridge, cpu::CPU, mmu::MMU};

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
pub struct GameBoyConfig {
    pub cartridge_path: String,

    #[arg(short, long)]
    pub print_serial: bool,
}

pub struct GameBoy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl GameBoy {
    pub fn new(cartridge: Cartridge, config: GameBoyConfig) -> Self {
        GameBoy {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge, config),
        }
    }

    pub fn tick(&mut self) {
        let opcode = self.mmu.read_byte(self.cpu.registers.pc());
        let cycles = self.execute_opcode(opcode);
        self.mmu.tick(cycles);

        if self.cpu.ei && opcode != 0xFB {
            self.cpu.set_ime(true);
            self.cpu.ei = false;
        }
    }
}
