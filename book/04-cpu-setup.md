# Chapter IV. CPU Setup

[*Return to Index*](../README.md)

[*Previous Chapter*](03-project-setup.md)

Our first steps towards emulation will be to implement the CPU. Since the CPU has various subpieces of its own, we'll create a Rust module for CPU-related files to be grouped together. Inside of `core/src`, create a new folder called `cpu` with a `mod.rs` file inside of it. Let's also create a `utils.rs` file to hold any helper functions that might be needed in multiple places.

```
.
├── core
│   ├── Cargo.toml
│   └── src
│       ├── lib.rs
│       ├── utils.rs
│       └── cpu
│           └── mod.rs
├── desktop
│   ├── Cargo.toml
│   └── src
│       └── main.rs
└── wasm
    ├── Cargo.toml
    └── src
        └── lib.rs
```

If you're new to Rust, anytime you add a new file you need to inform the complier of their existence. In `core/src/lib.rs`, delete all the automatically added code, and instead replace it with the lines below.

```rust
// In core/src/lib.rs

pub mod cpu;
pub mod utils;
```

This tells the compiler that inside of the `core`, there are two new modules called `cpu` and `utils`. The compiler will automatically check `cpu/mod.rs` for further details about the `cpu` submodule. If you're a more advanced Rust user, feel free to reorganize the hierarchy of this project to better suit your style. I'm going to utilize a somewhat flat structure to keep things focused on the emulation development with as few distractions as possible.

With the structure in place, we can begin implementing the CPU. In `cpu/mod.rs`, we'll begin by creating a `Cpu` class which will encapsulate the emulated behavior of a CPU as we've described.

```rust
// In cpu/mod.rs

pub struct Cpu {
}
```

This begs the question, what do we put in our `Cpu`? As mentioned in the [CPU introduction](03-cpu-specs.md) chapter, the CPU functions entirely by accessing its registers  -- the eight 8-bit ones, as well as the 16-bit "SP" and "PC" -- so let's begin by adding those. We'll also add a basic constructor function to initialize these values. Note that these initial values for each of the registers are placeholders, we'll go over how they should be correctly initialized later.

```rust
// In cpu/mod.rs

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
}

impl Cpu {
    pub fn new() -> Self {
        pc: 0x0000,
        sp: 0x0000,
        a: 0x00,
        b: 0x00,
        c: 0x00,
        d: 0x00,
        e: 0x00,
        f: 0x00,
        h: 0x00,
        l: 0x00,
    }
}
```

Each of the ten different registers receives their own member variable inside the `Cpu` class. The 16-bit ones -- PC and SP -- are stored as `u16` while the 8-bit ones are likewise stored as `u8`.<sup>1</sup>

<sup>1</sup> I've seen a few different methods for implementing the 8-bit registers. Some projects will store them as eight 8-bit values, some as four 16-bit values, as four unions between 8-bit and 16-bit, and some have even created specific objects for the four register pairs. In the end, most of the register access will be done via helper functions anyway, so you can structure any way you like, so long as you're consistent. For this project, we shall store them as 8-bit values and create helper functions to concatenate them into a 16-bit pair when needed.

## CPU Enums

Next, we'll define some enumerations with elements corresponding to each of the registers. While the Game Boy CPU is capable of several hundred different instructions, there is a great deal of overlap in their behavior, as we'll see. An instruction operating on the B register is distinct to the one operating on the C register, even though their implementation is virtually identical. These enum values will allow us to create helper functions for different categories of instruction, rather than having to create a distinct one for every opcode.

```rust
// In cpu/mod.rs
// Unchanged code omitted

#[derive(Copy, Clone)]
pub enum Regs {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Copy, Clone)]
pub enum Regs16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}
```

Since some operations want the 8-bit versions and others the 16-bit pairs, we'll create enum values for every combination, which will be used in different versions of the functions. This includes the Stack Pointer, as while it isn't one of the paired registers, it can still be read and written to, and it will make some functions cleaner down the road by having it as an option. I'll also save you the trouble and tell you that these will need the `Copy` and `Clone` traits down the line.

