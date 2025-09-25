use std::{path::PathBuf, process::exit};

use clap::Parser;
use gb_emulator::{
    cartridge::Cartridge,
    gb::{GameBoy, GameBoyConfig},
};

fn main() {
    let config = GameBoyConfig::parse();

    let cartridge = match Cartridge::load_cartridge(&PathBuf::from(&config.cartridge_path)) {
        Ok(cart) => {
            #[cfg(not(feature = "gb_doctor"))]
            println!("Loaded ROM: {}", cart.title);
            cart
        }
        Err(e) => {
            eprintln!("Failed to load rom from {}: {}", config.cartridge_path, e);
            exit(1);
        }
    };

    let mut gb = GameBoy::new(cartridge, config);
    loop {
        gb.tick();
    }
}
