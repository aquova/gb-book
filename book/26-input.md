# Chapter XXVI. Input

[*Return to Index*](../README.md)

[*Previous Chapter*](25-window-layer.md)

Let's take a brief break from the rendering and focus on another critical aspect of Game Boy emulation &mdash; actually being able to control the games. The Game Boy only has eight buttons in total &mdash; A, B, Select, Start, and the four directions on the D-Pad: Up, Down, Left, Right. These each have a binary state, either off or on, and thus can be stored with a single bit each.

Given that there's eight different buttons, it follows that all the button information can be stored within a single byte, at address 0xFF00 to be precise. Somewhat counter-intuitively though, this byte is not organized as a simple bitfield. Instead, only the four low bits store button information, with bits four and five signalling which group of four are available at that time.

| Bit | Purpose                                         |
| --- | ----------------------------------------------- |
|  7  | Unused                                          |
|  6  | Unused                                          |
|  5  | If 0, lower bits holds Start/Select/B/A presses |
|  4  | If 0, lower bits holds D-pad presses            |

| Bit | Purpose if bit 5 is 0 |
| --- | --------------------- |
|  3  | Start                 |
|  2  | Select                |
|  1  | B                     |
|  0  | A                     |

| Bit | Purpose if bit 4 is 0 |
| --- | --------------------- |
|  3  | Down                  |
|  2  | Up                    |
|  1  | Left                  |
|  0  | Right                 |

For example, if bit five is 0, then the lower bits contain the information for the Start, Select, B, and A buttons. If bit four is 0, then those bits hold the status of Down, Up, Left, Right. Note that if the button is pressed, then the corresponding bit is 0, not 1, which might run counter intuitively to what you would assume. Likewise, the selection registers are set if the bit is a 0. Bits 6 and 7 are unused.

If both bits 4 and 5 are set, then neither option is chosen and the lower four bits will be set to 0xF, meaning no buttons are selected. However, both bits being set to 0 seems to be undefined behavior as far as I can tell. Most "well behaving" games won't intentionally do this, but in the even that they do, we'll also just set the lower bits to 0xF again, being consistent with the other condition.

## IO class

We'll create a new object that will hold the status of the different buttons, and provide an interface for reading and writing data. We'll store this in a new `io.rs` file. We'll first add a new set of enum values for each of the button types.

```rust
// In io.rs

pub enum Buttons {
    A       = 0,
    B       = 1,
    Select  = 2,
    Start   = 3,
    Right   = 4,
    Left    = 5,
    Up      = 6,
    Down    = 7,
}

const DPAD_BUTTONS: [Buttons; 4] = [
    Buttons::Right, Buttons::Left, Buttons::Up, Buttons::Down,
];

const FACE_BUTTONS: [Buttons; 4] = [
    Buttons::A, Buttons::B, Buttons::Select, Buttons::Start,
];
```

There's a few different ways to store the status of each of the buttons. We could store them simply in an array, although we then need to assign index values to each of the buttons (which is what I've done here). Other ideas would be to use something like a hashmap, although we'd then need to initialize it or handle cases where the value doesn't exist. For simplicity, we'll go with the array. The two constant arrays are the two groupings of buttons as outlined above. Note that the numbers assigned to each of the enum values is arbitrary, we simply need to make sure that each has a unique index. The ordering in the two constant arrays though, is not arbitrary. The Game Boy is expecting the bit flags to be structured in a particular order, as seen in the diagram above, and these arrays must reflect that.

Next, we'll create the actual struct that will hold this data. It will have four fields &mdash; an eight element array of booleans which holds the state of whether each button is pressed. There are two flags which the real Game Boy uses for signaling whether grouping of buttons should be read. Again, this system is odd, as if both bits are set, then which grouping is read? What about if neither are set? In those cases, the unit returns that no buttons are being pressed (which would be 0x0F, since a set bit means button released). Given we've named this file `io.rs`, we'll also eventually store other I/O behavior here; we'll start by storing block of memory that deals with I/O. The Joypad is specifically assigned 0xFF00, but there are other I/O devices, such as the serial port and audio. We won't deal with either for now, but they will still need a place to live.

