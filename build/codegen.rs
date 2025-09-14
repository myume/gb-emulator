use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::LitInt;

#[derive(Serialize, Deserialize)]
struct OpcodeTable {
    // BTrees for ordered keys/opcodes. We love BTrees.
    unprefixed: BTreeMap<String, OpcodeEntry>,
    cbprefixed: BTreeMap<String, OpcodeEntry>,
}

#[derive(Serialize, Deserialize)]
struct Operand {
    name: String,
    immediate: bool,
    increment: Option<bool>,
    bytes: Option<u16>,
}

#[derive(Serialize, Deserialize)]
struct OpcodeEntry {
    mnemonic: String,
    bytes: u16,
    cycles: Vec<usize>,
    immediate: bool,
    flags: Flags,
    operands: Vec<Operand>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct Flags {
    z: String,
    n: String,
    h: String,
    c: String,
}

pub fn generate_opcode_instructions(opcode_table_path: &Path) -> String {
    let opcode_json = File::open(&opcode_table_path).expect("Failed to open opcodes.json");
    let reader = BufReader::new(opcode_json);
    let opcode_table: OpcodeTable =
        serde_json::from_reader(reader).expect("Invalid Opcode JSON Structure");

    let match_arms = opcode_table.unprefixed.iter().map(|(opcode, entry)| {
        let full_instruction = format!(
            " {} {}",
            entry.mnemonic.clone(),
            entry
                .operands
                .iter()
                .map(|operand| operand.name.as_str())
                .collect::<Vec<&str>>()
                .join(",")
        );
        let hex_literal = LitInt::new(opcode, Span::call_site());
        let cycles = entry.cycles[0];
        let bytes = entry.bytes;

        let body = generate_opcode_body(&entry);

        quote! {
            #[doc = #full_instruction]
            #hex_literal => {
                #body
                self.cpu.registers.set_pc(self.cpu.registers.pc() + #bytes);
                #cycles
            }
        }
    });

    let instructions = quote! {
        #[allow(unused_doc_comments)]
        impl GameBoy {
            pub fn execute_opcode(&mut self, opcode: u8) -> Cycles {
                match opcode {
                    #(#match_arms,)*
                    // _ => unreachable!("Encountered illegal opcode"),
                }
            }
        }
    };

    let ast = syn::parse2(instructions).unwrap();
    prettyplease::unparse(&ast)
}

fn generate_opcode_body(entry: &OpcodeEntry) -> TokenStream {
    match entry.mnemonic.as_str() {
        "NOP" => quote! {},
        "LD" => handle_load_instruction(entry),
        "INC" => handle_inc_dec_instruction(entry),
        "DEC" => handle_inc_dec_instruction(entry),
        _ => quote! {
            // todo!("Unhandled Instruction");
        },
    }
}

fn handle_inc_dec_instruction(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "INC" || entry.mnemonic == "DEC");

    let reg = entry.operands[0].name.to_lowercase();
    let setter = format_ident!("set_{}", reg);
    let getter = format_ident!("{}", reg);
    let op = if entry.mnemonic == "INC" {
        format_ident!("wrapping_add")
    } else {
        format_ident!("wrapping_sub")
    };
    quote! {
        self.cpu.registers.#setter(
            self.cpu.registers.#getter().#op(1)
        );
    }
}

fn is_register(name: &str) -> bool {
    let name = name.to_lowercase();
    name == "a"
        || name == "f"
        || name == "af"
        || name == "b"
        || name == "c"
        || name == "bc"
        || name == "d"
        || name == "e"
        || name == "de"
        || name == "h"
        || name == "l"
        || name == "hl"
        || name == "sp"
        || name == "pc"
}

fn handle_load_instruction(entry: &OpcodeEntry) -> TokenStream {
    assert_eq!(entry.mnemonic, "LD");

    let dest = &entry.operands[0];
    let src = &entry.operands[1];
    let dest_is_register = is_register(&dest.name);
    let src_is_register = is_register(&src.name);

    let loaded_val = format_ident!("val");
    let load = if src.immediate && src_is_register {
        // load from immediate register
        let reg = format_ident!("{}", src.name.to_lowercase());
        let mut load_val = quote! {
            let #loaded_val = self.cpu.registers.#reg();
        };
        if entry.operands.len() == 3 && src.name.to_lowercase() == "sp" {
            let offset = &entry.operands[2];
            assert_eq!(offset.name, "e8");
            load_val = quote! {
                #load_val
                let offset = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1)) as i8 as i16 as u16;
                let #loaded_val = #loaded_val.wrapping_add(offset);
                self.cpu.registers.set_flag(CpuFlags::Z, false);
                self.cpu.registers.set_flag(CpuFlags::N, false);
                let h_flag = (self.cpu.registers.sp() & 0x0F) + (offset as u16 & 0x0F) > 0x0F;
                let c_flag = (self.cpu.registers.sp() & 0xFF) + (offset as u16 & 0xFF) > 0xFF;
                self.cpu.registers.set_flag(CpuFlags::H, h_flag);
                self.cpu.registers.set_flag(CpuFlags::C, c_flag);

            }
        }

        load_val
    } else if src.immediate && !src_is_register {
        // load from immediate value
        let read_op = if src.bytes.unwrap_or(1) == 1 {
            format_ident!("read_byte")
        } else {
            format_ident!("read_word")
        };
        quote! {
            let #loaded_val = self.mmu.#read_op(self.cpu.registers.pc().wrapping_add(1));
        }
    } else if !src.immediate && src_is_register {
        // load from address with register
        let reg = format_ident!("{}", src.name.to_lowercase());
        quote! {
            let #loaded_val = self.mmu.read_byte(self.cpu.registers.#reg());
        }
    } else {
        // load from immediate address
        quote! {
            let address = self.mmu.read_word(self.cpu.registers.pc().wrapping_add(1));
            let #loaded_val = self.mmu.read_byte(address);
        }
    };

    let store = if dest_is_register && dest.immediate {
        // store into immediate register
        let setter = format_ident!("set_{}", dest.name.to_lowercase());
        quote! {
            self.cpu.registers.#setter(
                #loaded_val
            );
        }
    } else if dest_is_register && !dest.immediate {
        // store into register address
        let reg = format_ident!("{}", dest.name.to_lowercase());
        quote! {
            let address = self.cpu.registers.#reg();
            self.mmu.write_byte(address, #loaded_val);

        }
    } else {
        // store into immediate address
        let write_op = if src.name.to_lowercase() == "sp" {
            format_ident!("write_word")
        } else {
            format_ident!("write_byte")
        };

        quote! {
            let address = self.mmu.read_word(self.cpu.registers.pc() + 1);
            self.mmu.#write_op(address, #loaded_val);
        }
    };

    let increment = {
        if Some(true) == dest.increment && dest.name.to_lowercase() == "hl" {
            let setter = format_ident!("set_{}", dest.name.to_lowercase());
            let getter = format_ident!("{}", dest.name.to_lowercase());
            quote! {
                self.cpu.registers.#setter(
                    self.cpu.registers.#getter().wrapping_add(1)
                );
            }
        } else if Some(true) == src.increment && src.name.to_lowercase() == "hl" {
            let setter = format_ident!("set_{}", src.name.to_lowercase());
            let getter = format_ident!("{}", src.name.to_lowercase());
            quote! {
                self.cpu.registers.#setter(
                    self.cpu.registers.#getter().wrapping_add(1)
                );
            }
        } else {
            quote! {}
        }
    };

    quote! {
        #load
        #store
        #increment
    }
}
