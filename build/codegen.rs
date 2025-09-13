use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use proc_macro2::Span;
use quote::quote;
use serde::{Deserialize, Serialize};
use syn::LitInt;

#[derive(Serialize, Deserialize)]
pub struct OpcodeTable {
    // BTrees for ordered keys/opcodes. We love BTrees.
    unprefixed: BTreeMap<String, OpcodeEntry>,
    cbprefixed: BTreeMap<String, OpcodeEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct OpcodeEntry {
    mnemonic: String,
    bytes: u8,
    cycles: Vec<usize>,
    immediate: bool,
    flags: Flags,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Flags {
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
        let op = entry.mnemonic.clone();
        let hex_literal = LitInt::new(opcode, Span::call_site());
        let cycles = entry.cycles[0];
        let bytes = entry.bytes;
        quote! {
            #hex_literal => {
                // #op;
                self.cpu.registers.pc += #bytes;
                #cycles
            }
        }
    });

    let instructions = quote! {
        impl GameBoy {
            pub fn execute_opcode(&mut self, opcode: u8) {
                match opcode {
                    #(#match_arms,)*
                    _ => unreachable!("Encountered illegal opcode"),
                }
            }
        }
    };

    let ast = syn::parse2(instructions).unwrap();
    prettyplease::unparse(&ast)
}
