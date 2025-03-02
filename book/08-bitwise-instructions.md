# VIII. Bitwise Operations

[*Return to Index*](../README.md)

[*Previous Chapter*](07-load-instructions.md)

We discussed how bitwise operators worked in [Chapter I](01-refresher.md), and for the purposes of this section we're going to explore `ADD`, `SUB`, `AND`, `OR`, `XOR`, and `CP`. I won't list out all of their indices, but they largely live in the block between 0x80 and 0xBF, with a few strays here and there. All of the instructions in that block deal with 8-bit values, and share a majority of their traits. We haven't yet discussed `CP` yet though, and it stands for "compare" (not copy). This instruction compares the two specified values, and leaves both alone, but updates some of the flags based on what it found. It's actually simpler to think of the `CP` instructions as subtracting the two values, but discarding the result, which makes items such as the Half-Carry flag have more context.

In that block, there are two types of instructions I haven't mentioned yet though, and those are `ADC` and `SBC`. These are the "add with carry" and "subtract with carry" instructions. These are the first of our instructions to actually use the flags. In this case, `ADC` will add the two values, as the `ADD` command does, but then it will also add the value of the Carry flag. This has some useful applications such as using the overflow from a previous instruction as part of a later calculation. `SBC` uses the carry flag in the same way, but with subtraction.

There are a few bitwise operators outside of this block, but they follow a similar pattern to other commands we've seen before. 0xC6, `ADD A, u8` adds an immediate value into the A register, and other bitwise operators in the bottom quarter of the chart do a similar maneuver. Finally there is 0xE8, `ADD SP, i8`, which is similar to 0xC6, but treats its immediate value as a signed integer, and adds it to the SP register. We'll cover that in more detail in a moment.

## 8-bit Helpers

By this point in development, you should be well versed in how to handle 8-bit versus 16-bit operations, so these shouldn't pose too much of a challenge. Due to the large amount of repetition, we will again create some helper functions in our `Cpu` object to reduce the amount of code duplication. While there were load instructions for every combination of 8-bit registers, the majority of the 8-bit bitwise operations use the A register as an input and as the output. We'll create helpers that take an 8-bit value and apply it to the value stored in the A register. For example, a helper function for the `AND` instructions will look something like so.

```rust
// In cpu/mod.rs
// Unchanged code omitted

pub fn and_a_u8(&mut self, val: u8) {
    let mut a = self.get_r8(Regs::A);
    a &= val;

    self.set_r8(Regs::A, a);
    self.set_flag(Flags::Z, a == 0);
    self.set_flag(Flags::N, false);
    self.set_flag(Flags::H, true);
    self.set_flag(Flags::C, false);
}
```

