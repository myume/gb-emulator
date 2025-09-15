use paste::paste;

pub type Cycles = usize;

pub struct CPU {
    pub registers: Registers,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registers: Registers::new(),
        }
    }

    pub fn alu_add(&mut self, b: u8, add_carry: bool) {
        let a = self.registers.a();
        let c = if add_carry {
            self.registers.get_flag(CpuFlags::C) as u8
        } else {
            0
        };
        let sum = a.wrapping_add(b).wrapping_add(c);
        self.registers.set_a(sum);

        self.registers.set_flag(CpuFlags::Z, sum == 0);
        self.registers.set_flag(CpuFlags::N, false);
        self.registers
            .set_flag(CpuFlags::H, (a & 0x0F) + (b & 0x0F) > 0x0F);
        self.registers.set_flag(
            CpuFlags::C,
            (a as u16 & 0xFF) + (b as u16 & 0xFF) + (c as u16 & 0xFF) > 0xFF,
        );
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
        self.f & !mask > 0
    }

    pub fn set_flag(&mut self, flag: CpuFlags, set: bool) {
        let mask = flag as u8;
        match set {
            true => self.f |= mask,
            false => self.f &= !mask,
        }
    }

    fn combine_registers(r1: u8, r2: u8) -> u16 {
        ((r1 as u16) << 8) | r2 as u16
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
                            Registers::combine_registers(self.$r1, self.$r2)
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
}