```rust
// In io.rs
// Unchanged code omitted

pub const IO_START: u16   = 0xFF00;
pub const IO_STOP: u16    = 0xFF3F;

const JOYPAD_ADDR: u16    = 0xFF00;
const IO_SIZE: usize      = (IO_STOP - IO_START + 1) as usize;

pub fn IO {
    buttons: [bool; 8],
    dpad_selected: bool,
    face_selected: bool,
    ram: [u8; IO_SIZE],
}

impl IO {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            dpad_selected: false,
            face_selected: false,
            ram: [0; IO_SIZE],
        }
    }
}
```

To complete this class, we'll add four functions. This includes setting the button's state when the user interacts with the system, and then functions for reading and writing the data as an 8-bit value. Note that when writing to the Joypad register, only two bits are actually affected, the two bits for which grouping is selected. The button bits themselves get their values from the hardware (or in our case, with `set_button`).

```rust
// In io.rs
// Unchanged code omitted

const FACE_SELECT_BIT: u8 = 5;
const DPAD_SELECT_BIT: u8 = 4;

impl IO {
    pub fn read_u8(&self, addr: u16) -> u8 {
        if addr == JOYPAD_ADDR {
            self.read_joypad()
        } else {
            let relative_addr = addr - IO_START;
            self.ram[relative_addr as usize]
        }
    }

    fn read_joypad(&self) -> u8 {
        if self.face_selected == self.dpad_selected {
            return 0;
        }

        let mut ret = 0;
        if self.dpad_selected {
            for btn in DPAD_BUTTONS {
                let idx = btn as usize;
                let mask = (if self.buttons[idx] { 0 } else { 1 }) << (idx - 4);
                ret |= mask;
            }
        } else {
            for btn in FACE_BUTTONS {
                let idx = btn as usize;
                let mask = (if self.buttons[idx] { 0 } else { 1 }) << idx;
                ret |= mask;
            }
        }
        ret
    }

    pub fn set_button(&mut self, button: Buttons, pressed: bool) {
        self.buttons[button as usize] = pressed;
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        if addr == JOYPAD_ADDR {
            self.face_selected = !val.get_bit(FACE_SELECT_BIT);
            self.dpad_selected = !val.get_bit(DPAD_SELECT_BIT);
        } else {
            let relative_addr = addr - IO_START;
            self.ram[relative_addr as usize] = val;
        }
    }
}
```

## Connecting to the Frontends

With the joypad functionality in place, we need to hook it up to the two frontends so that the emulator properly recognizes button inputs. Recall to when we implemented reading in a file and loading it as a ROM. We created a new object to hold the data, hooked it up to the bus, then added API up to the frontend to supply that data. It was then up to each of the two frontends to implement loading in a file in their own way. We're going to do something very similar here, again hooking up this new class to the bus, adding API access to the frontends, then implementing keypress detection in their own ways.

We'll begin with the bus. We'll add a new `io` member variable to its struct, then amend `read_ram` and `write_ram` to correctly handle when the buttons are being written to, which unlike the previous examples is only a single memory address. We'll also add a `press_button` function for our parents to use.

```rust
// In bus.rs
// Unchanged code omitted

use crate::io::{IO, Buttons};

pub struct Bus {
    rom: Cart,
    ppu: Ppu,
    io: IO,
    ram: [u8; 0x6000],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Cart::new(),
            ppu: Ppu::new(),
            io: IO::new(),
            ram: [0; 0x6000],
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.read_cart(addr)
            },
            VRAM_START..=VRAM_STOP => {
                self.ppu.read_vram(addr)
            },
            OAM_START..=OAM_STOP => {
                self.ppu.read_oam(addr)
            },
            IO_START..=IO_STOP => {
                self.io.read_u8(addr)
            },
            LCD_REG_START..=LCD_REG_STOP => {
                self.ppu.read_lcd_reg(addr)
            },
            _ => {
                let offset = addr - VRAM_STOP - 1;
                self.ram[offset as usize]
            }
        }
    }

    pub fn press_button(&mut self, button: Buttons, pressed: bool) {
        self.joypad.set_button(button, pressed);
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.write_cart(addr, val);
            },
            VRAM_START..=VRAM_STOP => {
                self.ppu.write_vram(addr, val);
            },
            OAM_START..=OAM_STOP => {
                self.ppu.write_oam(addr, val);
            },
            IO_START..=IO_STOP => {
                self.io.write_u8(addr, val);
            },
            LCD_REG_START..=LCD_REG_STOP => {
                self.ppu.write_lcd_reg(addr, val)
            },
            _ => {
                let offset = addr - VRAM_STOP - 1;
                self.ram[offset as usize] = val;
            }
        }
    }
}
```

