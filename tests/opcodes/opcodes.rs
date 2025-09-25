use paste::paste;
use std::{
    fs::{File, read_dir},
    io::BufReader,
    process::ExitCode,
};

use gb_emulator::{
    cartridge::{Cartridge, NoMBC},
    cpu::CPU,
    gb::{GameBoy, GameBoyConfig},
    mmu::MMU,
};
use libtest_mimic::{Arguments, Failed, Trial};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct TestCase {
    name: String,
    initial: GBState,
    #[serde(rename = "final")]
    expected: GBState,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GBState {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    ime: u8,
    ei: Option<u8>,
    ie: Option<u8>,
    ram: Vec<(u16, u8)>,
}

fn main() -> ExitCode {
    let args = Arguments::from_args();

    let json_test_dir = read_dir("./tests/opcodes/sm83/v1").expect("Missing sm83 tests");

    // TODO: figure out how to load this faster.
    // Bottleneck is loading the test cases and not actually running the tests lol.

    let tests = json_test_dir
        .map(|file| {
            let json_test_file = File::open(file.unwrap().path()).unwrap();
            let reader = BufReader::new(json_test_file);
            let json_tests: Vec<TestCase> =
                serde_json::from_reader(reader).expect("Invalid Test Structure");

            // vec![json_tests[0].clone()]
            json_tests
        })
        .map(|json_tests| {
            json_tests
                .into_iter()
                .map(|json_test| Trial::test(json_test.name.clone(), || test_opcode(json_test)))
        })
        .flatten()
        .collect();

    libtest_mimic::run(&args, tests).exit_code()
}

fn initialize_test(initial: &GBState) -> GameBoy {
    let mut cpu = CPU::new();
    cpu.registers.set_pc(initial.pc);
    cpu.registers.set_sp(initial.sp);
    cpu.registers.set_a(initial.a);
    cpu.registers.set_b(initial.b);
    cpu.registers.set_c(initial.c);
    cpu.registers.set_d(initial.d);
    cpu.registers.set_e(initial.e);
    cpu.registers.set_f(initial.f);
    cpu.registers.set_h(initial.h);
    cpu.registers.set_l(initial.l);
    cpu.set_ime(initial.ime > 0);

    let cart = Cartridge {
        title: "test".into(),
        mbc: Box::new(NoMBC::new()),
    };
    let mut mmu = MMU::new(cart, GameBoyConfig::default());

    initial
        .ram
        .iter()
        .for_each(|(address, value)| mmu.write_byte(*address, *value));

    GameBoy { cpu, mmu }
}

fn validate_test(expected: &GBState, gb: &GameBoy) -> Result<(), Failed> {
    macro_rules! validate_register {
        ($($r:ident),*) => {
            paste! {
                $(
                    if gb.cpu.registers.$r() != expected.$r {
                        return Err(Failed::from(format!(
                            "Expected register {} to be {:#04X} found {:#04X}",
                            stringify!($r),
                            expected.$r,
                            gb.cpu.registers.$r()
                        )));
                    }
                )*
            }
        };
    }

    validate_register!(pc, sp, a, b, c, d, e, f, h, l);

    if gb.cpu.get_ime() as u8 != expected.ime {
        return Err(Failed::from(format!(
            "Expected ime to be {:#04X} found {:#04X}",
            expected.ime,
            gb.cpu.get_ime() as u8
        )));
    }

    if gb.cpu.ei as u8 != expected.ei.unwrap_or(0x00) {
        return Err(Failed::from(format!(
            "Expected ei to be {:#04X} found {:#04X}",
            expected.ei.unwrap_or(0x00),
            gb.cpu.ei as u8
        )));
    }

    for (address, value) in &expected.ram {
        let actual = gb.mmu.read_byte(*address);
        if actual != *value {
            return Err(Failed::from(format!(
                "Expected ram at address {:#06X} to be {:#04X} found {:#04X}",
                address, value, actual
            )));
        }
    }

    Ok(())
}

fn test_opcode(test_case: TestCase) -> Result<(), Failed> {
    let mut gb = initialize_test(&test_case.initial);

    let opcode = u8::from_str_radix(test_case.name.split(" ").collect::<Vec<&str>>()[0], 16)
        .expect(&format!("Invalid opcode in {}", test_case.name));
    gb.execute_opcode(opcode);

    validate_test(&test_case.expected, &gb)
}
