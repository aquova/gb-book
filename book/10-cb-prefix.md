# X. The 0xCB Prefix Table

Time to answer a big mystery. In the [opcode reference page](https://izik1.github.io/gbops/), there is a second table below the one we've been referencing. What is that second opcode table and how is it used? If the CPU was only limited to a single byte as an index, that would limit the number of possible CPU operations to be 256. This might be fine for some architectures, but the designers of this CPU desired a greater number, and thus added support for a second entire table of 256 more instructions. To switch over to it, they added a special instruction, with index 0xCB, appropriately named `PREFIX CB`. This instruction doesn't do anything by itself, but instead tells the CPU to read another byte, and use that one as the opcode index for the other table. At this point, the CPU will repeat the whole process, but instead using the second table. This means it will fetch another byte, treat that as an index, look at what instruction in the CB table that maps to, and perform its execution accordingly. And yes, this does mean that there are 256 more instructions to explain and implement.

![Diagram showing the second instruction table](img/10-cb-table.png)

The second, 0xCB opcode table. Don't be discouraged!

Fortunately, the CB table is very repetitive, with many of its instructions only slightly varying from each other. One fourth of the instructions are `BIT` instructions, where a flag is set if the given bit in the given register is true or not. Another fourth of the instructions are for `SET`, which sets a specified bit to be 1; and another fourth of the instructions are for `RES`, which does the same thing as `SET` except sets the bit to be 0. That covers 75% of the CB instructions off the bat.

The final quarter handle other bitwise operations that we haven't discussed. `RLC`, `RRC`, `RL`, and `RR` are "rotate" instructions, where all the bits shift over one spot *and* the bit that would be cut off circles around to the other side. For example, if you have 11110000 and rotate left, you would get 11100001 as the 1 on the left circles around to the right. `RLC` and `RL` rotate to the left, while `RRC` and `RR` rotate to the right. The "C" at the end of `RLC` and `RRC` is for the carry flag. In those instructions, rather than directly circling the dropped bit around, that bit is stored in the carry flag, and the old carry flag value is what is circled in, almost as if it was one of the bits itself. The `RL` and `RR` instructions circle around like normal.

Similarly to the rotate instructions are the shift instructions; `SLA`, `SRA`, and `SRL`, which are left shift, arithmetic right shift, and logical right shift. These functions are covered in [Chapter I](01-refresher.md), and they operate in the typical fashion here. Finally, there are the `SWAP` instructions, which swaps the top four bits of the 8-bit value with the bottom four bits.

We'll begin by implementing the `PREFIX CB` function, which will be the gateway to these new operations.

```rust
// In cpu/opcodes.rs

fn prefix_cb(cpu: &mut Cpu) -> u8 {
    let cb_index = cpu.fetch();
    execute_cb(cpu, cb_index)
}
```

This is similar to what our original `execute` function does. It fetches the value pointed at by the PC, and uses that to determine the next instruction. Our `execute` function utilizes the `OPCODES` function array to find the corresponding instruction's function. We could do the same thing for the 0xCB table and add a second lookup table with 256 accompanying functions, but that's a bit overkill for this table. We chose to use a lookup table as while the normal opcode table has some common patterns, there's enough variation from those patterns to make the table worth it. Here though, the 0xCB very strongly sticks to its patterns, so we can avoid all the boilerplate and decode the opcode's meaning a little more intelligently.

Let's begin by identifying these patterns. If you look at the 0xCB table, you'll notice that each type of instruction comes in blocks together like the following.

| Instruction Type | Range     |
| ---------------- | --------- |
| RLC              | 0x00-0x07 |
| RRC              | 0x08-0x0F |
| RL               | 0x10-0x17 |
| RR               | 0x18-0x1F |
| SLA              | 0x20-0x27 |
| SRA              | 0x28-0x2F |
| SWAP             | 0x30-0x37 |
| SRL              | 0x38-0x3F |
| BIT              | 0x40-0x7F |
| RES              | 0x80-0xBF |
| SET              | 0xC0-0xFF |

We can conveniently use these ranges to decode which instruction -- and thus which helper function -- we need to use. It doesn't end here though, the columns of the table also follow a strict pattern. There are eight 8-bit register inputs (A, B, C, D, E, H, L and (HL)) that always appear in the same order in the table, meaning that by looking at just the last three bits, we can determine what the input of the instruction will be. In fact, let's start with that function; where we take the opcode index and extract the register in question.

```rust
// In cpu/opcodes.rs

fn get_cb_reg(op: u8) -> Regs {
    match op & 0b111 {
        0 => { Regs::B },
        1 => { Regs::C },
        2 => { Regs::D },
        3 => { Regs::E },
        4 => { Regs::HL },
        5 => { Regs::H },
        6 => { Regs::L },
        7 => { Regs::A },
        _ => unreachable!()
    }
}

fn execute_cb(cpu: &mut Cpu, op: u8) -> u8 {
    // 0x00-0x07 -> RLC
    // 0x08-0x0F -> RRC
    // 0x10-0x17 -> RL
    // 0x18-0x1F -> RR
    // 0x20-0x27 -> SLA
    // 0x28-0x2F -> SRA
    // 0x30-0x37 -> SWAP
    // 0x38-0x3F -> SRL
    // 0x40-0x7F -> BIT
    // 0x80-0xBF -> RES
    // 0xC0-0xFF -> SET

    let cb_reg = get_cb_reg(op);
    match op {
        0x00..=0x07 => { unimplemented!(); },
        0x08..=0x0F => { unimplemented!(); },
        0x10..=0x17 => { unimplemented!(); },
        0x18..=0x1F => { unimplemented!(); },
        0x20..=0x27 => { unimplemented!(); },
        0x28..=0x2F => { unimplemented!(); },
        0x30..=0x37 => { unimplemented!(); },
        0x38..=0x3F => { unimplemented!(); },
        0x40..=0x7F => { unimplemented!(); },
        0x80..=0xBF => { unimplemented!(); },
        0xC0..=0xFF => { unimplemented!(); },
    }
    2
}
```

Much like what we've done previously, each of these instruction types should have its own helper function. Some of them, such as `RLC` and `RL`, and `RRC` and `RR`, will be nearly identical except for the usage of the carry flag (much like `ADD` and `ADC`). For those, we can have a single function with a boolean flag. Let's begin by looking at these instructions. As mentioned before, they are the "rotate" instructions, similar to left and right shifts, except the bit that "falls off" is wrapped and placed on the other end, with that value also being set as the carry flag in the two `C` instructions. Rust conveniently has `rotate_left` and `rotate_right` functions, but they don't actually tell us what the bit that circled around was. We will need to gather that for ourselves. In fact, the ability to grab the value specific bit will be used enough that it's worth adding functions to handle it.

The premise behind grabbing a single bit is simple. We'll pass in which bit index is needed (typical convention is to count from right to left, starting at 0), and a mask will be generated to retrieve that value. We will eventually want to have this functionality for both `u8` and `u16` values, so we shall create another new trait.

Annoyingly though, the process for changing or retrieving a single bit is virtually identical for a `u8` and a `u16`. Rather than define nearly the exact same functions for both, I'm going to utilize a Rust macro to do this for us. We'll write the single function once, marking the places where the two implementations differ, then allow the macro to do the heavy lifting.

```rust
// In utils.rs

pub trait BitOps {
    fn get_bit(&self, bit: u8) -> bool;
    fn set_bit(&mut self, bit: u8, set: bool);
}

macro_rules! impl_bitops {
    ($T:ty) => {
        impl BitOps for $T {
            fn get_bit(&self, bit: u8) -> bool {
                let mask = 0b1 << bit;
                (self & mask) != 0
            }

            fn set_bit(&mut self, bit: u8, set: bool) {
                let mask = 0b1 << bit;
                if set {
                    *self |= mask;
                } else {
                    *self &= !mask;
                }
            }
        }
    }
}

impl_bitops!(u8);
impl_bitops!(u16);
```

Why write out everything twice when we can have a macro do it for us? This simple example is about the limit of my Rust macro knowledge, so you won't be seeing any more after this.

## 0xCB Instructions

### Rotation

With the functionality in place to get and set bits as we please, we are now ready to implement the rotation functions. Notice this flags for these commands, `Z00C`, which we will need to be mindful of.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn rotate_left(&mut self, reg: Regs, carry: bool) {
        let val = self.get_r8(reg);
        let msb = val.get_bit(7);
        let mut new = val.rotate_left(1);
        if carry {
            new.set_bit(0, self.get_flag(Flags::C));
        }
        self.set_r8(reg, new);
        self.set_flag(Flags::Z, new == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, msb);
    }

    pub fn rotate_right(&mut self, reg: Regs, carry: bool) {
        let val = self.get_r8(reg);
        let lsb = val.get_bit(0);
        let mut new = val.rotate_right(1);
        if carry {
            new.set_bit(7, self.get_flag(Flags::C));
        }
        self.set_r8(reg, new);
        self.set_flag(Flags::Z, new == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, lsb);
    }
}
```

Both functions are nearly identical. We save the bit that is about to be lost and then rotate the value over by one. If the `carry` parameter is true, then the value in the C flag is used as the new bit, otherwise we leave it as is. We then update the register and the flags accordingly.

These two functions will complete the first four instruction types.

```rust
// In cpu/opcodes.rs