This function updates the value in the A register using Rust's AND operator, `&`. It also sets the flags to the specification that all the `AND` instructions share, namely `Z010`. While it would be nice to accept an 8-bit register enum value, we'll use this function to also accommodate handling an immediate value, such as 0xE6 `AND A, u8`, so it will be up to the caller to retrieve the value itself. In Rust, the `OR` operator is `|` and the `XOR` operator `^`, so nearly identical helper functions can be written for those two instruction types, taking care to update the flags accordingly (an exercise I'll leave for the reader).

As mentioned previously, there are two types of 8-bit addition instructions, one which uses the carry flag (`ADC`) and one that does not (`ADD`). The small different between the two instructions means it's easiest to write a single helper function that accepts a boolean parameter for whether or not to use the carry flag. If set to true, and the C flag is actually set, then we will add 1 to the sum.

```rust
// In cpu/mod.rs
// Unchanged code omitted

pub fn add_a_u8(&mut self, val: u8, adc: bool) {
    let mut carry = 0;
    if adc && self.get_flag(Flags::C) {
        carry = 1;
    }
    let a = self.get_r8(Regs::A);
    let result1 = a.overflowing_add(val);
    let h_check1 = check_h_carry_u8(a, val);
    let result2 = result1.0.overflowing_add(carry);
    let h_check2 = check_h_carry_u8(result1.0, carry);
    let set_h = h_check1 || h_check2;
    let set_c = result1.1 || result2.1;

    self.set_flag(Flags::N, false);
    self.set_flag(Flags::C, set_c);
    self.set_flag(Flags::H, set_h);
    self.set_flag(Flags::Z, result2.0 == 0);
    self.set_r8(Regs::A, result2.0);
}
```

This function might look more complicated than you'd expect, and that is due to ensuring that we're updating the flags correctly. We're technically doing two additions here, the first when we add our 8-bit `val`, and the second when we add the carry flag. *Both* of these additions have the potential trip the carry or half carry flags, so we need to ensure that we're keeping track if the flags get set during either sum. The `SUB` and `SBC` helper function is structured in a similar fashion for the same reason.

```rust
// In cpu/mod.rs
// Unchanged code omitted

pub fn sub_a_u8(&mut self, val: u8, sbc: bool) {
    let mut carry = 0;
    if sbc && self.get_flag(Flags::C) {
        carry = 1;
    }
    let a = self.get_r8(Regs::A);
    let result1 = a.overflowing_sub(val);
    let check_h1 = check_h_borrow_u8(a, val);
    let result2 = result1.0.overflowing_sub(carry);
    let check_h2 = check_h_borrow_u8(result1.0, carry);
    let set_h = check_h1 || check_h2;

    self.set_flag(Flags::N, true);
    self.set_flag(Flags::Z, result2.0 == 0);
    self.set_flag(Flags::H, set_h);
    self.set_flag(Flags::C, result1.1 || result2.1);
    self.set_r8(Regs::A, result2.0);
}
```

One other thing to note. As I write this, Rust has an experimental `carrying_add` and `borrowing_sub` which accepts a carry flag boolean as a parameter. Given that it's still experimental, I don't wish to use it, but if that's a stable API in the future that you live in, you're welcome to see if it helps simplify your own code.

Next, we'll need to create a helper function for the compare `CP` instructions. These instructions perform a comparison between two 8-bit values by subtracting the two -- but not storing the output -- and updating the flags accordingly. The Z flag will thus be set if the two values are equal, the N flag will always be set since it's a subtraction operation, and C will be set anytime the first value (which is always the value in the A register) is smaller than the second.

```rust
// In cpu/mod.rs

pub fn cp_a_u8(&mut self, val: u8) {
    let a = self.get_r8(Regs::A);
    let set_h = check_h_borrow_u8(a, val);

    self.set_flag(Flags::Z, a == val);
    self.set_flag(Flags::N, true);
    self.set_flag(Flags::H, set_h);
    self.set_flag(Flags::C, a < val);
}
```

With this, we've covered all of the 8-bit bitwise helpers, which are used by nearly all of the instructions. There is one final category, however, and those are the 16-bit `ADD` instructions, of which there are four. These add a value in one 16-bit register to another, so we can create a helper function that does just that.

```rust
// In cpu/mod.rs

pub fn add_r16(&mut self, dst_r: Regs16, src_r: Regs16) {
    let dst = self.get_r16(dst_r);
    let src = self.get_r16(src_r);
    let res = dst.overflowing_add(src);
    let set_h = check_h_carry_u16(dst, src);

    self.set_r16(dst_r, res.0);
    self.set_flag(Flags::N, false);
    self.set_flag(Flags::H, set_h);
    self.set_flag(Flags::C, res.1);
}
```

The function accepts two 16-bit register enum values, the destination -- where we will store the sum -- and the source register. While we use our usual `check_h_carry_u16` to perform the H flag check, we can use `overflowing_add` to both calculate the final sum, and check if there has been an overflow, which is the criteria for the C flag.

We can now return to the (repetitive) task of putting all these functions to good use. With our helper functions in place, everything should be pretty straight-forward, with one notable exception. For the sake of completion though, here are a few examples.

```rust
// In cpu/opcodes.rs

// ADD HL, BC
// -0HC
fn add_09(cpu: &mut Cpu) -> u8 {
    cpu.add_r16(Regs16::HL, Regs16::BC);
    2
}

// ADD A, B
// Z0HC
fn add_80(cpu: &mut Cpu) -> u8 {
    let val = cpu.get_r8(Regs::B);
    cpu.add_a_u8(val, false);
    1
}

// SBC A, (HL)
// Z1HC
fn sbc_9e(cpu: &mut Cpu) -> u8 {
    let val = cpu.get_r8(Regs::HL);
    cpu.sub_a_u8(val, true);
    2
}

// CP A, u8
// Z1HC
fn cp_fe(cpu: &mut Cpu) -> u8 {
    let val = cpu.fetch();
    cpu.cp_a_u8(val);
    2
}
```

Sadly, there is one instruction that doesn't match the pattern of any of our helper functions, and that's 0xE8 `ADD SP, i8`, due to it being a 16-bit immediate addition instruction. For this one opcode, we shall have to implement it explicitly.

```rust
// ADD SP, i8
// 00HC
fn add_e8(cpu: &mut Cpu) -> u8 {
    let val = cpu.fetch() as i8 as u16;
    let sp = cpu.get_r16(Regs16::SP);
    let res = sp.wrapping_add(val);
    let set_c = check_c_carry_u16(sp, val);
    let set_h = check_h_carry_u16(sp, val);

    cpu.set_r16(Regs16::SP, res);
    cpu.set_flag(Flags::Z, false);
    cpu.set_flag(Flags::N, false);
    cpu.set_flag(Flags::H, set_h);
    cpu.set_flag(Flags::C, set_c);
    2
}
```

This function strongly resemebles `add_r16`, but rather than reading from a source register, the system will first fetch a value. This value needs to be treated as a signed value (and then cast into a `u16` so the Rust compiler will allow addition with other `u16`). This is the only exception however, and once you have all of the bitwise operations in place, the new updated `OPCODES` array should look something like this.

```rust
const OPCODES: [fn(&mut Cpu) -> u8; 256] = [
//  0x00,   0x01,   0x02,   0x03,   0x04,   0x05,   0x06,   0x07,   0x08,   0x09,   0x0A,   0x0B,   0x0C,   0x0D,   0x0E,   0x0F
    nop_00, ld_01,  ld_02,  inc_03, inc_04, dec_05, ld_06,  todo,   ld_08,  add_09, ld_0a,  dec_0b, inc_0c, dec_0d, ld_0e,  todo,   // 0x00
    todo,   ld_11,  ld_12,  inc_13, inc_14, dec_15, ld_16,  todo,   todo,   add_19, ld_1a,  dec_1b, inc_1c, dec_1d, ld_1e,  todo,   // 0x10
    todo,   ld_21,  ld_22,  inc_23, inc_24, dec_25, ld_26,  todo,   todo,   add_29, ld_2a,  dec_2b, inc_2c, dec_2d, ld_2e,  todo,   // 0x20
    todo,   ld_31,  ld_32,  inc_33, inc_34, dec_35, ld_36,  todo,   todo,   add_39, ld_3a,  dec_3b, inc_3c, dec_3d, ld_3e,  todo,   // 0x30
    ld_40,  ld_41,  ld_42,  ld_43,  ld_44,  ld_45,  ld_46,  ld_47,  ld_48,  ld_49,  ld_4a,  ld_4b,  ld_4c,  ld_4d,  ld_4e,  ld_4f,  // 0x40
    ld_50,  ld_51,  ld_52,  ld_53,  ld_54,  ld_55,  ld_56,  ld_57,  ld_58,  ld_59,  ld_5a,  ld_5b,  ld_5c,  ld_5d,  ld_5e,  ld_5f,  // 0x50
    ld_60,  ld_61,  ld_62,  ld_63,  ld_64,  ld_65,  ld_66,  ld_67,  ld_68,  ld_69,  ld_6a,  ld_6b,  ld_6c,  ld_6d,  ld_6e,  ld_6f,  // 0x60
    ld_70,  ld_71,  ld_72,  ld_73,  ld_74,  ld_75,  todo,   ld_77,  ld_78,  ld_79,  ld_7a,  ld_7b,  ld_7c,  ld_7d,  ld_7e,  ld_7f,  // 0x70
    add_80, add_81, add_82, add_83, add_84, add_85, add_86, add_87, adc_88, adc_89, adc_8a, adc_8b, adc_8c, adc_8d, adc_8e, adc_8f, // 0x80
    sub_90, sub_91, sub_92, sub_93, sub_94, sub_95, sub_96, sub_97, sbc_98, sbc_99, sbc_9a, sbc_9b, sbc_9c, sbc_9d, sbc_9e, sbc_9f, // 0x90
    and_a0, and_a1, and_a2, and_a3, and_a4, and_a5, and_a6, and_a7, xor_a8, xor_a9, xor_aa, xor_ab, xor_ac, xor_ad, xor_ae, xor_af, // 0xA0
    or_b0,  or_b1,  or_b2,  or_b3,  or_b4,  or_b5,  or_b6,  or_b7,  cp_b8,  cp_b9,  cp_ba,  cp_bb,  cp_bc,  cp_bd,  cp_be,  cp_bf,  // 0xB0
    todo,   todo,   todo,   todo,   todo,   todo,   add_c6, todo,   todo,   todo,   todo,   todo,   todo,   todo,   adc_ce, todo,   // 0xC0
    todo,   todo,   todo,   todo,   todo,   todo,   sub_d6, todo,   todo,   todo,   todo,   todo,   todo,   todo,   sbc_de, todo,   // 0xD0
    ld_e0,  todo,   ld_e2,  todo,   todo,   todo,   and_e6, todo,   add_e8, todo,   ld_ea,  todo,   todo,   todo,   xor_ee, todo,   // 0xE0
    ld_f0,  todo,   ld_f2,  todo,   todo,   todo,   or_f6,  todo,   ld_f8,  ld_f9,  ld_fa,  todo,   todo,   todo,   cp_fe,  todo,   // 0xF0
];
```

[*Next Chapter*](09-stack.md)
