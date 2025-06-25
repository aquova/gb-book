# XXXVIII. Battery Saving

[*Return to Index*](../README.md)

[*Previous Chapter*](37-render-scanline.md)

At this point, you've likely played around in your emulator a fair bit and after reloading some games you've noted a particular deficient in our emulator -- it doesn't save. This is a considerable oversight, there are a number of games that are impracticable to play through in a single sitting, nor would you want to lose your high scores or collected items. Fortunately, implementing save behavior is only an extension of what we presently have created. Unfortunately though, it requires a different implementation between our desktop and web frontends, so each must be developed separately, for reasons we'll see.

## Background

Firstly, let's cover the concepts of how a Game Boy saves its data. In the modern day, saving is something we take for granted. When a game needs to save, it does so by writing data to some special flash or hard drive space, and when the game is reloaded, the save file can be retrieved from that storage. However, when the Game Boy was designed in the late 1980s, persistent storage devices were far too expensive and large to be included in each game. For many games, they simply didn't save any data at all, once power was turned off all the data in RAM was simply lost. To avoid this, developers came up with a clever solution to avoid losing data stored in RAM -- you simply never turn off the RAM. A small battery could be included within the game cartridge, powering just enough circuitry so external RAM inside the cartridge remained powered, even if the cartridge was removed from the Game Boy entirely.

This is why we refer to persistent Game Boy saves as "battery saves", it's the section of data being maintained by the cartridge battery. This also means that only games with both external RAM and a battery will support game saving. To save a game, our emulator should save dumps of the external RAM whenever necessary, and be able to load these files back into the emulator when needed. You can see why then we need a different solution for desktop versus a browser. Our desktop frontend will store these dumps as a traditional file somewhere on the user's file system. For a browser, we'll need to use the browser's persistent storage interfaces instead.

We begin by modifying `cart/mod.rs`, which is where our cartridge RAM data is stored. While we have already created most of the API we'll need, we currently do not have any functions to retrieve the entirety of cartridge RAM, which we'll need to do when it's time to save it. Thus, we'll add two new functions -- `get_battery_data` and `set_battery_data` -- which will modify our RAM data in its entirety.

```rust
// In cart/mod.rs
// Unchanged code omitted

impl Cart {
    pub fn get_battery_data(&self) -> &[u8] {
        &self.ram
    }

    pub fn set_battery_data(&mut self, data: &[u8]) {
        self.ram.copy_from_slice(data);
    }
}
```

Our eventual goals then are to use these two functions when it comes time to deal with save data. `get_battery_data` will be used to gather the data that will comprise our save files, and `set_battery_data` will be used to restore the data when it is time to load those save files.

As is our usual behavior, we need to create functions in `bus.rs` that makes these functions available to the higher modules. Here, we'll need to create three, for `get_battery_data`, `set_battery_data`, and `has_battery`, which we created in an earlier chapter but has as of yet been unused.

```rust
// In bus.rs
// Unchanged code omitted

impl Bus {
    pub fn get_battery_data(&self) -> &[u8] {
        self.rom.get_battery_data()
    }

    pub fn has_battery(&self) -> bool {
        self.rom.has_battery()
    }

    pub fn set_battery_data(&mut self, data: &[u8]) {
        self.rom.set_battery_data(data);
    }
}
```

Another change is also needed in `bus.rs`. Saving data to a file is not a trivial operation, so it would be best to avoid doing so unless entirely required. However, we also do not want to run the risk of losing the player's save data, thus we need to ensure that it is up to date as much as possible. We need to identify under what criteria it is time to update our save file. This is actually rather simple in practice. Since the save files will contain complete dumps of the external RAM data, we need to update our save file anytime external RAM is written to. Writes to RAM are handled in our appropriately named `write_ram` function. We will change this function to now return a boolean. It will return false until most circumstances, but if our system writes data to the external RAM region of memory, we will return true, signifying that our save file is out of date.

```rust
// In bus.rs
// Unchanged code omitted

impl Bus {
    pub fn write_ram(&mut self, addr: u16, val: u8) -> bool {
        let mut battery_write = false;
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.write_cart(addr, val);
            },
            VRAM_START..=VRAM_STOP => {
                self.ppu.write_vram(addr, val);
            },
            EXT_RAM_START..=EXT_RAM_STOP => {
                self.rom.write_ram(addr, val);
                battery_write = true;
            },
            WRAM_START..=ECHO_STOP => {
                self.wram.write_u8(addr, val)
            },
            OAM_START..=OAM_STOP => {
                self.ppu.write_oam(addr, val);
            },
            IO_START..=IO_STOP => {
                self.io.write_u8(addr, val);
            },
            LCD_REG_START..=LCD_REG_STOP => {
                if addr == OAM_DMA {
                    self.dma_transfer(val);
                }
                self.ppu.write_lcd_reg(addr, val)
            },
            HRAM_START..=HRAM_STOP => {
                let relative_addr = addr - HRAM_START;
                self.hram[relative_addr as usize] = val;
            },
            _ => {}
        }
        battery_write
    }
}
```

