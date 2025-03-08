# XV. Cartridge ROM

[*Return to Index*](../README.md)

[*Previous Chapter*](14-wasm-setup.md)

At this point in our project, a user is able to use either of our frontends to identify a file they would like to attempt to play. Either frontend reads that file in and passes it to our `Cpu` object as a byte array. The `Cpu` then passes that data along to the `Bus`, where it... stops. We need somewhere to store the game ROM while also making it accessible via the Bus. It might be tempting to simply load this data into the first half of our RAM array, but as previously noted, only the smallest of games are able to fit into this space. We shall need a dedicated object to manage this.

Inside of `core/src`, create a new `cart` directory. As we'll eventually see, there's more than one way to map ROM to RAM, and we will handle these different implementations together in this directory. We'll first need to add this new module to `core/src/lib.rs`.

```rust
// In lib.rs

pub mod bus;
pub mod cart;
pub mod cpu;
pub mod utils;
```

Create `cart/mod.rs`, where we will handle all of the common logic for ROM handling. Our first step will be to create a new object which will hold the original ROM data as passed in from the frontend. This data will be stored, unaltered until the game emulation stops. Since Game Boy titles shipped in several different sizes, we cannot guarantee how much space this may occupy; so we will need to store the data in a `Vec<u8>`. We'll also implement a function `load_cart` which finally ends the journey of the ROM data provided from the frontend.

```rust
// In cart/mod.rs

pub struct Cart {
    rom: Vec<u8>,
}

impl Cart {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
        }
    }

    pub fn load_cart(&mut self, rom: &[u8]) {
        self.rom = rom.to_vec();
    }
}
```

We will next need to create the `Bus::load_rom` function that we promised when setting up the frontends. This will accept the ROM data and give it to a `Cart` instance, which will be a member of `Bus`.

```rust
// In bus.rs

use crate::cart::Cart;

pub struct Bus {
    rom: Cart,
    ram: [u8; 0x10000],
}

impl Bus {
    // Unchanged code omitted

    pub fn new() -> Self {
        Self {
            rom: Cart::new(),
            ram: [0; 0x10000],
        }
    }

    pub fn load_rom(&mut self, data: &[u8]) {
        self.rom.load_cart(data);
    }
}
```

Excellent, the ROM data now lives within the `Cart` object. This does raise a question about how the bus's `read_ram` and `write_ram` functions are meant to function. We said that the first 32 KiB of RAM was meant for accessing a copy of ROM data; do we need to maintain a subset of that data ourselves in our `ram` array? While this is certainly possible, and perhaps more similar to how a real Game Boy functions, it's rather wasteful and prone to errors. We have no need to keep two copies of the same data when one will do. Instead, anytime the system wants to access a RAM address that we know falls into the cartridge memory space (address 0x0000 through 0x7FFF), we will simply pass that request along to `Cart` to handle. Anything outside of that range we'll supply from our `ram` array. This means that the first 32 KiB of `ram` would never actually be used, and thus we don't need to keep it. Let's start by adding adding functions to `Cart` to access its data. We'll ignore the intricacies of bank switching for the moment. We'll also create some constants in here for use elsewhere.

```rust
// In cart/mod.rs

pub const ROM_START: u16    = 0x0000;
pub const ROM_STOP: u16     = 0x7FFF;

impl Cart {
    // Unchanged code omitted

    pub fn read_cart(&self, addr: u16) -> u8 {
        // TODO: Handle bank switching
        self.rom[addr as usize]
    }

    pub fn write_cart(&mut self, addr: u16, val: u8) {
        // TODO: Handle bank switching
    }
}
```

As mentioned, this is a bit naive. For the majority of games, their stored ROM data will exceed what we can access with our 16-bit address, so the data we return here won't be correct. For smaller titles though, this will work for now. Those will be small enough to function correctly, which is fortunate for us. Now that `Cart` is handling the Cartridge RAM memory space, we can trim down the size of the bus's array. Note that since we're removing the *front* of the array here, anything that legitimately needs to access it will need to be offset by some constant to ensure it's mapped to the start of the array, in this case offset by 0x8000, the size of Cartridge RAM.

```rust
// In bus.rs

use crate::cart::{Cart, ROM_START, ROM_STOP};

pub struct Bus {
    rom: Cart,
    ram: [u8; 0x8000],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Cart::new(),
            ram: [0; 0x8000],
        }
    }

    // Unchanged code omitted

    pub fn read_ram(&self, addr: u16) -> u8 {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.read_cart(addr)
            },
            _ => {
                let offset = addr - ROM_STOP - 1;
                self.ram[offset as usize]
            }
        }
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.write_cart(addr, val);
            },
            _ => {
                let offset = addr - ROM_STOP - 1;
                self.ram[offset as usize] = val;
            }
        }
    }
}
```

We'll again use Rust's pattern matching to assist us here. If the system is looking for an address corresponding to Cartridge RAM (0x0000 through 0x7FFF), then forward them along to the `Cart` object. If instead they're looking for a value above that, then offset the Cartridge RAM size and use that address to access the bus's `ram` array. With this, either the `desktop` or `html` projects should be able to read a Game Boy file in and store it into the `Cart` object. Feel free to add some debug statements to ensure that this is indeed the case.

Even with the prospect of bank switching looming over us, we shall be satisfied with what we have for now and move away from the cartridge handling. Instead, we shall implement the final item needed for CPU verification -- basic video rendering.
