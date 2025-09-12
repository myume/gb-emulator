use crate::{cpu::Cycles, gb::GameBoy};

impl GameBoy {
    pub fn execute_opcode(&mut self, opcode: u8) -> Cycles {
        match opcode {
            // NOP
            0x00 => 4,

            // LD BC, n16
            0x01 => {
                let immediate = self.mmu.read_word(self.cpu.registers.pc + 1);
                self.cpu.registers.set_bc(immediate);
                self.cpu.registers.pc += 3; // increase the pc by the length of the instruction
                12
            }

            // LD [BC], A
            0x02 => 8,
            _ => unreachable!("Encountered illegal opcode"),
        }
    }
}