We now move up another layer to our `cpu/mod.rs` file. We'll need to continue exposing the functions we added to `bus.rs`, so that they're accessible to the frontends, and we'll need to handle the new boolean return value from `write_ram`. We'll handle that behavior first. Nearly 40 chapters into this guide, we're going to add a new member variable to `Cpu`, a boolean to store whether the external RAM data is dirty<sup>1</sup> or not. This `dirty_battery` flag can then be accessed by the frontends as a simple check to know whether we need to update our save file or not. This flag will be initially set to be false, and shall be updated whenever we call the Bus's `write_ram` function. We'll also need a public function to reset the dirty flag, which we'll use after we've updated our save file, signifying we don't need to update it again.

<sup>1</sup> Dirty here means that the data in question has been modified since we last saved it.

```rust
// In cpu/mod.rs
// Unchanged code omitted

pub struct Cpu {
    pc: u16,
    sp: u16,
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
    irq_enabled: bool,
    halted: bool,
    bus: Bus,
    last_read: Option<u16>,
    last_write: Option<u16>,
    dirty_battery: bool,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            pc: 0x0100,
            sp: 0xFFFE,
            a: 0x01,
            b: 0x00,
            c: 0x13,
            d: 0x00,
            e: 0xD8,
            f: 0xB0,
            h: 0x01,
            l: 0x4D,
            irq_enabled: false,
            halted: false,
            bus: Bus::new(),
            last_read: None,
            last_write: None,
            dirty_battery: false,
        };

        // Magic values for RAM initialization
        cpu.write_ram(0xFF10, 0x80);
        cpu.write_ram(0xFF11, 0xBF);
        cpu.write_ram(0xFF12, 0xF3);
        cpu.write_ram(0xFF14, 0xBF);
        cpu.write_ram(0xFF16, 0x3F);
        cpu.write_ram(0xFF19, 0xBF);
        cpu.write_ram(0xFF1A, 0x7F);
        cpu.write_ram(0xFF1B, 0xFF);
        cpu.write_ram(0xFF1C, 0x9F);
        cpu.write_ram(0xFF1E, 0xBF);
        cpu.write_ram(0xFF20, 0xFF);
        cpu.write_ram(0xFF23, 0xBF);
        cpu.write_ram(0xFF24, 0x77);
        cpu.write_ram(0xFF25, 0xF3);
        cpu.write_ram(0xFF26, 0xF1); // 0xF0 for SGB
        cpu.write_ram(0xFF40, 0x91);
        cpu.write_ram(0xFF47, 0xFC);
        cpu.write_ram(0xFF48, 0xFF);
        cpu.write_ram(0xFF49, 0xFF);

        cpu
    }

    pub fn clean_battery(&mut self) {
        self.dirty_battery = false;
    }

    pub fn is_battery_dirty(&self) -> bool {
        self.dirty_battery
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        self.last_write = Some(addr);
        self.dirty_battery |= self.bus.write_ram(addr, val);
    }
}
```

In addition, we'll need to hook up our battery data public functions as well, so that the frontends can load or save the entirety of the battery data.

```rust
// In cpu/mod.rs
// Unchanged code omitted

impl Cpu {
    pub fn get_battery_data(&self) -> &[u8] {
        self.bus.get_battery_data()
    }

    pub fn set_battery_data(&mut self, data: &[u8]) {
        self.bus.set_battery_data(data);
    }
}
```

## Desktop Frontend

With all the various pieces plugged in, it's time to add support to battery saving to the frontends. This will require us to check if our `dirty_battery` flag is set, and if so, update our save file on the filesystem and to reset the dirty flag. Likewise, we'll need to add a function to load in a save file if one is detected when the emulation first begins.

We'll start with the saving functionality. The main emulation loop takes place in our `tick_until_draw` function, which repeatedly calls the `tick` function and checks for various debug functionality, if you elected to add it. While it's true that we could be updating the external RAM data every single tick, it's a bit overkill to check with that granularity. Since this is a rather expensive operation, we'll instead only update our save files once per frame, giving a nice balance of responsiveness and performance.

Here, we'll check if the dirty flag is set, and if so, call a new function to perform the save. Notice that the parameters for `tick_until_draw` have changed. We'll need to pass the name of the ROM file into our `write_battery_save` function, so that we can modify the filename slightly to use as the name of our save file.

```rust
// In desktop/main.rs
// Unchanged code omitted

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
    // TODO
}
```

In `write_battery_save`, we'll first need to construct the name of the save file, which for simplicity will just be the name of the ROM with ".sav" appended to the end. We'll then get the entirety of external RAM from the backend, and save it into that file, overwriting it if needed.

```rust
// In desktop/main.rs
// Unchanged code omitted

use std::fs::{File, OpenOptions};
use std::io::prelude::*;

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
```

We'll also need to implement the opposite behavior -- loading a save back from a file. This operation is simpler to place, as it only needs to happen once when the game is first loaded in.

```rust
// In desktop/main.rs
// Unchanged code omitted

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
    // Etc...
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
```

`load_battery_save` works in reverse to `write_battery_save` in many ways. The battery data is read from a file into a buffer then passed down into the core, as opposed to being fetched from the core and saved into a file.