While we're at it, lets also make an enum for the flag values.

```rust
// In cpu/mod.rs
// Unchanged code omitted

pub enum Flags {
    Z,
    N,
    H,
    C,
}
```

## CPU Access Helper Functions

To begin, let's define getters and setters for the 8-bit register enums. These two function will go within the `impl Cpu` block, as they are functions for our `Cpu` class.

```rust
// In cpu/mod.rs
// Unchanged code omitted

impl Cpu {
    pub fn get_r8(&self, r: Regs) -> u8 {
        match r {
            Regs8::A => { self.a },
            Regs8::B => { self.b },
            Regs8::C => { self.c },
            Regs8::D => { self.d },
            Regs8::E => { self.e },
            Regs8::F => { self.f },
            Regs8::H => { self.h },
            Regs8::L => { self.l },
        }
    }

    pub fn set_r8(&mut self, r: Regs, val: u8) {
        match r {
            Regs8::A => { self.a = val },
            Regs8::B => { self.b = val },
            Regs8::C => { self.c = val },
            Regs8::D => { self.d = val },
            Regs8::E => { self.e = val },
            // Note: The bottom four bits of F shall always be 0
            Regs8::F => { self.f = val & 0xF0 },
            Regs8::H => { self.h = val },
            Regs8::L => { self.l = val },
        }
    }
}
```

These two functions will allow us to get and set any of the 8-bit registers by passing it its corresponding enum value. The implementation here is pretty straight-forward. The only non-obvious item is that since the four flags are only defined for the highest bits, the lower four bits of the F register default to zero. By using an AND operation with 0xF0, the lower four bits will always be zero while the upper four will remain as they are. This method is sometimes referred to as "masking", and it's a technique we'll see again.

We'll now create similar functions for the 16-bit registers. This will require us to append some of our `u8` values end-to-end to create a single `u16` -- for example combining the B and C values to create the 16-bit BC value. Merging two 8-bit values is an operation we'll be doing fairly often, so let's create a helper function to do so. We'll store this in our `utils.rs` file, so that it's available to other files besides `cpu/mod.rs`.

Inside `utils.rs`, we'll begin with a function to create a `u16` from two `u8` values. This function will take two bytes, one `high` and one `low`. Our first step is to cast both to a `u16`, as not only is it the return type, but our shifting operations would overflow if tried on an 8-bit value. The low byte can remain where it is, but the high byte needs to be shifted over eight bits, making room for the low byte to fit next to it. Finally, the two are combined with an OR operation. Similar to our masking technique earlier, by using an OR operation, the empty bits of high and low will be filled with whatever value is in their counterpart.<sup>1</sup>

```rust
// In utils.rs

pub fn merge_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}
```

Next, we'll want to do the opposite operation, taking a `u16` value and extracting either the high or low `u8` from it. Here is one of the few examples where this guide will use a Rust-specific feature, although this could also just as easily be done with a regular function. This will be such a common operation that we'll define a new Trait for `u16` to fetch both the low and high bytes. This is done first by defining the trait and the functions within, then implementing them for `u16`. We'll store these in `utils.rs` as well.

```rust
// In utils.rs
// Unchanged code omitted

pub trait ByteOps {
    fn high_byte(&self) -> u8;
    fn low_byte(&self) -> u8;
}

impl ByteOps for u16 {
    fn high_byte(&self) -> u8 {
        (self >> 8) as u8
    }

    fn low_byte(&self) -> u8 {
        (self & 0xFF) as u8
    }
}
```

Back in `cpu/mod.rs`, we can use these functions by including `utils` at the top of the file, and then define our 16-bit getter and setter inside the `impl Cpu` block of `cpu/mod.rs`.

