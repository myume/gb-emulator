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
    decrement: Option<bool>,
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
                .map(|operand| if operand.immediate {
                    operand.name.clone()
                } else {
                    format!("[{}]", operand.name)
                })
                .collect::<Vec<String>>()
                .join(",")
        );
        let hex_literal = LitInt::new(opcode, Span::call_site());
        let epilogue =
            // These instructions alter the PC and can take multiple possible cycles
            // so handle them individually
            if ["JP", "JR", "CALL", "RET", "RETI", "RST"].contains(&entry.mnemonic.as_str()) || entry.mnemonic == "PREFIX" || entry.mnemonic.starts_with("ILLEGAL") {
                quote! {}
            } else {
                let cycles = entry.cycles[0];
                let mut bytes = entry.bytes;

                if entry.mnemonic == "STOP" {
                    bytes = 1;
                }

                quote! {
                    self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
                    #cycles
                }
            };

        let body = generate_opcode_body(&entry, opcode);

        quote! {
            #[doc = #full_instruction]
            #hex_literal => {
                #body
                #epilogue
            }
        }
    });

    let cb_match_arms = opcode_table.cbprefixed.iter().map(|(opcode, entry)| {
        let full_instruction = format!(
            " {} {}",
            entry.mnemonic.clone(),
            entry
                .operands
                .iter()
                .map(|operand| if operand.immediate {
                    operand.name.clone()
                } else {
                    format!("[{}]", operand.name)
                })
                .collect::<Vec<String>>()
                .join(",")
        );
        let hex_literal = LitInt::new(opcode, Span::call_site());
        let cycles = entry.cycles[0];
        let bytes = entry.bytes - 1;

        let body = generate_cb_body(&entry);

        let epilogue = quote! {
            self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
            #cycles
        };

        quote! {
            #[doc = #full_instruction]
            #hex_literal => {
                #body
                #epilogue
            }
        }
    });

    let instructions = quote! {
        #[allow(unused_doc_comments,unreachable_code)]
        impl GameBoy {
            pub fn execute_opcode(&mut self, opcode: u8) -> Cycles {
                match opcode {
                    #(#match_arms,)*
                }
            }

            pub fn execute_cb_opcode(&mut self, opcode: u8) -> Cycles {
                match opcode {
                    #(#cb_match_arms,)*
                }
            }
        }
    };

    let ast = syn::parse2(instructions).unwrap();
    prettyplease::unparse(&ast)
}

