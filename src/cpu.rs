use crate::utils::compose_bytes;
use paste::paste;

pub type Cycles = usize;

pub struct CPU {
    pub registers: Registers,
    ime: bool, // ime flag
    pub halted: bool,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
            ime: false,
            halted: false,
        }
    }

    pub fn set_ime(&mut self, set: bool) {
        self.ime = set;
    }
    pub fn get_ime(&self) -> bool {
        self.ime
    }

    pub fn alu_add(&mut self, rhs: u8, add_carry: bool) {
        let lhs = self.registers.a();
        let c = if add_carry {
            self.registers.get_flag(CpuFlags::C) as u8
        } else {
            0
        };
        let sum = lhs.wrapping_add(rhs).wrapping_add(c);
        self.registers.set_a(sum);

        self.registers.set_flag(CpuFlags::Z, sum == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers
            .set_flag(CpuFlags::H, (lhs & 0x0F) + (rhs & 0x0F) + (c & 0x0F) > 0x0F);
        self.registers.set_flag(
            CpuFlags::C,
            (lhs as u16 & 0xFF) + (rhs as u16 & 0xFF) + (c as u16 & 0xFF) > 0xFF,
        );
    }

    /// Perform a sub operation. Return sum and set flags.
    pub fn alu_sub_flags(&mut self, rhs: u8, sub_carry: bool) -> u8 {
        let lhs = self.registers.a();
        let c = if sub_carry {
            self.registers.get_flag(CpuFlags::C) as u8
        } else {
            0
        };
        let diff = lhs.wrapping_sub(rhs).wrapping_sub(c);

        self.registers.set_flag(CpuFlags::Z, diff == 0);
        self.registers.set_flag(CpuFlags::N, true);
        self.registers
            .set_flag(CpuFlags::H, (lhs & 0x0F) < (rhs & 0x0F) + (c & 0x0F));
        self.registers.set_flag(
            CpuFlags::C,
            (lhs as u16 & 0xFF) < (rhs as u16 & 0xFF) + (c as u16 & 0xFF),
        );
        diff
    }

    pub fn alu_sub(&mut self, rhs: u8, sub_carry: bool) {
        let diff = self.alu_sub_flags(rhs, sub_carry);
        self.registers.set_a(diff);
    }

    pub fn alu_and(&mut self, rhs: u8) {
        let lhs = self.registers.a();
        let result = lhs & rhs;
        self.registers.set_a(result);

        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, true);
        self.registers.set_flag(CpuFlags::C, false);
    }

    pub fn alu_or(&mut self, rhs: u8) {
        let lhs = self.registers.a();
        let result = lhs | rhs;
        self.registers.set_a(result);

        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, false);
    }

    pub fn alu_xor(&mut self, rhs: u8) {
        let lhs = self.registers.a();
        let result = lhs ^ rhs;
        self.registers.set_a(result);

        self.registers.set_flag(CpuFlags::Z, result == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, false);
    }

    pub fn alu_cp(&mut self, rhs: u8) {
        self.alu_sub_flags(rhs, false);
    }

    pub fn alu_rlca(&mut self) {
        let a = self.registers.a();
        self.registers.set_a(a.rotate_left(1));

        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, a >= 0b1000_0000);
    }

    pub fn alu_rla(&mut self) {
        let a = self.registers.a();

        let c = if self.registers.get_flag(CpuFlags::C) {
            1
        } else {
            0
        };

        self.registers.set_a((a << 1) + c);

        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, a >= 0b1000_0000);
    }

    pub fn alu_rrca(&mut self) {
        let a = self.registers.a();
        self.registers.set_a(a.rotate_right(1));

        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, a & 0b0000_0001 == 1);
    }

    pub fn alu_rra(&mut self) {
        let a = self.registers.a();

        let c = if self.registers.get_flag(CpuFlags::C) {
            0b1000_0000
        } else {
            0
        };

        self.registers.set_a((a >> 1) | c);

        self.registers.set_flag(CpuFlags::Z, false);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, a & 0b0000_0001 == 1);
    }

    pub fn alu_cpl(&mut self) {
        let a = self.registers.a();
        self.registers.set_a(!a);

        self.registers.set_flag(CpuFlags::N, true);
        self.registers.set_flag(CpuFlags::H, true);
    }

    pub fn alu_daa(&mut self) {
        let a = self.registers.a();
        let mut adjustment = 0;

        let n = self.registers.get_flag(CpuFlags::N);
        let h = self.registers.get_flag(CpuFlags::H);
        let c = self.registers.get_flag(CpuFlags::C);

        if n {
            if h {
                adjustment += 0x06;
            }
            if c {
                adjustment += 0x60;
            }
            self.registers
                .set_a(self.registers.a().wrapping_sub(adjustment));
        } else {
            if h || a & 0x0F > 0x09 {
                adjustment += 0x06;
            }

            if c || a > 0x99 {
                adjustment += 0x60;
            }
            self.registers.set_a(a.wrapping_add(adjustment));
        }

        self.registers
            .set_flag(CpuFlags::Z, self.registers.a() == 0);
        self.registers.set_flag(CpuFlags::H, false);
        self.registers.set_flag(CpuFlags::C, adjustment >= 0x60);
    }
}

pub struct Registers {
    a: u8,
    f: u8, // flags: z (zero), n (sub BCD), h (half carry BCD), c (carry)

    b: u8,
    c: u8,

    d: u8,
    e: u8,

    h: u8,
    l: u8,

    sp: u16,
    pc: u16,
}

#[derive(Copy, Clone)]
pub enum CpuFlags {
    Z = 0b10000000,
    N = 0b01000000,
    H = 0b00100000,
    C = 0b00010000,
}

