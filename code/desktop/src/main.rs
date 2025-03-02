mod debug;

use crate::debug::Debugger;

use gb_core::cpu::Cpu;
use gb_core::io::Buttons;
use gb_core::utils::{SCREEN_HEIGHT, SCREEN_WIDTH, DISPLAY_BUFFER};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use std::env;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::Read;
use std::process::exit;

const SCALE: u32 = 3;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        println!("Please specify a ROM location: cargo run path/to/game");
        return;
    }

    let mut gbd = Debugger::new();
    let mut gb = Cpu::new();
    let filename = &args[1];
    let rom = load_rom(filename);
    gb.load_rom(&rom);
    load_battery_save(&mut gb, filename);
    let title = gb.get_title();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window(title, WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered().opengl().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(Keycode::Space), ..} => {
                    gbd.set_debugging(true);
                },
                Event::KeyDown{keycode: Some(keycode), ..} => {
                    if let Some(button) = key2btn(keycode) {
                        gb.press_button(button, true);
                    }
                },
                Event::KeyUp{keycode: Some(keycode), ..} => {
                    if let Some(button) = key2btn(keycode) {
                        gb.press_button(button, false);
                    }
                },
                _ => {}
            }
        }

        // Keep ticking until told to stop
        tick_until_draw(&mut gb, &mut gbd, filename);
        let frame = gb.render();
        draw_screen(&frame, &mut canvas);
    }
}

fn draw_screen(data: &[u8], canvas: &mut Canvas<Window>) {
    for i in (0..DISPLAY_BUFFER).step_by(4) {
        canvas.set_draw_color(Color::RGB(data[i], data[i + 1], data[i + 2]));
        let pixel = i / 4;
        let x = (pixel % SCREEN_WIDTH) as u32;
        let y = (pixel / SCREEN_WIDTH) as u32;

        let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
        canvas.fill_rect(rect).unwrap();
    }
    canvas.present();
}

fn key2btn(key: Keycode) -> Option<Buttons> {
    match key {
        Keycode::Down =>        { Some(Buttons::Down)   },
        Keycode::Up =>          { Some(Buttons::Up)     },
        Keycode::Left =>        { Some(Buttons::Left)   },
        Keycode::Right =>       { Some(Buttons::Right)  },
        Keycode::Return =>      { Some(Buttons::Start)  },
        Keycode::Backspace =>   { Some(Buttons::Select) },
        Keycode::X =>           { Some(Buttons::A)      },
        Keycode::Z =>           { Some(Buttons::B)      },
        _ =>                    { None                  }
    }
}

fn load_battery_save(gb: &mut Cpu, gamename: &str) {
    if gb.has_battery() {
        let mut battery_data: Vec<u8> = Vec::new();
        let mut filename = gamename.to_owned();
        filename.push_str(".sav");

        let f = OpenOptions::new().read(true).open(filename);
        if f.is_ok() {
            f.unwrap().read_to_end(&mut battery_data).expect("Error reading save file");
            gb.set_battery_data(&battery_data);
        }
    }
}

fn load_rom(path: &str) -> Vec<u8> {
    let mut buffer: Vec<u8> = Vec::new();

    let mut f = File::open(path).expect("Error opening ROM file");
    f.read_to_end(&mut buffer).expect("Error loading ROM");
    buffer
}

fn tick_until_draw(gb: &mut Cpu, gbd: &mut Debugger, gamename: &str) {
    loop {
        let render = gb.tick();

        gbd.check_exec_breakpoints(gb.get_pc());
        if let Some(addr) = gb.get_read() {
            gbd.check_read_breakpoints(addr);
        }
        if let Some(addr) = gb.get_write() {
            gbd.check_write_breakpoints(addr);
        }
        if gbd.is_debugging() {
            gbd.print_info();
            let quit = gbd.debugloop(gb);
            if quit {
                exit(0);
            }
        }

        if render {
            break;
        }
    }

    if gb.is_battery_dirty() {
        write_battery_save(gb, &gamename);
    }
}

fn write_battery_save(gb: &mut Cpu, gamename: &str) {
    if gb.has_battery() {
        let battery_data = gb.get_battery_data();
        let mut filename = gamename.to_owned();
        filename.push_str(".sav");

        let mut file = OpenOptions::new().write(true).create(true).open(filename).expect("Error opening save file");
        file.write(battery_data).unwrap();
        gb.clean_battery();
    }
}