fn generate_opcode_body(entry: &OpcodeEntry, opcode: &str) -> TokenStream {
    match entry.mnemonic.as_str() {
        "NOP" => quote! {},
        "STOP" => quote! {},
        "HALT" => handle_halt(entry),
        "LD" => handle_load_instruction(entry),
        "INC" => handle_inc_dec_instruction(entry),
        "DEC" => handle_inc_dec_instruction(entry),
        "ADD" => handle_add(entry),
        "ADC" => handle_add(entry),
        "SUB" => handle_sub(entry),
        "SBC" => handle_sub(entry),
        "AND" => handle_boolean_op(entry),
        "OR" => handle_boolean_op(entry),
        "XOR" => handle_boolean_op(entry),
        "CP" => handle_boolean_op(entry),
        "RLCA" => handle_rlca(entry),
        "RLA" => handle_rla(entry),
        "RRCA" => handle_rrca(entry),
        "RRA" => handle_rra(entry),
        "JP" => handle_jump(entry),
        "JR" => handle_jump(entry),
        "CPL" => handle_cpl(entry),
        "SCF" => handle_scf(entry),
        "CCF" => handle_ccf(entry),
        "DAA" => handle_daa(entry),
        "RET" => handle_ret(entry),
        "RETI" => handle_ret(entry),
        "POP" => handle_pop(entry),
        "PUSH" => handle_push(entry),
        "CALL" => handle_call(entry),
        "RST" => handle_rst(entry),
        "LDH" => handle_ldh(entry),
        "EI" => handle_ei(entry),
        "DI" => handle_di(entry),
        "PREFIX" => handle_cb(entry),
        _ => {
            let err_message = format!("Unhandled instruction {}", opcode);
            quote! {
                panic!(#err_message);
            }
        }
    }
}

fn handle_inc_dec_instruction(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "INC" || entry.mnemonic == "DEC");

    let operand = &entry.operands[0];
    let reg = operand.name.to_lowercase();
    let setter = format_ident!("set_{}", reg);
    let getter = format_ident!("{}", reg);
    let op = if entry.mnemonic == "INC" {
        format_ident!("wrapping_add")
    } else {
        format_ident!("wrapping_sub")
    };

    let load = if operand.immediate {
        quote! {
            let val = self.cpu.registers.#getter();
            let result = val.#op(1);
        }
    } else {
        quote! {
            let val = self.mmu.read_byte(self.cpu.registers.#getter());
            let result = val.#op(1);
        }
    };
    let store = if operand.immediate {
        quote! {
            self.cpu.registers.#setter(result);
        }
    } else {
        quote! {
            self.mmu.write_byte(self.cpu.registers.#getter(), result);
        }
    };

    let flags = if entry.mnemonic == "INC"
        && (reg.len() != 2 || (reg == "hl" && !operand.immediate))
    {
        quote! {
            self.cpu.registers.set_flag(CpuFlags::Z, result == 0);
            self.cpu.registers.set_flag(CpuFlags::N, false);
            self.cpu.registers.set_flag(CpuFlags::H, (val & 0x0F) + 1 > 0x0F);
        }
    } else if entry.mnemonic == "DEC" && !(is_register(&reg) && operand.immediate && reg.len() == 2)
    {
        quote! {
            self.cpu.registers.set_flag(CpuFlags::Z, result == 0);
            self.cpu.registers.set_flag(CpuFlags::N, true);
            self.cpu.registers.set_flag(CpuFlags::H, (val & 0x0F) == 0);
        }
    } else {
        quote! {}
    };

    quote! {
        #load
        #store
        #flags
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
            let address = self.mmu.read_word(self.cpu.registers.pc().wrapping_add(1));
            self.mmu.#write_op(address, #loaded_val);
        }
    };

    let crement = {
        if (Some(true) == dest.increment || Some(true) == dest.decrement)
            && dest.name.to_lowercase() == "hl"
        {
            let setter = format_ident!("set_{}", dest.name.to_lowercase());
            let getter = format_ident!("{}", dest.name.to_lowercase());
            let op = if Some(true) == dest.increment {
                format_ident!("wrapping_add")
            } else {
                format_ident!("wrapping_sub")
            };
            quote! {
                self.cpu.registers.#setter(
                    self.cpu.registers.#getter().#op(1)
                );
            }
        } else if (Some(true) == src.increment || Some(true) == src.decrement)
            && src.name.to_lowercase() == "hl"
        {
            let setter = format_ident!("set_{}", src.name.to_lowercase());
            let getter = format_ident!("{}", src.name.to_lowercase());
            let op = if Some(true) == src.increment {
                format_ident!("wrapping_add")
            } else {
                format_ident!("wrapping_sub")
            };
            quote! {
                self.cpu.registers.#setter(
                    self.cpu.registers.#getter().#op(1)
                );
            }
        } else {
            quote! {}
        }
    };

    quote! {
        #load
        #store
        #crement
    }
}

