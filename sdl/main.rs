use std::{path::PathBuf, process::exit};

use clap::Parser;
use gb_emulator::{
    cartridge::Cartridge,
    gb::GameBoy,
    ppu::{GB_SCREEN_HEIGHT, GB_SCREEN_WIDTH},
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::TextureAccess};

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub cartridge_path: String,

    #[arg(short, long)]
    pub print_serial: bool,
}

fn main() {
    let args = Args::parse();

    let cartridge = match Cartridge::load_cartridge(&PathBuf::from(&args.cartridge_path)) {
        Ok(cart) => cart,
        Err(e) => {
            eprintln!("Failed to load rom from {}: {}", args.cartridge_path, e);
            exit(1);
        }
    };
    let mut gb = GameBoy::new(cartridge, args.print_serial);

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

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(
            PixelFormatEnum::ARGB8888,
            TextureAccess::Streaming,
            GB_SCREEN_WIDTH as u32,
            GB_SCREEN_HEIGHT as u32,
        )
        .expect("Failed to create texture");

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
        texture
            .update(None, gb.pixel_data(), GB_SCREEN_WIDTH * 4)
            .expect("Failed to update texture");
        canvas
            .copy(&texture, None, None)
            .expect("Failed to copy texture to canvas");
        canvas.present();
    }
}
