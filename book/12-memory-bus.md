# XII. Memory Bus

[*Return to Index*](../README.md)

[*Previous Chapter*](11-final-misc.md)

It's finally time to move on from the CPU to other subsystems of the Game Boy. As part of adding the different opcodes, we commonly relied on two incomplete functions -- `read_ram` and `write_ram` -- which are to handle accessing memory at a specified address. We'll begin by implementing our basic memory structure and discussing how the Game Boy views its RAM.

While you might be familiar with how modern operating systems deal with memory, older systems such as our Game Boy utilized it in a different way. On a phone or modern desktop computer, the operating system has a large pool of RAM available to use, and when some program requests some, the OS sections off a block of memory for that program to use, and disallows any other program from accessing it. This means that while a program thinks it has the entire RAM for itself, it's actually being given a block at a somewhat arbitrary spot in the middle. This is known as *virtual memory*, and this is not how the Game Boy does things. With only one program running at a time, no operating system, and very little RAM to use, the Game Boy instead relies on breaking up the RAM into very specific purposes, often referred to as its *memory map*. These strict definitions means that the CPU always knows where data relevant to its operation resides, and the same is the case for the video processing unit, I/O, general usage, etc.

![A diagram showing the regions of the Game Boy's memory map](img/memory-map.svg)

The Game Boy Memory Map

Let's go over the Game Boy memory map. As you're well aware by now, the CPU uses 16-bit registers for memory addressing, meaning it can specify an address between 0x0000 and 0xFFFF -- a total of 64 KiB. The first half of that, from address 0x0000 to 0x7FFF, is for cartridge ROM data. As you've seen, the CPU gets its opcodes from RAM, not the cartridge directly, so game code has to be loaded into RAM somewhere. This only totals up to 32 KiB however, which would imply that the maximum size a Game Boy game could be is also 32 KiB. That is a very small amount of storage, even for games at the time, although some titles are actually this size (*Tetris* is the best known example).

Fortunately for Game Boy developers, a mechanism called "bank switching" was added to the Game Boy, which allows half (16 KiB, from 0x4000 to 0x7FFF) of the cartridge ROM address space to be switched out at will. The first 16 KiB, from 0x0000 to 0x3FFF, is always the first 16 KiB of ROM, known as "Bank 0". This isn't just for convenience, there are some instructions that rely on functions placed at the beginning of the memory space (recall the `RST` instructions for example). It should be noted that this memory space is technically read-only, aside from when banks are being switched out. It would be a massive headache to keep track of which data is unmodified from the ROM and which is dirty, so the option is completely removed. Attempting to write in that block of address does have a function, however; it's the mechanism used to signal which bank we want swapped in, a process we'll study more later.

From 0x8000 to 0x9FFF is 8 KiB of video memory. This space is utilized by the PPU ("Pixel Processing Unit", sometimes also called the "Video Processing Unit") when it goes to render the screen. It's broken down into its own subsections as well, which we'll explore when we reach that point. Addresses 0xA000 to 0xBFFF are reserved for any optional RAM the developers might have shipped along with the physical game cartridge. Addresses 0xC000 to 0xFFFF serves a variety of purposes, such as general use work RAM, sprite information, I/O port data, etc.

We'll begin by creating a simple array to store our memory data, and as we begin to add specialized behavior, we might move chunks out of the array to their own locations, depending on the use case. For now, let's finally create a new file, still inside our `core` directory, called `bus.rs`. We'll also need to update `lib.rs` to let Rust know this new file exists. For those unfamiliar with the term, a "bus" in computing is a system that transfers data from one component to another, such as between the CPU, main memory, the PPU, etc.

```rust
// In lib.rs

pub mod bus;
pub mod cpu;
pub mod utils;
```

Inside of `bus.rs`, we'll create a new `Bus` object which for now will only hold a simple `u8` array serving as our RAM. We'll also create a constructor which initializes the data to 0.

```rust
// In bus.rs

pub struct Bus {
    ram: [u8; 0x10000],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            ram: [0; 0x10000],
        }
    }
}
```

Next, we'll create some functions inside `bus.rs` to handle the reading and writing to RAM. For now, these are trivial functions, but later as we compartmentalize the different sections of RAM, we will need to ensure that the different ranges get passed to the right places. For now though, we'll add a `write_ram` and `read_ram` function.

```rust
// In bus.rs

impl Bus {
    // Unchanged code omitted

    pub fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        self.ram[addr as usize] = val;
    }
}
```

We can now finish the initial set up by finally connecting this new `Bus` object to our `Cpu`. If you recall our architecture diagram, the CPU will need access to the bus, which in turn needs access to the other submodules. To create this structure, this new `Bus` struct will be created as a member variable of the `Cpu`. With it, we can modify the CPU's RAM access function to utilize those on the Bus.

```rust
// In cpu/mod.rs

use crate::bus::Bus;

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
}

impl Bus {
    // Unchanged code omitted

    pub fn read_ram(&self, addr: u16) -> u8 {
        self.bus.read_ram(addr)
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        self.bus.write_ram(addr, val);
    }
}
```

With this change, we finally have a working CPU with RAM access! The problem now is that the RAM is currently full of zeroes, which doesn't make for very interesting execution. Our next step will be to read a game ROM file in and store it in our emulator. To do that, we need to take a step back and finally set up our frontends so that they can read in a file and pass it to the `core`.

[*Next Chapter*](13-desktop-setup.md)