fn execute_cb(cpu: &mut Cpu, op: u8) -> u8 {
    // 0x00-0x07 -> RLC
    // 0x08-0x0F -> RRC
    // 0x10-0x17 -> RL
    // 0x18-0x1F -> RR
    // 0x20-0x27 -> SLA
    // 0x28-0x2F -> SRA
    // 0x30-0x37 -> SWAP
    // 0x38-0x3F -> SRL
    // 0x40-0x7F -> BIT
    // 0x80-0xBF -> RES
    // 0xC0-0xFF -> SET

    let cb_reg = get_cb_reg(op);
    match op {
        0x00..=0x07 => { cpu.rotate_left(cb_reg, true); },
        0x08..=0x0F => { cpu.rotate_right(cb_reg, true); },
        0x10..=0x17 => { cpu.rotate_left(cb_reg, false); },
        0x18..=0x1F => { cpu.rotate_right(cb_reg, false); },
        0x20..=0x27 => { unimplemented!(); },
        0x28..=0x2F => { unimplemented!(); },
        0x30..=0x37 => { unimplemented!(); },
        0x38..=0x3F => { unimplemented!(); },
        0x40..=0x7F => { unimplemented!(); },
        0x80..=0xBF => { unimplemented!(); },
        0xC0..=0xFF => { unimplemented!(); },
    }
    2
}
```

### Shifting

Next we'll cover `SLA`, `SRA`, and `SRL`, the shifting operations. These are very similar to the rotate ones we just created, except left shift always places in a 0, while arithmetic `SRA` and logical `SRL` right shifts either places 0 or a copy of the previously most significant bit. Those two can again share a single function with a boolean parameter.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn shift_left(&mut self, reg: Regs) {
        let val = self.get_r8(reg);
        let msb = val.get_bit(7);
        let res = val.wrapping_shl(1);

        self.set_r8(reg, res);
        self.set_flag(Flags::Z, res == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, msb);
    }

    pub fn shift_right(&mut self, reg: Regs, arith: bool) {
        let val = self.get_r8(reg);
        let lsb = val.get_bit(0);
        let msb = val.get_bit(7);
        let mut res = val.wrapping_shr(1);
        if arith {
            res.set_bit(7, msb);
        }

        self.set_r8(reg, res);
        self.set_flag(Flags::Z, res == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, lsb);
    }
```

