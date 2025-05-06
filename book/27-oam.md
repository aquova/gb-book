# Chapter XXVII. Object Attribute Memory

[*Return to Index*](../README.md)

[*Previous Chapter*](26-input.md)

With the background and window layers supported, there is only one remaining &mdash; the sprite layer. While some aspects of the sprites work in a similar fashion to the other two layers, such as using the same tiles, the free movement of the sprites requires other mechanics. The additional data needed for the sprites, such as their coordinate position, whether or not they're flipped, and which tile to use, is stored in an area known as *Object Attribute Memory*.

The OAM lies in RAM between 0xFE00 through 0xFE9F. This is 160 bytes which allows 40 sprites to have four bytes each of metadata. These are assigned the following roles.

| Byte | Purpose           |
| ---- | ----------------- |
| 0    | Sprite Y Position |
| 1    | Sprite X Position |
| 2    | Tile Index        |
| 3    | Flags             |

The first three bytes are pretty self-explanatory. Bytes 0 and 1 store the coordinate for the sprite, and byte 2 stores the tile index (much like the background tile map does). Byte 3 holds a variety of true/false flags for other sprite behavior. However, while all eight bits do have a purpose, only bits 4 through 7 are relevant for the original Game Boy, as the other four are only used by the Game Boy Color, which we won't cover.

| Bit | Purpose             |
| --- | ------------------- |
| 7   | Background Priority |
| 6   | Y Flip              |
| 5   | X Flip              |
| 4   | Sprite Palette      |
| 0-3 | Reserved for GBC    |


Bit 4 signals which of the two different palettes should be used when rendering the sprite. While the all background and window tiles only had the one palette to use, sprites have a choice between two. Bits 5 and 6 determine if the sprite should be flipped horizontally and/or vertically, respectively. This allows for sprites to change direction with a single flipped bit, rather than having to rewrite several bytes of pixel information. Bit 7 is an interesting feature where, if set, the sprite is drawn *behind* the window and background layers. For a formal definition, this means that a sprite pixel is written to the framebuffer only if the background/window value is index 0.

There are a few different ways to implement this, but we're going to do something similar to what we did in `tile.rs` and create a new structure to hold all the sprite metadata. We'll again add functions which handling reading and writing of the raw data as needed, but also hold the data in a more structured format. To begin, we'll create a new `ppu/sprite.rs` file, and define a `Sprite` struct inside it.

```rust
// In ppu/sprite.rs

use crate::utils::*;

#[derive(Clone, Copy)]
pub struct Sprite {
    coords: Point,
    tile_num: u8,
    bg_priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette1: bool,
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            coords: Point::new(0, 0),
            tile_num: 0,
            bg_priority: false,
            x_flip: false,
            y_flip: false,
            palette1: false,
        }
    }
}
```

This class isn't too difficult to implement, with the only noteworthy functions being the reading and writing of the raw data. The address will be used (in `ppu/mod.rs`) to determine which `Sprite` object we're targeting, but it will be used again to determine which of the four bytes is being edited. We can simply use a modulo operator here, then set the input value accordingly. To keep things more readable, I'm also going to define some constants for each of the different bit values, much like we did when we implemented the LCDC.

```rust
// In ppu/sprite.rs
// Unchanged code omitted

const BG_PRIORITY_BIT: u8   = 7;
const Y_FLIP_BIT: u8        = 6;
const X_FLIP_BIT: u8        = 5;
const PALETTE_BIT: u8       = 4;

impl Sprite {
    pub fn read_u8(&self, addr: u16) -> u8 {
        let offset = addr % 4;
        match offset {
            0 => {
                self.coords.y
            },
            1 => {
                self.coords.x
            },
            2 => {
                self.tile_num
            },
            3 => {
                let mut ret = 0;
                ret.set_bit(BG_PRIORITY_BIT, self.bg_priority);
                ret.set_bit(Y_FLIP_BIT, self.y_flip);
                ret.set_bit(X_FLIP_BIT, self.x_flip);
                ret.set_bit(PALETTE_BIT, self.palette1);
                ret
            }
            _ => { unreachable!() }
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        let offset = addr % 4;
        match offset {
            0 => {
                self.coords.y = val;
            },
            1 => {
                self.coords.x = val;
            },
            2 => {
                self.tile_num = val;
            },
            3 => {
                self.bg_priority = val.get_bit(BG_PRIORITY_BIT);
                self.y_flip = val.get_bit(Y_FLIP_BIT);
                self.x_flip = val.get_bit(X_FLIP_BIT);
                self.palette1 = val.get_bit(PALETTE_BIT);
            },
            _ => { unreachable!(); }
        }
    }
}
```

As stated above, byte 0 corresponds to the Y coordinate, byte 1 to the X coordinate, and byte 2 to the tile index. Byte 3 has three assigned bits for an original Game Boy, and we will completely ignore the others. Finally, we'll add some getter functions to access each of the stored fields. We don't need any setter functions since that will always happen in `write_u8`.

