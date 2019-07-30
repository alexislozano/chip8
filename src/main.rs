mod chip8;
use crate::chip8::Chip8;

use std::fs::read;
use std::{thread, time};

use sdl2::audio::{AudioCallback, AudioSpecDesired, AudioStatus};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use clap::{crate_authors, crate_name, crate_version, App, Arg};

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for x in out.iter_mut() {
            *x = if self.phase >= 0.0 && self.phase < 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn set_key(chip8: &mut Chip8, keycode: Keycode, keydown: bool) {
    match keycode {
        Keycode::Num1 => chip8.set_key(0, keydown),
        Keycode::Num2 => chip8.set_key(1, keydown),
        Keycode::Num3 => chip8.set_key(2, keydown),
        Keycode::Num4 => chip8.set_key(3, keydown),
        Keycode::Q => chip8.set_key(4, keydown),
        Keycode::W => chip8.set_key(5, keydown),
        Keycode::E => chip8.set_key(6, keydown),
        Keycode::R => chip8.set_key(7, keydown),
        Keycode::A => chip8.set_key(8, keydown),
        Keycode::S => chip8.set_key(9, keydown),
        Keycode::D => chip8.set_key(10, keydown),
        Keycode::F => chip8.set_key(11, keydown),
        Keycode::Z => chip8.set_key(12, keydown),
        Keycode::X => chip8.set_key(13, keydown),
        Keycode::C => chip8.set_key(14, keydown),
        Keycode::V => chip8.set_key(15, keydown),
        _ => (),
    }
}

fn run(file: Vec<u8>) {
    let size = 10;
    let sdl_context = sdl2::init().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chip8", 64 * size, 32 * size)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| SquareWave {
            phase_inc: 440.0 / spec.freq as f32,
            phase: 0.0,
            volume: 0.25,
        })
        .unwrap();

    let mut chip8 = Chip8::new();
    chip8.load_font();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    chip8.load_game(file);
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => {
                    set_key(&mut chip8, keycode, true);
                }
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => {
                    set_key(&mut chip8, keycode, false);
                }
                _ => {}
            }
        }

        chip8.run_cycle();

        match device.status() {
            AudioStatus::Stopped | AudioStatus::Paused => {
                if chip8.sound_timer() > 0 {
                    device.resume();
                }
            }
            AudioStatus::Playing => {
                if chip8.sound_timer() == 0 {
                    device.pause();
                }
            }
        }

        let display = chip8.display();
        for (h, row) in display.iter().enumerate() {
            for (w, pixel) in row.iter().enumerate() {
                if *pixel {
                    canvas.set_draw_color(Color::RGB(255, 255, 255));
                } else {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                }
                canvas
                    .fill_rect(Rect::new(
                        w as i32 * size as i32,
                        h as i32 * size as i32,
                        size,
                        size,
                    ))
                    .unwrap();
            }
        }
        canvas.present();

        thread::sleep(time::Duration::from_millis(2));
    }
}

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("An implementation of a Chip 8 emulator")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("Sets a file to process")
                .takes_value(true),
        )
        .get_matches();

    match matches.value_of("file") {
        Some(filename) => match read(filename) {
            Ok(file) => {
                run(file);
            }
            Err(_) => eprintln!("Cannot open file"),
        },
        None => eprintln!("Please set the -f flag with a file"),
    }
}
