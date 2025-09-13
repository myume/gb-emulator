use std::{env, fs::File, io::Write, path::Path};

use crate::codegen::generate_opcode_instructions;

mod codegen;

fn main() {
    let json_path = Path::new("build").join("opcodes.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let instructions = generate_opcode_instructions(&json_path);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("instruction.rs");
    let mut f = File::create(&dest_path).unwrap();
    f.write_all(instructions.as_bytes())
        .expect("Failed to write instructions to file");
}
