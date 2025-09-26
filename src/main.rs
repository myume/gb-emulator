use std::{path::PathBuf, process::exit};

use clap::Parser;
use gb_emulator::{
    cartridge::Cartridge,
    gb::{GameBoy, GameBoyConfig},
    ppu::{GB_SCREEN_HEIGHT, GB_SCREEN_WIDTH},
};
use sdl2::{event::Event, keyboard::Keycode};

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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window(
            "Gameboy Emulator",
            GB_SCREEN_WIDTH as u32,
            GB_SCREEN_HEIGHT as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.clear();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        gb.tick();
        canvas.present();
    }
}