With these in place, three more 0xCB ranges can be filled out.

```rust
// In cpu/opcodes.rs

fn execute_cb(cpu: &mut Cpu, op: u8) -> u8 {
    // 0x00-0x07 -> RLC
    // 0x08-0x0F -> RRC
    // 0x10-0x17 -> RL
    // 0x18-0x1F -> RR
    // 0x20-0x27 -> SLA
    // 0x28-0x2F -> SRA
    // 0x30-0x37 -> SWAP
    // 0x38-0x3F -> SRL
    // 0x40-0x7F -> BIT
    // 0x80-0xBF -> RES
    // 0xC0-0xFF -> SET

    let cb_reg = get_cb_reg(op);
    match op {
        0x00..=0x07 => { cpu.rotate_left(cb_reg, true); },
        0x08..=0x0F => { cpu.rotate_right(cb_reg, true); },
        0x10..=0x17 => { cpu.rotate_left(cb_reg, false); },
        0x18..=0x1F => { cpu.rotate_right(cb_reg, false); },
        0x20..=0x27 => { cpu.shift_left(cb_reg); },
        0x28..=0x2F => { cpu.shift_right(cb_reg, true); },
        0x30..=0x37 => { unimplemented!(); },
        0x38..=0x3F => { cpu.shift_right(cb_reg, false); },
        0x40..=0x7F => { unimplemented!(); },
        0x80..=0xBF => { unimplemented!(); },
        0xC0..=0xFF => { unimplemented!(); },
    }
    2
}
```