Our Swiss cheesening of the `ram` array continues. I promise at some point we'll clean that up. Next, we need to add a `press_button` function to `cpu/mod.rs`. This function is what the two frontends will call when they detect a button press/release. This will both call the bus's `press_button` function, as well as signal that there is a Joypad interrupt ready. Fortunately, we've already laid the groundwork for handling interrupts, so this is the only step we need to take to initiate a Joypad interrupt. This doesn't mean that all games will necessarily utilize this interrupt, but it needs to be there for the ones that do.

```rust
// In cpu/mod.rs
// Unchanged code omitted

use crate::io::Buttons;

impl Cpu {
    pub fn press_button(&mut self, button: Buttons, pressed: bool) {
        self.bus.press_button(button, pressed);
        self.enable_irq_type(Interrupts::Joypad, true);
    }
}
```

## Desktop Frontend

We'll begin with the `desktop` frontend. We've already been listening for keypresses here, such as for exiting the emulator when escape is pressed. Rather than add many entries into the event pump `match` statement, lets add a helper function that will check if a pressed key is one that we have an interest in, and if so, which Game Boy button that corresponds to.

```rust
// In desktop/src/mod.rs

use gb_core::io::Buttons;

fn main() {
    // Unchanged code omitted
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
    }
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
```

The `key2btn` function checks if any of the key in question is one of the ones we care about, then returns an optional. We then add two new events, one for `KeyDown` and another for `KeyUp`, which passes in their keycode and signals to the core to press or release their button, respectively. This does mean that for our emulator, the key mapping is hardcoded. I've chosen on that I personally use when playing Game Boy games, but you can of course edit this, or go a step further and implement some sort of configuration file. That will be outside the scope of this tutorial however.

That's all that is needed for the `desktop` frontend. It might be hard to test out before we have the ability to render any sprites. We now turn our attention to the `wasm` frontend to implement the same behavior.

## WebAssembly Frontend

As we've seen previously when implementing `wasm` behavior, the line can be a bit fuzzy for whether functionality should live in the `wasm` or `html` modules. My personal preference here is to have JavaScript pass `wasm` the key in question, and to allow `wasm` to perform a nearly identical operation as we just implemented. This way we can utilize optionals and match statements in a similar manner.

```rust
// In wasm/src/lib.rs
// Unchanged code omitted

use gb_core::io::Buttons;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData, KeyboardEvent};

#[wasm_bindgen]
impl GB {
    // Unchanged code omitted

    #[wasm_bindgen]
    pub fn press_button(&mut self, event: KeyboardEvent, pressed: bool) {
        let key = event.key();
        if let Some(button) = key2btn(&key) {
            self.cpu.press_button(button, pressed);
        }
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
```

All that remains now is to instruct JavaScript to listen for key events and to then pass them along to the `press_button` function.

```javascript
// In html/index.js
// Unchanged code omitted

async function run() {
    document.addEventListener("keydown", function(e) {
        gb.press_button(e, true)
    })

    document.addEventListener("keyup", function(e) {
        gb.press_button(e, false)
    })
}
```

With that, the `core` should be receiving button events from both frontends, with which the CPU can access by reading or writing the Joypad memory address. Let's return to the PPU and add the final rendering layer so we can see the fruit of our efforts.

[*Next Chapter*](27-oam.md)
