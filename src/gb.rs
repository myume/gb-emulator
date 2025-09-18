use crate::{cartridge::Cartridge, cpu::CPU, mmu::MMU};

pub struct GameBoy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl GameBoy {
    pub fn new(cartridge: Cartridge) -> Self {
        GameBoy {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge),
        }
    }
}