### Swapping

Next comes `SWAP`, which reverses the order of the first and last 4 bits of an 8-bit value.<sup>1</sup>

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn swap_bits(&mut self, reg: Regs) {
        let val = self.get_r8(reg);
        let low = val & 0xF;
        let high = (val & 0xF0) >> 4;
        let res = (low << 4) | high;

        self.set_r8(reg, res);
        self.set_flag(Flags::Z, res == 0);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, false);
        self.set_flag(Flags::C, false);
    }
}
```

This should be pretty straight-forward. Mask to get the high and low nibble, then swap their position by shifting and ORing them back together. With this, we have completed the first quarter of the 0xCB table. This might not seem like much, but the final 75% is largely the same, so things should pick up a bit.

<sup>1</sup> Fun fact! Four bits are called a "nibble", because it's half a byte.

### Bitwise

Looking at the table, there are three types of instructions remaining -- `BIT`, `SET`, and `RES`. The `BIT` instructions checks a bit in a given 8-bit register, and sets the Z flag to *the opposite* of the bit's value. It's a bit of a strange convention, but the Z flag checks if something is zero, so if the bit is zero, then the flag is set to true. `SET` and `RES` don't modify any flags, but either "set" or "reset" the specified bit in the specified 8-bit register. Eight possible bits times eight possible registers times three instructions gives our remaining 192 opcodes, which we will handle with only two helpers.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn test_bit(&mut self, reg: Regs, bit: u8) {
        let byte = self.get_r8(reg);
        let val = byte.get_bit(bit);

        self.set_flag(Flags::Z, !val);
        self.set_flag(Flags::N, false);
        self.set_flag(Flags::H, true);
    }

    pub fn write_bit(&mut self, reg: Regs, bit: u8, set: bool) {
        let mut byte = self.get_r8(reg);
        byte.set_bit(bit, set);
        self.set_r8(reg, byte);
    }
}
```

We're starting to run out of good function names, as `get_bit` and `set_bit` are taken, so we'll use `test` and `write` instead. With these added, we have completely implemented the 0xCB opcode table, and can update our match statement. There is one remaining item to decode however. We've determined the range for these instructions, and how to determine which register they operate on, but if we want to use our match statement, we will need to decode the pattern for which bit each instruction targets.

Looking at the `BIT` instructions, they begin at index 0x40 and proceed in blocks of eight for each bit, before moving on to the next, i.e. 0x40-0x47 are for bit 0, 0x48-0x4f are bit 1, 0x50-0x57 are bit 2, etc. The pattern is more clearly seen if we rewrite those ranges in binary.

