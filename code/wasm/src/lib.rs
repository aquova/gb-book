use gb_core::cpu::Cpu;
use gb_core::io::Buttons;
use gb_core::utils::{SCREEN_HEIGHT, SCREEN_WIDTH};

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, KeyboardEvent};

#[wasm_bindgen]
pub struct GB {
    cpu: Cpu,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl GB {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<GB, JsValue> {
        let cpu = Cpu::new();

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()
            .map_err(|_| ())
            .unwrap();

        let ctx = canvas.get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let gb = GB { cpu, ctx };
        Ok(gb)
    }

    #[wasm_bindgen]
    pub fn get_title(&self) -> String {
        self.cpu.get_title().to_string()
    }

    #[wasm_bindgen]
    pub fn load_rom(&mut self, data: Uint8Array) {
        let mut rom: Vec<u8> = Vec::new();

        for i in 0..data.byte_length() {
            rom.push(data.get_index(i));
        }
        self.cpu.load_rom(&rom);
    }

    #[wasm_bindgen]
    pub fn press_button(&mut self, event: KeyboardEvent, pressed: bool) {
        let key = event.key();
        if let Some(button) = key2btn(&key) {
            self.cpu.press_button(button, pressed);
        }
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) -> bool {
        self.cpu.tick()
    }

    #[wasm_bindgen]
    pub fn draw_screen(&mut self) {
        let mut framebuffer = self.cpu.render();
        let img_data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut framebuffer), SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32).unwrap();
        self.ctx.put_image_data(&img_data, 0.0, 0.0).unwrap();
    }
}

fn key2btn(key: &str) -> Option<Buttons> {
    match key {
        "ArrowDown" =>    { Some(Buttons::Down)   },
        "ArrowUp" =>      { Some(Buttons::Up)     },
        "ArrowRight" =>   { Some(Buttons::Right)  },
        "ArrowLeft" =>    { Some(Buttons::Left)   },
        "Enter" =>        { Some(Buttons::Start)  },
        "Backspace" =>    { Some(Buttons::Select) },
        "x" =>            { Some(Buttons::A)      },
        "z" =>            { Some(Buttons::B)      },
        _ =>              { None                  }
    }
}