```rust
// In cpu/mod.rs
// Unchanged code omitted

use crate::utils::*;

impl Cpu {
    pub fn get_r16(&self, r: Regs16) -> u16 {
        match r {
            Regs16::AF => { merge_bytes(self.a, self.f) },
            Regs16::BC => { merge_bytes(self.b, self.c) },
            Regs16::DE => { merge_bytes(self.d, self.e) },
            Regs16::HL => { merge_bytes(self.h, self.l) },
            Regs16::SP => { self.sp },
        }
    }

    pub fn set_r16(&mut self, r: Regs16, val: u16) {
        let high = val.high_byte();
        let low = val.low_byte();
        match r {
            Regs16::AF => {
                self.set_r8(Regs::A, high);
                self.set_r8(Regs::F, low);
            },
            Regs16::BC => {
                self.set_r8(Regs::B, high);
                self.set_r8(Regs::C, low);
            },
            Regs16::DE => {
                self.set_r8(Regs::D, high);
                self.set_r8(Regs::E, low);
            },
            Regs16::HL => {
                self.set_r8(Regs::H, high);
                self.set_r8(Regs::L, low);
            },
            Regs16::SP => { self.sp = val; },
        }
    }
}
```

These are a bit more complicated than the 8-bit versions. For `get_r16`, we'll use a match statement to check the enum value passed into the function. Then, depending on the value, we'll merge together the corresponding bytes to create a single `u16` to return. In the Stack Pointer's case, we can just return `self.sp` outright, as it's a 16-bit value. For `set_r16`, the opposite behavior occurs. Rather than returning a `u16` value, we need to be given one along with the identifying enum value. We'll then break the `u16` into its two bytes and store them into the correct register variable.

<sup>1</sup> For the record, using addition would've worked perfectly fine as well.

<sup>2</sup> If you're following this book but not using Rust, simple functions would work as well.

### Flag Helpers

For the final part of this chapter, we will create setters and getters for the flags. Although the above functions do write to the F register, there will be many operations that need to deal with a single flag at a time. We'll cover what each of the flags actually do in due time; for now we'll just need to have the ability to modify any of them if we need. Unlike the getters and setters for the registers, it's more useful for us to know if a flag is set or not -- in that case we'll accept a flag enum as an input and return a boolean as output.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn get_flag(&self, f: Flags) -> bool {
        match f {
            Flags::Z => { (self.f & 0b1000_0000) != 0 },
            Flags::N => { (self.f & 0b0100_0000) != 0 },
            Flags::H => { (self.f & 0b0010_0000) != 0 },
            Flags::C => { (self.f & 0b0001_0000) != 0 },
        }
    }

    pub fn set_flag(&mut self, f: Flags, val: bool) {
        if val {
            match f {
                Flags::Z => { self.f |= 0b1000_0000 },
                Flags::N => { self.f |= 0b0100_0000 },
                Flags::H => { self.f |= 0b0010_0000 },
                Flags::C => { self.f |= 0b0001_0000 },
            }
        } else {
            match f {
                Flags::Z => { self.f &= 0b0111_0000 },
                Flags::N => { self.f &= 0b1011_0000 },
                Flags::H => { self.f &= 0b1101_0000 },
                Flags::C => { self.f &= 0b1110_0000 },
            }
        }
    }
}
```

These functions again utilize the masking technique we saw earlier to extract only a single bit at a time. Inside the Flag register, the 7th bit<sup>1</sup> always holds the value of the Z flag, the 6th bit holds the N flag, then H and C accordingly, with the lower four bits unused. Our `get_flag` function uses this fact to AND the flag register with a byte comprised only of the bit we want to extract. The final step is then to see if this value is non-zero, which will convert to our output boolean.

The `set_flag` function accepts a flag enum and the boolean value we want to set our flag to. If we want the flag to be set -- so a 1 should be stored at the corresponding bit -- we'll create a byte with only that bit set and OR it to our flag register, like we did before. This will leave most of the bits as they were, and guarantee the desired bit is set to 1. If we need to clear the flag, we'll utilize almost the opposite technique. We'll create a byte with the other bits set to 1, and then AND that with our flag register. The bits AND'ed with 1 will remain the same, while the bit we want to clear will be so.

Perhaps surprisingly, this is all the setup we need to begin defining all the different CPU operations. Of course, we'll likely create additional helper functions, but it's a solid enough foundation to get started.

<sup>1</sup> When bit numbers are given, it's always counted from the right.

[*Next Chapter*](05-opcode-setup.md)