| Hex Range | Binary Range          | Target Bit |
| --------- | --------------------- | ---------- |
| 0x40-0x47 | 0b01000000-0b01000111 | Bit 0      |
| 0x48-0x4F | 0b01001000-0b01001111 | Bit 1      |
| 0x50-0x57 | 0b01010000-0b01010111 | Bit 2      |
| 0x58-0x5F | 0b01011000-0b01011111 | Bit 3      |
| 0x60-0x67 | 0b01100000-0b01100111 | Bit 4      |
| 0x68-0x6F | 0b01101000-0b01101111 | Bit 5      |
| 0x70-0x77 | 0b01110000-0b01110111 | Bit 6      |
| 0x78-0x7F | 0b01111000-0b01111111 | Bit 7      |

We're looking for the subset of bits in each range that correspond to our target bit. In this case, it's bits 3-5 that encode this information (remember we start counting from 0 on the right). This pattern actually holds for the `SET` and `RES` instructions as well, the difference between the three categories is just highest two bits. We can mask those bits out and use them as the input for our helper functions.

| Hex Range | Binary Range              | Target Bit |
| --------- | ------------------------- | ---------- |
| 0x40-0x47 | 0b01*000*000-0b01*000*111 | Bit 0      |
| 0x48-0x4F | 0b01*001*000-0b01*001*111 | Bit 1      |
| 0x50-0x57 | 0b01*010*000-0b01*010*111 | Bit 2      |
| 0x58-0x5F | 0b01*011*000-0b01*011*111 | Bit 3      |
| 0x60-0x67 | 0b01*100*000-0b01*100*111 | Bit 4      |
| 0x68-0x6F | 0b01*101*000-0b01*101*111 | Bit 5      |
| 0x70-0x77 | 0b01*110*000-0b01*110*111 | Bit 6      |
| 0x78-0x7F | 0b01*111*000-0b01*111*111 | Bit 7      |

```rust
// In cpu/opcodes.rs

fn execute_cb(cpu: &mut Cpu, op: u8) -> u8 {
    // 0x00-0x07 -> RLC
    // 0x08-0x0F -> RRC
    // 0x10-0x17 -> RL
    // 0x18-0x1F -> RR
    // 0x20-0x27 -> SLA
    // 0x28-0x2F -> SRA
    // 0x30-0x37 -> SWAP
    // 0x38-0x3F -> SRL
    // 0x40-0x7F -> BIT
    // 0x80-0xBF -> RES
    // 0xC0-0xFF -> SET

    let cb_reg = get_cb_reg(op);
    match op {
        0x00..=0x07 => { cpu.rotate_left(cb_reg, true); },
        0x08..=0x0F => { cpu.rotate_right(cb_reg, true); },
        0x10..=0x17 => { cpu.rotate_left(cb_reg, false); },
        0x18..=0x1F => { cpu.rotate_right(cb_reg, false); },
        0x20..=0x27 => { cpu.shift_left(cb_reg); },
        0x28..=0x2F => { cpu.shift_right(cb_reg, true); },
        0x30..=0x37 => { cpu.swap_bits(cb_reg); },
        0x38..=0x3F => { cpu.shift_right(cb_reg, false); },
        0x40..=0x7F => {
            let bit = (op & 0b111000) >> 3;
            cpu.test_bit(cb_reg, bit);
        },
        0x80..=0xBF => {
            let bit = (op & 0b111000) >> 3;
            cpu.write_bit(cb_reg, bit, false);
        },
        0xC0..=0xFF => {
            let bit = (op & 0b111000) >> 3;
            cpu.write_bit(cb_reg, bit, true);
        },
    }
    2
}
```

With the largest block of instructions out of the way, don't forget to add `prefix_cb` to original `OPCODES` table before proceeding.

[*Next Chapter*](11-final-misc.md)
