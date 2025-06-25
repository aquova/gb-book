# XXIX. Work, Echo, and High RAM

[*Return to Index*](../README.md)

[*Previous Chapter*](28-sprite-layer.md)

We've been defining more and more of the RAM map the past few chapters, while managing to make a mess of our `Bus` class. It's time to clean that up and correctly establish the structure we will use going forward.

![](img/memory-map.svg)

If we reference our RAM map once again, a good amount of it has been implemented and utilized. The first half (0x0000 through 0x7FFF) is for the cartridge ROM, and indeed if the system needs to read or write to those addresses, we forward that request on to our `Cart` object. 0x8000 through 0x9FFF is for VRAM, handled by the PPU. Here though, things begin to go astray. The next address we correctly handle is all the way over at 0xFE00, which is the beginning of the OAM. This means that the cartridge RAM, work RAM and its echo, and high RAM, are currently just handled by that `ram` array within the `Bus`. Even if this array didn't have a number of "holes" in it, it would still be advantageous to split it out into its respective categories, as we have done for the other sections. We'll save the cartridge RAM handling for the next chapters, and instead begin with the work RAM.

## WRAM

Work RAM ("WRAM") is a block of memory that, unlike many of the others, is meant for the developer to use as they please to perform calculations. It may be surprising to learn that despite the Game Boy having 64 KiB of RAM, very little of it is for general usage, in the way that we typically think of it. Instead, it's assigned specific roles and only a small amount is unencumbered. In addition, while addresses 0xC000 through 0xDFFF are for WRAM, the following section -- from 0xE000 to 0xFDFF -- is an exact copy of that data, known as "Echo RAM". This means that if you read or write to address 0xC000, the exact same operation happens to 0xE000. Accessing 0xC001 does the same to 0xE001, and so on. Echo RAM is actually slightly smaller than WRAM, so 0xDE00 through 0xDFFF don't have an Echo counterpart. It might seem odd, but it's another quirk of the Game Boy that we must support.

To do so, I'm going to create a new file to hold the WRAM and Echo data, called `wram.rs`. You arguably could accomplish this with some arrays within `bus.rs`, but I think this is a bit cleaner. As an aside, the Game Boy Color gives some extra functionality to this region of memory, so if you're planning on expanding beyond the original Game Boy, this is a bit of future proofing.

```rust
// In wram.rs
// Unchanged code omitted

impl WRAM {
    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            WRAM_START..=WRAM_STOP => {
                let relative_addr = addr - WRAM_START;
                self.wram[relative_addr as usize]
            },
            ECHO_START..=ECHO_STOP => {
                let relative_addr = addr - ECHO_START;
                self.wram[relative_addr as usize]
            },
            _ => { unreachable!() }
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            WRAM_START..=WRAM_STOP => {
                let relative_addr = addr - WRAM_START;
                self.wram[relative_addr as usize] = val;
            },
            ECHO_START..=ECHO_STOP => {
                let relative_addr = addr - ECHO_START;
                self.wram[relative_addr as usize] = val;
            },
            _ => { unreachable!() }
        }
    }
}

```

When accessing either block of memory, the respective address start is subtracted to get us the correct array index, then the operation is performed. This is rounded out with an `unreachable!` statement, as Rust requires all values to be accounted for in a match statement.

The last step is to return to `bus.rs` and assign this new object to a member variable and to plug the read and write functions into our existing structure.

```rust
// In bus.rs
// Unchanged code omitted

use crate::wram::{WRAM, ECHO_STOP, WRAM_START};

pub struct Bus {
    rom: Cart,
    ppu: Ppu,
    io: IO,
    wram: WRAM,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Cart::new(),
            ppu: Ppu::new(),
            io: IO::new(),
            wram: WRAM::new(),
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
            WRAM_START..=ECHO_STOP => {
                self.wram.read_u8(addr)
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
                0
            }
        }
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.write_cart(addr, val);
            },
            VRAM_START..=VRAM_STOP => {
                self.ppu.write_vram(addr, val);
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
            _ => {}
        }
    }
```

You'll notice I did remove the old `ram` variable, meaning that the remaining unsupported memory addresses aren't going to be handled correctly as all now. Attempting to read from them will always return 0, and writing will do nothing. This is a temporary measure, we'll need to remedy this for full functionality.

## High RAM

We now have Cartridge ROM, VRAM, WRAM & Echo RAM, the OAM data, I/O, and PPU registers officially accounted for. If you cross-reference the RAM map, you should see that this leaves four sections unsupported.

| Range | Usage |
| ----- | ----- |
| 0xA000-0xBFFF | Cartridge RAM (we'll handle this as we expand our `Cart` class, coming up) |
| 0xFEA0-0xFEFF | Unused |
| 0xFF4C-0xFF7F | Unused |
| 0xFF80-0xFFFF | High RAM |

The two empty blocks do exactly as we need them to do right now, just return a constant value on reads and do nothing on writes. As stated, we'll handle the cartridge RAM in the upcoming chapters, so this only leaves the High RAM ("HRAM") remaining. Named because it's at the high end of the memory addresses, HRAM is also somewhat general purpose, although you'll notice that it actually overlaps some other defined memory addresses, most notably the stack. This is by design, and it's up to the developer to make sure they aren't corrupting their own stack upon utilizing HRAM. From an emulation standpoint, we need to support it like any other RAM value.

For this, I'm not going to create a new class (although you can if you want), I'm just going to handle it as an array within the `Bus`.

```rust
// In bus.rs
// Unchanged code omitted

const HRAM_START: u16   = 0xFF80;
const HRAM_STOP: u16    = 0xFFFF;
const HRAM_SIZE: usize  = (HRAM_STOP - HRAM_START + 1) as usize;

pub struct Bus {
    rom: Cart,
    ppu: Ppu,
    io: IO,
    wram: WRAM,
    hram: [u8; HRAM_SIZE],
}

impl Bus {
    pub fn new() -> Self {
        Self {
            rom: Cart::new(),
            ppu: Ppu::new(),
            io: IO::new(),
            wram: WRAM::new(),
            hram: [0; HRAM_SIZE],
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
            WRAM_START..=ECHO_STOP => {
                self.wram.read_u8(addr)
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
            HRAM_START..=HRAM_STOP => {
                let relative_addr = addr - HRAM_START;
                self.hram[relative_addr as usize]
            },
            _ => {
                0
            }
        }
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        match addr {
            ROM_START..=ROM_STOP => {
                self.rom.write_cart(addr, val);
            },
            VRAM_START..=VRAM_STOP => {
                self.ppu.write_vram(addr, val);
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
    }
}
```

With the exception of the Cartridge RAM, this completes our RAM map. There are still some addresses that serve a specialized purpose, but those values are hooked up to be read and written to, it's simply up to us to interpret that data correctly.

[*Next Chapter*](30-cpu-timer.md)