```rust
// In ppu/sprite.rs
// Unchanged code omitted

const Y_OFFSET: isize = 16;
const X_OFFSET: isize = 8;

impl Sprite {
    pub fn get_bg_priority(&self) -> bool {
        self.bg_priority
    }

    pub fn get_coords(&self) -> (isize, isize) {
        (self.pos.x as isize - X_OFFSET, self.pos.y as isize - Y_OFFSET)
    }

    pub fn get_tile_num(&self) -> u8 {
        self.tile_num
    }

    pub fn is_x_flipped(&self) -> bool {
        self.x_flip
    }

    pub fn is_y_flipped(&self) -> bool {
        self.y_flip
    }

    pub fn use_palette1(&self) -> bool {
        self.palette1
    }
}
```

Most of these are pretty obvious, with the exception of `get_coords`. Rather than returning one of our normal `Point` values (which are two `u8`s), I returned a tuple of `isize`. Recall that the coordinates actually stored in the OAM are offset from where they are drawn on screen. These are the coordinates we actually want, but they have the potential to be negative. We could alternatively store the actual screen coordinates, but we would need to perform the same adjustment when reading or writing the 8-bit value. I personally think this is slightly cleaner.

With the `Sprite` object defined, we can return to `ppu/mod.rs` and utilize it. As stated, this section of memory is large enough to hold 40 sprites worth of metadata, which we will define as constants. We'll add an `oam` field to our `Ppu` object which will hold 40 of our `Sprite` objects. Since this is a new block of RAM, we'll add functions similarly to how we implemented reading/writing to VRAM, which can then plug into the bus.

```rust
// In ppu/mod.rs
// Unchanged code omitted

mod sprite;

use sprite::Sprite;

pub const OAM_START: u16 = 0xFE00;
pub const OAM_STOP: u16  = 0xFE9F;

const NUM_OAM_SPRITES: usize = 40;
const BYTES_PER_SPRITE: u16  = 4;

pub struct Ppu {
    mode: Lcd,
    tiles: [Tile; NUM_TILES],
    maps: [u8; TILE_MAP_SIZE],
    lcd_regs: [u8; LCD_REG_SIZE],
    oam: [Sprite; NUM_OAM_SPRITES],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            mode: Lcd::new(),
            tiles: [Tile::new(); NUM_TILES],
            maps: [0; TILE_MAP_SIZE],
            lcd_regs: [0; LCD_REG_SIZE],
            oam: [Sprite::new(); NUM_OAM_SPRITES],
        }
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let relative_addr = addr - OAM_START;
        let oam_idx = relative_addr / BYTES_PER_SPRITE;
        self.oam[oam_idx as usize].read_u8(addr)
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        let relative_addr = addr - OAM_START;
        let oam_idx = relative_addr / BYTES_PER_SPRITE;
        self.oam[oam_idx as usize].write_u8(addr, val);
    }
}
```

All that remains is to plug `read_oam` and `write_oam` into the bus's read and write functions so that they can be accessed correctly during game execution.

```rust
// In bus.rs
// Unchanged code omitted

impl Bus {
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
            JOYPAD_ADDR => {
                self.joypad.read_u8()
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
            JOYPAD_ADDR => {
                self.joypad.write_u8(val);
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

We really are making a right mess with our `ram` array. Not only is the PPU Register section going unused, but the OAM region is now as well. Eventually we'll try and tidy this up as best we can, but for now I'm afraid we're going to leave it as is, as it's more trouble than it's worth to restructure around it.

## OAM DMA Transfer

While the Game Boy is free to copy information to the OAM using ordinary writes, it actually has its own, additional way to be updated. OAM DMA (Direct Memory Access) Transfer is a process where the system will copy a block of memory from a user-defined location into OAM. If you recall, when we went over the PPU registers there was one address we skipped over, 0xFF46. We return to it now, as this is the register that, when written to, kicks off the DMA transfer. The value written to it specifies the high byte of the source address of the memory block to copy into OAM. For example, if 0x12 is written to 0xFF46, then the OAM DMA transfer begins, copying the 160 bytes from 0x1200-0x129F into 0xFE00-0xFE9F. While this sounds like a complicated procedure, it's actually pretty straight-forward to implement.

```rust
// In bus.rs
// Unchanged code omitted

const OAM_DMA: u16 = 0xFF46;

impl Bus {
    fn dma_transfer(&mut self, val: u8) {
        let src = (val as u16) << 8;
        for i in 0..0xA0 {
            let val = self.read_ram(src + i);
            self.write_ram(OAM_START + i, val);
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
            OAM_START..=OAM_STOP => {
                self.ppu.write_oam(addr, val);
            },
            JOYPAD_ADDR => {
                self.joypad.write_u8(val);
            },
            LCD_REG_START..=LCD_REG_STOP => {
                if addr == OAM_DMA {
                    self.dma_transfer(val);
                }
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

Since `OAM_DMA` is within the LCD register range, we can place it within that block, and when written to will kick off the transfer process. The transfer uses the byte written as the high byte and directly copies that memory over.

With this in place, the emulator now has all the necessary tools to populate the sprite metadata table; all that remains is to actually render the sprite layer.

[*Next Chapter*](27-oam.md)
