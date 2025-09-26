use crate::{
    cartridge::Cartridge,
    cpu::{CPU, Cycles},
    mmu::MMU,
};

pub struct GameBoy {
    pub cpu: CPU,
    pub mmu: MMU,
}

impl GameBoy {
    pub fn new(cartridge: Cartridge, print_serial: bool) -> Self {
        GameBoy {
            cpu: CPU::new(),
            mmu: MMU::new(cartridge, print_serial),
        }
    }

    pub fn tick(&mut self) -> Cycles {
        #[cfg(feature = "gb_doctor")]
        self.print_registers();

        let opcode = self.mmu.read_byte(self.cpu.registers.pc());
        let cycles = self.execute_opcode(opcode);
        self.mmu.tick(cycles);

        if self.cpu.ei && opcode != 0xFB {
            self.cpu.set_ime(true);
            self.cpu.ei = false;
        }

        cycles
    }

    pub fn pixel_data(&self) -> &[u8] {
        self.mmu.ppu.pixel_data()
    }

    #[cfg(feature = "gb_doctor")]
    fn print_registers(&self) {
        println!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.cpu.registers.a(),
            self.cpu.registers.f(),
            self.cpu.registers.b(),
            self.cpu.registers.c(),
            self.cpu.registers.d(),
            self.cpu.registers.e(),
            self.cpu.registers.h(),
            self.cpu.registers.l(),
            self.cpu.registers.sp(),
            self.cpu.registers.pc(),
            self.mmu.read_byte(self.cpu.registers.pc()),
            self.mmu.read_byte(self.cpu.registers.pc() + 1),
            self.mmu.read_byte(self.cpu.registers.pc() + 2),
            self.mmu.read_byte(self.cpu.registers.pc() + 3),
        );
    }
}