#[derive(Debug)]
pub enum CpuFlagError {
    ParseError,
}

impl CpuFlags {
    pub fn from_str(s: &str) -> Result<Self, CpuFlagError> {
        match s {
            "Z" => Ok(Self::Z),
            "N" => Ok(Self::N),
            "H" => Ok(Self::H),
            "C" => Ok(Self::C),
            _ => Err(CpuFlagError::ParseError),
        }
    }
}

impl Registers {
    pub fn new() -> Self {
        Registers {
            a: 0,
            f: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            h: 0,
            l: 0,
            sp: 0,
            pc: 0,
        }
    }

    pub fn get_flag(&self, flag: CpuFlags) -> bool {
        let mask = flag as u8;
        self.f & mask > 0
    }

    pub fn set_flag(&mut self, flag: CpuFlags, set: bool) {
        let mask = flag as u8;
        match set {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
    }
}

macro_rules! create_base_registers {
    ($($r:ident: $t:ty),*) => {
        impl Registers {
                paste! {
                    $(
                        pub fn [<$r>](&self) -> $t {
                            self.$r
                        }

                        pub fn [<set_ $r>](&mut self, value: $t) {
                            self.$r = value;
                        }
                    )*
                }
        }
    };
}

macro_rules! create_combined_registers {
    ($(($r1:ident, $r2:ident)),*) => {
        impl Registers {
                paste! {
                    $(
                        pub fn [<$r1 $r2>](&self) -> u16 {
                            compose_bytes(self.$r1, self.$r2)
                        }

                        pub fn [<set_ $r1 $r2>](&mut self, value: u16) {
                            let [lower, upper] = value.to_le_bytes();
                            self.$r1 = upper;
                            self.$r2 = lower;
                        }
                    )*
                }
        }
    };
}

create_base_registers!(
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: u16
);

create_combined_registers!((a, f), (b, c), (d, e), (h, l));

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_combined_registers() {
        let mut regs = Registers::new();

        assert_eq!(regs.b(), 0);
        assert_eq!(regs.c(), 0);

        assert_eq!(regs.bc(), 0);

        regs.set_bc(0xabcd);

        assert_eq!(regs.bc(), 0xabcd);
        assert_eq!(regs.b(), 0xab);
        assert_eq!(regs.c(), 0xcd);

        regs.set_b(0x12);
        regs.set_c(0x34);

        assert_eq!(regs.bc(), 0x1234);
        assert_eq!(regs.b(), 0x12);
        assert_eq!(regs.c(), 0x34);
    }

    #[test]
    fn test_alu_add() {
        let mut cpu = CPU::new();
        cpu.alu_add(10, false);
        assert_eq!(cpu.registers.a(), 10);
    }

    #[test]
    fn test_alu_add_carry() {
        let mut cpu = CPU::new();
        cpu.registers.set_flag(CpuFlags::C, true);
        cpu.alu_add(10, true);

        assert_eq!(cpu.registers.a(), 11);
    }

    #[test]
    fn test_alu_add_flags() {
        let mut cpu = CPU::new();
        cpu.registers.set_a(1);
        cpu.alu_add(0xFF, false);
        assert_eq!(cpu.registers.a(), 0);
        assert!(cpu.registers.get_flag(CpuFlags::Z));
        assert!(!cpu.registers.get_flag(CpuFlags::N));
        assert!(cpu.registers.get_flag(CpuFlags::H));
        assert!(cpu.registers.get_flag(CpuFlags::C));
    }

    #[test]
    fn test_alu_sub() {
        let mut cpu = CPU::new();
        cpu.alu_sub(1, false);
        assert_eq!(cpu.registers.a(), 0xFF);
        assert!(!cpu.registers.get_flag(CpuFlags::Z));
        assert!(cpu.registers.get_flag(CpuFlags::N));
        assert!(cpu.registers.get_flag(CpuFlags::H));
        assert!(cpu.registers.get_flag(CpuFlags::C));
    }

    #[test]
    fn test_rlca() {
        let mut cpu = CPU::new();

        assert!(!cpu.registers.get_flag(CpuFlags::C));

        cpu.registers.set_a(0b1000_0000);
        cpu.alu_rlca();

        assert!(cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b0000_0001);
    }

    #[test]
    fn test_rla() {
        let mut cpu = CPU::new();

        assert!(!cpu.registers.get_flag(CpuFlags::C));

        cpu.registers.set_a(0b1000_0000);
        cpu.alu_rla();

        assert!(cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b0000_0000);

        cpu.alu_rla();
        assert!(!cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b0000_0001);
    }

    #[test]
    fn test_rrca() {
        let mut cpu = CPU::new();

        cpu.registers.set_a(0b0000_0001);
        cpu.alu_rrca();

        assert!(cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b1000_0000);
    }

    #[test]
    fn test_rra() {
        let mut cpu = CPU::new();

        cpu.registers.set_a(0b0000_0001);
        cpu.alu_rra();

        assert!(cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b0000_0000);

        cpu.alu_rra();
        assert!(!cpu.registers.get_flag(CpuFlags::C));
        assert_eq!(cpu.registers.a(), 0b1000_0000);
    }

    #[test]
    fn test_cpl() {
        let mut cpu = CPU::new();

        cpu.registers.set_a(0b1010_0101);
        cpu.alu_cpl();

        assert!(cpu.registers.get_flag(CpuFlags::N));
        assert!(cpu.registers.get_flag(CpuFlags::H));
        assert_eq!(cpu.registers.a(), 0b0101_1010);
    }
}
