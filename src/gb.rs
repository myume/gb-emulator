use crate::{
    cartridge::Cartridge,
    cpu::{CPU, Cycles},
    mmu::{InterruptFlag, MMU},
    utils::{is_set, reset_bit},
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

        let mut cycles = 0;
        if self.cpu.halted {
            if self.cpu.get_ime() {
                self.cpu.halted = false;
            } else {
                let pending_interrupt =
                    self.mmu.interrupt_enable & *self.mmu.interrupt_flag.borrow() != 0;
                if pending_interrupt {
                    self.cpu.halted = false;
                } else {
                    cycles = 4;
                }
            }
        }

        cycles += self.handle_interrupts();

        if !self.cpu.halted {
            let opcode = self.mmu.read_byte(self.cpu.registers.pc());
            if self.cpu.halt_bug {
                self.cpu.registers.set_pc(self.cpu.registers.pc() - 1);
                self.cpu.halt_bug = false;
            }
            cycles += self.execute_opcode(opcode);

            if self.cpu.ei && opcode != 0xFB {
                self.cpu.set_ime(true);
                self.cpu.ei = false;
            }
        }

        self.mmu.tick(cycles);

        cycles
    }

    pub fn pixel_data(&self) -> &[u8] {
        self.mmu.ppu.pixel_data()
    }

    fn stack_push_word(&mut self, value: u16) -> Cycles {
        let low = value & 0x00FF;
        let high = value >> 8;
        self.stack_push_byte(high as u8) + self.stack_push_byte(low as u8)
    }

    fn stack_push_byte(&mut self, value: u8) -> Cycles {
        self.cpu
            .registers
            .set_sp(self.cpu.registers.sp().wrapping_sub(1));
        self.mmu.write_byte(self.cpu.registers.sp(), value);

        4 // 4 T cycles to push a byte
    }

    fn handle_interrupts(&mut self) -> Cycles {
        if !self.cpu.get_ime() {
            return 0;
        }

        for interrupt in InterruptFlag::iter() {
            let flag = *self.mmu.interrupt_flag.borrow();
            if is_set(flag, interrupt as u8) && is_set(self.mmu.interrupt_enable, interrupt as u8) {
                let mut cycles = 0;
                self.cpu.set_ime(false);
                *self.mmu.interrupt_flag.borrow_mut() = reset_bit(flag, interrupt as u8);
                cycles += 8; // 2 NOP cycles

                cycles += self.stack_push_word(self.cpu.registers.pc());

                let handler_address = match interrupt {
                    InterruptFlag::Joypad => 0x60,
                    InterruptFlag::Serial => 0x58,
                    InterruptFlag::Timer => 0x50,
                    InterruptFlag::LCD => 0x48,
                    InterruptFlag::VBlank => 0x40,
                };
                self.cpu.registers.set_pc(handler_address);
                cycles += 4; // 1 M cycle to set PC;

                return cycles;
            }
        }

        0
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
