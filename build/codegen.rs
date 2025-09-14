use std::{collections::BTreeMap, fs::File, io::BufReader, path::Path};

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::{Deserialize, Serialize};
use syn::{
    LitInt,
    token::{Minus, Plus},
};

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
        syn::BinOp::Add(Plus::default())
    } else {
        syn::BinOp::Sub(Minus::default())
    };
    quote! {
        self.cpu.registers.#setter(
            self.cpu.registers.#getter() #op 1
        );
    }
}

fn handle_load_instruction(entry: &OpcodeEntry) -> TokenStream {
    assert_eq!(entry.mnemonic, "LD");
    let dest = &entry.operands[0];
    let src = &entry.operands[1];

    quote! {}
}