fn handle_add(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "ADD" || entry.mnemonic == "ADC");
    assert_eq!(entry.operands.len(), 2);

    let carry = entry.mnemonic == "ADC";
    let rhs = &entry.operands[1];
    let lhs = &entry.operands[0];

    assert!(is_register(&lhs.name));

    match lhs.name.to_lowercase().as_str() {
        "a" => {
            if is_register(&rhs.name) && rhs.immediate {
                let reg = format_ident!("{}", rhs.name.to_lowercase());
                quote! {
                    self.cpu.alu_add(self.cpu.registers.#reg(), #carry);
                }
            } else if is_register(&rhs.name) && !rhs.immediate {
                let reg = format_ident!("{}", rhs.name.to_lowercase());
                quote! {
                    let b = self.mmu.read_byte(self.cpu.registers.#reg());
                    self.cpu.alu_add(b, #carry);
                }
            } else {
                // must be immediate value. no instruction for immediate addresses
                quote! {
                    let b = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
                    self.cpu.alu_add(b, #carry);
                }
            }
        }

        "hl" => {
            assert!(is_register(&rhs.name) && rhs.immediate);

            let reg = format_ident!("{}", rhs.name.to_lowercase());
            quote! {
                let sum = self.cpu.registers.hl().wrapping_add(self.cpu.registers.#reg());
                self.cpu.registers.set_flag(CpuFlags::N, false);
                self.cpu.registers.set_flag(CpuFlags::H, (self.cpu.registers.hl() as u32 & 0x0FFF) + (self.cpu.registers.#reg() as u32 & 0x0FFF) > 0x0FFF);
                self.cpu.registers.set_flag(CpuFlags::C, (self.cpu.registers.hl() as u32 & 0xFFFF) + (self.cpu.registers.#reg() as u32 & 0xFFFF) > 0xFFFF);
                self.cpu.registers.set_hl(sum);
            }
        }

        "sp" => {
            assert!(!is_register(&rhs.name) && rhs.immediate);

            quote! {
                let b = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1)) as i8 as i16 as u16;
                self.cpu.registers.set_flag(CpuFlags::Z, false);
                self.cpu.registers.set_flag(CpuFlags::N, false);
                self.cpu.registers.set_flag(CpuFlags::H, (self.cpu.registers.sp() & 0x0F) + (b & 0x0F) > 0x0F);
                self.cpu.registers.set_flag(CpuFlags::C, (self.cpu.registers.sp() & 0xFF) + (b & 0xFF) > 0xFF);
                self.cpu.registers.set_sp(self.cpu.registers.sp().wrapping_add(b));
            }
        }

        _ => unreachable!(),
    }
}

fn handle_sub(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "SUB" || entry.mnemonic == "SBC");
    assert_eq!(entry.operands.len(), 2);

    let carry = entry.mnemonic == "SBC";
    let rhs = &entry.operands[1];
    let lhs = &entry.operands[0];

    assert_eq!(lhs.name, "A");

    if is_register(&rhs.name) && rhs.immediate {
        let reg = format_ident!("{}", rhs.name.to_lowercase());
        quote! {
            self.cpu.alu_sub(self.cpu.registers.#reg(), #carry);
        }
    } else if is_register(&rhs.name) && !rhs.immediate {
        let reg = format_ident!("{}", rhs.name.to_lowercase());
        quote! {
            let b = self.mmu.read_byte(self.cpu.registers.#reg());
            self.cpu.alu_sub(b, #carry);
        }
    } else {
        // must be immediate value. no instruction for immediate addresses
        quote! {
            let b = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
            self.cpu.alu_sub(b, #carry);
        }
    }
}

fn handle_boolean_op(entry: &OpcodeEntry) -> TokenStream {
    assert!(
        entry.mnemonic == "AND"
            || entry.mnemonic == "XOR"
            || entry.mnemonic == "OR"
            || entry.mnemonic == "CP"
    );
    assert_eq!(entry.operands.len(), 2);

    let lhs = &entry.operands[0];
    let rhs = &entry.operands[1];

    assert_eq!(lhs.name.to_lowercase(), "a");

    let operation = format_ident!("alu_{}", entry.mnemonic.to_lowercase());

    if is_register(&rhs.name) && rhs.immediate {
        let reg = format_ident!("{}", rhs.name.to_lowercase());
        quote! {
            self.cpu.#operation(self.cpu.registers.#reg());
        }
    } else if is_register(&rhs.name) && !rhs.immediate {
        let reg = format_ident!("{}", rhs.name.to_lowercase());
        quote! {
            let rhs = self.mmu.read_byte(self.cpu.registers.#reg());
            self.cpu.#operation(rhs);
        }
    } else {
        // must be immediate value. no instruction for immediate addresses
        quote! {
            let rhs = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
            self.cpu.#operation(rhs);
        }
    }
}

fn handle_rlca(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RLCA");

    quote! {
        self.cpu.alu_rlca();
    }
}

fn handle_rla(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RLA");

    quote! {
        self.cpu.alu_rla();
    }
}

fn handle_rrca(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RRCA");

    quote! {
        self.cpu.alu_rrca();
    }
}

fn handle_rra(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RRA");

    quote! {
        self.cpu.alu_rra();
    }
}

fn handle_jump(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "JP" || entry.mnemonic == "JR");

    let relative = entry.mnemonic == "JR";

    match entry.operands.len() {
        1 => {
            // jump to address

            let loaded_val = if relative {
                format_ident!("offset")
            } else {
                format_ident!("address")
            };

            let load_op = if relative {
                format_ident!("read_byte")
            } else {
                format_ident!("read_word")
            };

            let reg = if is_register(&entry.operands[0].name) {
                let reg = format_ident!("{}", entry.operands[0].name.to_lowercase());
                quote! {
                    self.cpu.registers.#reg()
                }
            } else {
                quote! {
                    self.cpu.registers.pc().wrapping_add(1)
                }
            };
            let load = if is_register(&entry.operands[0].name) && entry.operands[0].immediate {
                quote! {
                    let #loaded_val = #reg;
                }
            } else {
                quote! {
                    let #loaded_val = self.mmu.#load_op(#reg);
                }
            };

            let set = if relative {
                let bytes = entry.bytes;
                quote! {
                    let address = self.cpu.registers.pc().wrapping_add(#bytes).wrapping_add(#loaded_val as i8 as i16 as u16);
                    self.cpu.registers.set_pc(address);
                }
            } else {
                quote! {
                    self.cpu.registers.set_pc(#loaded_val);
                }
            };

            assert_eq!(entry.cycles.len(), 1);
            let cycles = entry.cycles[0];
            quote! {
                #load
                #set
                #cycles
            }
        }
        2 => {
            // jump to address on condition
            let cond = &entry.operands[0].name;
            let taken_cycles = entry.cycles[0];
            let untaken_cycles = entry.cycles[1];

            let bytes = entry.bytes;
            let load_and_jump = if relative {
                quote! {
                    let offset = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
                    let address = self.cpu.registers.pc().wrapping_add(#bytes).wrapping_add(offset as i8 as i16 as u16);
                    self.cpu.registers.set_pc(address);
                }
            } else {
                quote! {
                    let address = self.mmu.read_word(self.cpu.registers.pc().wrapping_add(1));
                    self.cpu.registers.set_pc(address);
                }
            };

            let condition = conditional(cond);

            quote! {
                if #condition {
                    #load_and_jump
                    #taken_cycles
                } else {
                    self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
                    #untaken_cycles
                }
            }
        }
        _ => unreachable!(),
    }
}

fn handle_cpl(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "CPL");

    quote! {
        self.cpu.alu_cpl();
    }
}

fn handle_scf(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "SCF");
    quote! {
        self.cpu.registers.set_flag(CpuFlags::N, false);
        self.cpu.registers.set_flag(CpuFlags::H, false);
        self.cpu.registers.set_flag(CpuFlags::C, true);
    }
}

fn handle_ccf(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "CCF");
    quote! {
        self.cpu.registers.set_flag(CpuFlags::N, false);
        self.cpu.registers.set_flag(CpuFlags::H, false);
        self.cpu.registers.set_flag(CpuFlags::C, !self.cpu.registers.get_flag(CpuFlags::C));
    }
}

fn handle_daa(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "DAA");
    quote! {
        self.cpu.alu_daa();
    }
}

fn handle_ret(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RET" || entry.mnemonic == "RETI");

    let pop = pop_stack("pc");

    if entry.operands.len() == 1 {
        let cond = &entry.operands[0].name;
        let taken_cycles = entry.cycles[0];
        let untaken_cycles = entry.cycles[1];
        let bytes = entry.bytes;

        let condition = conditional(cond);

        quote! {
            if #condition {
                #pop
                #taken_cycles
            } else {
                self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
                #untaken_cycles
            }
        }
    } else {
        let cycles = entry.cycles[0];
        let enable_interrupts = if entry.mnemonic == "RETI" {
            quote! {
                self.cpu.set_ime(true);
            }
        } else {
            quote! {}
        };
        quote! {
            #pop
            #enable_interrupts
            #cycles
        }
    }
}

fn pop_stack(dest: &str) -> TokenStream {
    let reg = dest.to_lowercase();
    let setter = format_ident!("set_{}", reg);

    quote! {
        let ret = self.mmu.read_word(self.cpu.registers.sp());
        self.cpu.registers.set_sp(self.cpu.registers.sp().wrapping_add(2));
        self.cpu.registers.#setter(ret);
    }
}

fn push_stack(src: &str) -> TokenStream {
    let reg = src.to_lowercase();
    let getter = format_ident!("{}", reg);

    quote! {
        let value = self.cpu.registers.#getter();
        let low = value & 0x00FF;
        let high = value >> 8;
        self.cpu.registers.set_sp(self.cpu.registers.sp().wrapping_sub(1));
        self.mmu.write_byte(self.cpu.registers.sp(), high as u8);
        self.cpu.registers.set_sp(self.cpu.registers.sp().wrapping_sub(1));
        self.mmu.write_byte(self.cpu.registers.sp(), low as u8);
    }
}

fn handle_pop(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "POP");
    pop_stack(&entry.operands[0].name)
}

fn handle_push(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "PUSH");
    push_stack(&entry.operands[0].name)
}

fn conditional(cond: &str) -> TokenStream {
    if cond.len() == 2 {
        assert!(cond.starts_with("N"));
        let (_, cond) = cond.split_at(1);
        quote! {
            !self.cpu.registers.get_flag(CpuFlags::from_str(#cond).expect("invalid condition"))
        }
    } else {
        quote! {
            self.cpu.registers.get_flag(CpuFlags::from_str(#cond).expect("invalid condition"))
        }
    }
}

fn handle_call(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "CALL");

    let bytes = entry.bytes;
    let load = quote! {
        let address = self.mmu.read_word(self.cpu.registers.pc().wrapping_add(1));
        self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
    };

    let base_call = {
        let push_pc = push_stack("pc");
        let cycles = entry.cycles[0];
        quote! {
            #load
            #push_pc
            self.cpu.registers.set_pc(address);
            #cycles
        }
    };

    if entry.operands.len() == 2 {
        let cond = &entry.operands[0].name;
        let untaken_cycles = entry.cycles[1];

        let condition = conditional(cond);

        quote! {
            if #condition {
                #base_call
            } else {
                self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
                #untaken_cycles
            }
        }
    } else {
        base_call
    }
}

fn handle_rst(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RST");
    let push_pc = push_stack("pc");
    let bytes = entry.bytes;
    let cycles = entry.cycles[0];

    let vec = LitInt::new(
        &entry.operands[0].name.replace("$", "0x"),
        Span::call_site(),
    );
    quote! {
        self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
        #push_pc
        self.cpu.registers.set_pc(#vec);
        #cycles
    }
}

fn handle_ldh(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "LDH");
    assert_eq!(entry.operands.len(), 2);

    let dest = &entry.operands[0];
    let src = &entry.operands[1];

    let loaded_val = format_ident!("val");
    let load = if is_register(&src.name) && src.immediate {
        assert!(src.name.to_lowercase() == "a");
        quote! {
            let #loaded_val = self.cpu.registers.a();
        }
    } else if is_register(&src.name) && !src.immediate {
        assert!(src.name.to_lowercase() == "c");

        quote! {
            let address = 0xFF00 + self.cpu.registers.c() as u16;
            let #loaded_val = self.mmu.read_byte(address);
        }
    } else {
        assert!(!is_register(&src.name) && !src.immediate);
        quote! {
            let offset = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
            let address = 0xFF00 + offset as u16;
            let #loaded_val = self.mmu.read_byte(address);
        }
    };

    let store = if is_register(&dest.name) && dest.immediate {
        assert_eq!(dest.name.to_lowercase(), "a");

        quote! {
            self.cpu.registers.set_a(#loaded_val);
        }
    } else if is_register(&dest.name) && !dest.immediate {
        assert_eq!(dest.name.to_lowercase(), "c");

        quote! {
            self.mmu.write_byte(0xFF00 + self.cpu.registers.c() as u16, #loaded_val);
        }
    } else {
        assert!(!is_register(&dest.name) && !dest.immediate);

        quote! {
            let offset = self.mmu.read_byte(self.cpu.registers.pc().wrapping_add(1));
            self.mmu.write_byte(0xFF00 + offset as u16, #loaded_val);
        }
    };

    quote! {
        #load
        #store
    }
}

fn handle_di(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "DI");
    quote! {
        self.cpu.set_ime(false);
    }
}

fn handle_ei(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "EI");
    quote! {
        // self.cpu.set_ime(true);
        self.cpu.ei = 1;
    }
}

fn handle_halt(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "HALT");
    quote! {
        self.cpu.halted = true;
    }
}

// CB prefix
fn handle_cb(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "PREFIX");
    let bytes = entry.bytes;
    let cycles = entry.cycles[0];
    quote! {
        self.cpu.registers.set_pc(self.cpu.registers.pc().wrapping_add(#bytes));
        let opcode = self.mmu.read_byte(self.cpu.registers.pc());
        #cycles + self.execute_cb_opcode(opcode)
    }
}

fn generate_cb_body(entry: &OpcodeEntry) -> TokenStream {
    match entry.mnemonic.as_str() {
        "RLC" => handle_alu_op(entry),
        "RRC" => handle_alu_op(entry),
        "RL" => handle_alu_op(entry),
        "RR" => handle_alu_op(entry),
        "SLA" => handle_alu_op(entry),
        "SRA" => handle_alu_op(entry),
        "SRL" => handle_alu_op(entry),
        "SWAP" => handle_alu_op(entry),
        "BIT" => handle_bit(entry),
        "RES" => handle_res_set(entry),
        "SET" => handle_res_set(entry),
        _ => unreachable!("Unhandled instruction"),
    }
}

fn handle_alu_op(entry: &OpcodeEntry) -> TokenStream {
    assert!(
        entry.mnemonic == "RLC"
            || entry.mnemonic == "RL"
            || entry.mnemonic == "RRC"
            || entry.mnemonic == "RR"
            || entry.mnemonic == "SLA"
            || entry.mnemonic == "SRA"
            || entry.mnemonic == "SRL"
            || entry.mnemonic == "SWAP"
    );

    let op = format_ident!("alu_{}", entry.mnemonic.to_lowercase());

    let reg = &entry.operands[0];
    let reg_name = reg.name.to_lowercase();

    assert!(is_register(&reg_name));

    let getter = format_ident!("{}", reg_name);
    let setter = format_ident!("set_{}", reg_name);
    let load = if reg.immediate {
        quote! {
            let val = self.cpu.registers.#getter();
        }
    } else {
        quote! {
            let address = self.cpu.registers.#getter();
            let val = self.mmu.read_byte(address);
        }
    };
    let store = if reg.immediate {
        quote! {
            self.cpu.registers.#setter(val);
        }
    } else {
        quote! {
            self.mmu.write_byte(address, val);
        }
    };
    quote! {
        #load
        let val = self.cpu.#op(val);
        #store
    }
}

fn handle_bit(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "BIT");

    let bit: u8 = entry.operands[0].name.parse().expect("valid bit number");

    let mask: u8 = 1 << bit;

    let reg = &entry.operands[1];
    let reg_name = reg.name.to_lowercase();

    assert!(is_register(&reg_name));

    let getter = format_ident!("{}", reg_name);
    let load = if reg.immediate {
        quote! {
            let val = self.cpu.registers.#getter();
        }
    } else {
        quote! {
            let address = self.cpu.registers.#getter();
            let val = self.mmu.read_byte(address);
        }
    };

    quote! {
        #load
        let set = val & #mask > 0;
        self.cpu.registers.set_flag(CpuFlags::Z, !set);
        self.cpu.registers.set_flag(CpuFlags::N, false);
        self.cpu.registers.set_flag(CpuFlags::H, true);
    }
}

fn handle_res_set(entry: &OpcodeEntry) -> TokenStream {
    assert!(entry.mnemonic == "RES" || entry.mnemonic == "SET");

    let bit: u8 = entry.operands[0].name.parse().expect("valid bit number");

    let reg = &entry.operands[1];
    let reg_name = reg.name.to_lowercase();

    assert!(is_register(&reg_name));

    let getter = format_ident!("{}", reg_name);
    let setter = format_ident!("set_{}", reg_name);
    let load = if reg.immediate {
        quote! {
            let val = self.cpu.registers.#getter();
        }
    } else {
        quote! {
            let address = self.cpu.registers.#getter();
            let val = self.mmu.read_byte(address);
        }
    };
    let store = if reg.immediate {
        quote! {
            self.cpu.registers.#setter(val);
        }
    } else {
        quote! {
            self.mmu.write_byte(address, val);
        }
    };

    let op = if entry.mnemonic == "SET" {
        let mask: u8 = 1 << bit;
        quote! {
            let val = val | #mask;
        }
    } else {
        let mask: u8 = !(1 << bit);
        quote! {
            let val = val & #mask;
        }
    };

    quote! {
        #load
        #op
        #store
    }
}
