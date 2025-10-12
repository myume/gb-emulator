use std::{
    path::PathBuf,
    process::exit,
    thread,
    time::{Duration, Instant},
};

use clap::Parser;
use gb_emulator::{
    cartridge::Cartridge,
    cpu::Cycles,
    gb::{GBButton, GameBoy, JoypadButton, JoypadDpad},
    ppu::{GB_SCREEN_HEIGHT, GB_SCREEN_WIDTH},
};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::PixelFormatEnum,
    rect::Rect,
    render::TextureAccess,
};

#[derive(Parser, Debug, Default)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub cartridge_path: String,

    #[arg(short, long)]
    pub print_serial: bool,
}

// Game Boy hardware constants
const CPU_CYCLES_PER_SECOND: u32 = 4_194_304;
const FPS: u32 = 60;
const CYCLES_PER_FRAME: u32 = (CPU_CYCLES_PER_SECOND as f64 / FPS as f64) as u32;
const FRAME_DURATION: Duration = Duration::from_nanos((1_000_000_000u32 / FPS) as u64);

fn get_screen_rect(win_w: u32, win_h: u32) -> Rect {
    let gb_aspect_ratio = GB_SCREEN_WIDTH as f32 / GB_SCREEN_HEIGHT as f32;
    let win_aspect_ratio = win_w as f32 / win_h as f32;

    let (w, h) = if win_aspect_ratio > gb_aspect_ratio {
        let h = win_h;
        let w = (h as f32 * gb_aspect_ratio) as u32;
        (w, h)
    } else {
        let w = win_w;
        let h = (w as f32 / gb_aspect_ratio) as u32;
        (w, h)
    };

    let x = (win_w - w) / 2;
    let y = (win_h - h) / 2;

    Rect::new(x as i32, y as i32, w, h)
}

fn main() {
    let args = Args::parse();
    let mut speedup = 1;

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
            (GB_SCREEN_WIDTH * 4) as u32,
            (GB_SCREEN_HEIGHT * 4) as u32,
        )
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let (win_w, win_h) = canvas.window().size();
    let mut screen_rect = get_screen_rect(win_w, win_h);
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
    let mut cycles_counter: Cycles = 0;

    'running: loop {
        let frame_start_time = Instant::now();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,

                // up down left right
                Event::KeyDown {
                    keycode: Some(Keycode::W),
                    ..
                } => gb.on_button_press(GBButton::Dpad(JoypadDpad::Up)),
                Event::KeyUp {
                    keycode: Some(Keycode::W),
                    ..
                } => gb.on_button_release(GBButton::Dpad(JoypadDpad::Up)),

                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => gb.on_button_press(GBButton::Dpad(JoypadDpad::Down)),
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => gb.on_button_release(GBButton::Dpad(JoypadDpad::Down)),

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => gb.on_button_press(GBButton::Dpad(JoypadDpad::Left)),
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => gb.on_button_release(GBButton::Dpad(JoypadDpad::Left)),

                Event::KeyDown {
                    keycode: Some(Keycode::D),
                    ..
                } => gb.on_button_press(GBButton::Dpad(JoypadDpad::Right)),
                Event::KeyUp {
                    keycode: Some(Keycode::D),
                    ..
                } => gb.on_button_release(GBButton::Dpad(JoypadDpad::Right)),

                // start select A B
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => gb.on_button_press(GBButton::Button(JoypadButton::Start)),
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => gb.on_button_release(GBButton::Button(JoypadButton::Start)),

                Event::KeyDown {
                    keycode: Some(Keycode::Tab),
                    ..
                } => gb.on_button_press(GBButton::Button(JoypadButton::Select)),
                Event::KeyUp {
                    keycode: Some(Keycode::Tab),
                    ..
                } => gb.on_button_release(GBButton::Button(JoypadButton::Select)),

                Event::KeyDown {
                    keycode: Some(Keycode::J),
                    ..
                } => gb.on_button_press(GBButton::Button(JoypadButton::A)),
                Event::KeyUp {
                    keycode: Some(Keycode::J),
                    ..
                } => gb.on_button_release(GBButton::Button(JoypadButton::A)),

                Event::KeyDown {
                    keycode: Some(Keycode::K),
                    ..
                } => gb.on_button_press(GBButton::Button(JoypadButton::B)),
                Event::KeyUp {
                    keycode: Some(Keycode::K),
                    ..
                } => gb.on_button_release(GBButton::Button(JoypadButton::B)),

                Event::Window {
                    win_event: WindowEvent::Resized(w, h),
                    ..
                } => {
                    screen_rect = get_screen_rect(w as u32, h as u32);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Backspace),
                    ..
                } => {
                    if speedup > 1 {
                        speedup = 1;
                    } else {
                        speedup = 20
                    }
                }
                _ => {}
            }
        }

        while cycles_counter < CYCLES_PER_FRAME as Cycles * speedup {
            cycles_counter += gb.tick();
        }
        cycles_counter %= CYCLES_PER_FRAME as Cycles;

        let elapsed = frame_start_time.elapsed();
        if let Some(sleep_duration) = FRAME_DURATION.checked_sub(elapsed) {
            thread::sleep(sleep_duration);
        }

        texture
            .update(None, gb.pixel_data(), GB_SCREEN_WIDTH * 4)
            .expect("Failed to update texture");

        canvas.clear();
        canvas
            .copy(&texture, None, screen_rect)
            .expect("Failed to copy texture to canvas");
        canvas.present();
    }
}
