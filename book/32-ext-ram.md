# XXXII. External RAM

[*Return to Index*](../README.md)

[*Previous Chapter*](31-header.md)

Our memory map, as defined in `bus.rs`, has one large remaining hole in it. We've discussed it previously, but the address space from 0xA000 to 0xBFFF is designated for mapping in RAM provided on the cartridge, if any exists. This will look pretty similar to how we've been handling the ROM data so far. We will expose read and write functions to the Bus, and since it is possible to have more external RAM than will fit in the address space, bank switching will need to be supported (more on that later). Just as the ROM vector is initialized when the game is loaded, we will also need to initialize a RAM vector that can store the game's needed external RAM.

## RAM Size Header Info

Just as games chose whether or not to include external RAM at all, the size of the RAM they did include was not always the same. There were a variety of common sizes that were chosen, with those larger than the 8 KiB of address space provided utilizing bank switching. While we could just define a RAM array that is the largest of these possible sizes, that feels a bit inefficient. Instead, we can turn again to the header to learn how much RAM the current game requires.

The RAM size is stored in the header at address 0x0149, and contains an index denoting the size of the RAM.

| Header Index | RAM Size (in KiB) |
| ------------ | ----------------- |
| 0            | 0                 |
| 1            | 2                 |
| 2            | 8                 |
| 3            | 32                |
| 4            | 128               |
| 5            | 64                |

Our next step is to create a new `ram` vector member variable, and when we are loading the game ROM, we will also need to initialize `ram`. This will require reading that header index, and initialize the vector to the right length. This is really only required as we'll get read/write errors later if the vector isn't the right size. We could use a data structure that handles this issue, but for simplicity I will stick with the trusty vector.

```rust
// In cart/mod.rs
// Unchanged code omitted

const RAM_SIZES: [usize; 6] = [
    0,
    2,
    8,
    32,
    128,
    64
];

const RAM_SIZE_ADDR: usize = 0x0149;

pub struct Cart {
    rom: Vec<u8>,
    ram: Vec<u8>,
    mbc: MBC,
}

impl Cart {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            ram: Vec::new(),
            mbc: MBC::NONE,
        }
    }

    fn init_ext_ram(&mut self) {
        let mut ram_size_idx = self.rom[RAM_SIZE_ADDR] as usize;

        // Some headers don't report their external RAM capacity correctly
        if self.has_external_ram() && ram_size_idx == 0 {
            ram_size_idx = 1;
        }

        let ram_size = RAM_SIZES[ram_size_idx] * 1024;
        self.ram = vec![0; ram_size];
    }

    pub fn load_cart(&mut self, rom: &[u8]) {
        self.rom = rom.to_vec();
        self.mbc = self.get_mbc();
        self.init_ext_ram();
    }
}
```

While the core concept of the new function is straight-forward, there are some games that don't have the correct RAM capacity defined in their header. I'm unsure if there are any commercial games that have this mistake, but the Blargg test ROMs we've been using fall into this category. We call this new function after the ROM has been loaded in, as we require the header to be correctly populated first.

To conclude this chapter, we will add functions to access the new `ram` object, finally completing our memory map. By now, this process will be very familiar, we'll add `read_ram` and `write_ram` functions and hook them up to their appropriate places in the `Bus`.

```rust
// In cart/mod.rs
// Unchanged code omitted

pub const EXT_RAM_START: u16    = 0xA000;
pub const EXT_RAM_STOP: u16     = 0xBFFF;

impl Cart {
    pub fn read_ram(&self, addr: u16) -> u8 {
        let rel_addr = addr - EXT_RAM_START;
        self.ram[rel_addr as usize]
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        let rel_addr = addr - EXT_RAM_START;
        self.ram[rel_addr as usize] = val;
    }
}
```

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
            EXT_RAM_START..=EXT_RAM_STOP => {
                self.rom.read_ram(addr)
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
            EXT_RAM_START..=EXT_RAM_STOP => {
                self.rom.write_ram(addr, val);
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

This sets up the basic structure for the external RAM. For games with only a single RAM bank (or with no RAM at all), what we have here will be enough. For those who have more than 8 KiB, additional mechanisms will need to be added. We've been talking about memory banks for many chapters now, but in the next we will finally implement them.

[*Next Chapter*](33-mbc1.md)
