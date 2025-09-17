use crate::{cpu::CPU, mmu::MMU};

pub struct GameBoy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl GameBoy {
    pub fn new() -> Self {
        GameBoy {
            cpu: CPU::new(),
            mmu: MMU::new(),
        }
    }
}
