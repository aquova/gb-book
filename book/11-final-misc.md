# XI. Final Misc. Instructions

## A Register Rotation

Time to clean up the final remaining instructions. While we implemented many bitwise rotation instructions in the 0xCB table, there are actually four in the regular table -- 0x07 `RLCA`, 0x0F `RRCA`, 0x17 `RLA`, and 0x1F `RRA`. These operate the same was as the rotate instructions we've implemented previously, with these operating on the A register. This might seem odd, as we already had rotation instructions for the A register in the 0xCB table, and you'd be right, with the one difference being how the Z flag is set. My guess is that these would be the instructions typically used (as they are fewer bytes per instruction), while the entries in the 0xCB table exist to complete the table pattern. In any case, we'll implement them again, taking care to handle the Z flag differently.

```rust
// In cpu/opcodes.rs

// RLCA
// 000C
fn rlca_07(cpu: &mut Cpu) -> u8 {
    cpu.rotate_left(Regs::A, true);
    cpu.set_flag(Flags::Z, false);
    1
}

// RRCA
// 000C
fn rrca_0f(cpu: &mut Cpu) -> u8 {
    cpu.rotate_right(Regs::A, true);
    cpu.set_flag(Flags::Z, false);
    1
}

// RLA
// 000C
fn rla_17(cpu: &mut Cpu) -> u8 {
    cpu.rotate_left(Regs::A, false);
    cpu.set_flag(Flags::Z, false);
    1
}

// RRA
// 000C
fn rra_1f(cpu: &mut Cpu) -> u8 {
    cpu.rotate_right(Regs::A, false);
    cpu.set_flag(Flags::Z, false);
    1
}
```

## Modify Carry Flag

Next, there are a pair of instructions that modifies the C flag. Up until now, the flags have only been modified as a side effect of other behavior, but it would be beneficial to manipulate them directly. These instructions are 0x37 `SCF` "Set Carry Flag", and 0x3F `CCF` "Compliment Carry Flag". "Compliment" here means to flip all of the bits in a value, so all the 0s become 1s and vice versa.

```rust
// In cpu/opcodes.

// SCF
// -001
fn scf_37(cpu: &mut Cpu) -> u8 {
    cpu.set_flag(Flags::N, false);
    cpu.set_flag(Flags::H, false);
    cpu.set_flag(Flags::C, false);
    1
}

// CCF
// -00C
fn ccf_3f(cpu: &mut Cpu) -> u8 {
    let c = cpu.get_flag(Flags::C);
    cpu.set_flag(Flags::N, false);
    cpu.set_flag(Flags::H, false);
    cpu.set_flag(Flags::C, !c);
    1
}
```

Speaking of compliment, there is instruction 0x2F `CPL` which compliments the value in the A register. The `!` operator in Rust is most commonly used for boolean values, when used on an integer it flips all the value's bits, exactly what we want here.

```rust
// In cpu/opcodes.rs

// CPL
// -11-
fn cpl_2f(cpu: &mut Cpu) -> u8 {
    let a = cpu.get_r8(Regs::A);
    cpu.set_r8(Regs::A, !a);
    cpu.set_flag(Flags::N, true);
    cpu.set_flag(Flags::H, true);
    1
}
```

## Interrupts

I've alluded to the interrupts a few times, and given how important they'll eventually be to the operation of the system, it might be a bit surprising that they haven't played a larger part in the implementation of the CPU. We'll cover them in detail in a later chapter, but as an overview, the interrupts act just as their name implies -- they interrupt the CPU's normal behavior to perform some other task. There are several interrupt types with different purposes, such as when the player presses a button or during certain stages of the screen rendering process. When they occur, the CPU pauses what it's doing, executes some interrupt-specific code, then returns back to where it was. For systems such as user input, this is critical, as the Game Boy doesn't have enough processing power to continuously poll for button state. Instead, it executes the game normally, and only handles a button when told to do so.

We'll see how this all fits together soon, but for now, there are three instructions that deal with the interrupt handling. Two of them -- 0xF3 `DI` and 0xFB `EI`, "Disable Interrupts" and "Enable Interrupts" respectively -- simply enable or disable all interrupt requests while the third -- 0xD9 `RETI` -- is a combination of `RET` and `EI` into a single instruction. There are some situations where the CPU might not want to stop what it's doing to handle an interrupt signal, so the CPU has the option to ignore them if need be. For this, we'll expand our `Cpu` struct slightly by including a flag for whether we're currently accepting interrupts.

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
    irq_enabled: bool,
}

impl Cpu {
    // Unchanged code omitted

    pub fn set_irq(&mut self, enabled: bool) {
        self.irq_enabled = enabled;
    }
}
```

"IRQ" is a commonly used abbreviation for "interrupt request". When it comes time to execute an interrupt, one of the first steps will be to check this boolean flag to see if the CPU is even willing to accept an interrupt request at this time. As said, this can be changed via one of three CPU instructions.

```rust
// In cpu/opcodes.rs

// RETI
// ----
fn reti_d9(cpu: &mut Cpu) -> u8 {
    let addr = cpu.pop();
    cpu.set_pc(addr);
    cpu.set_irq(true);
    4
}

// DI
// ----
fn di_f3(cpu: &mut Cpu) -> u8 {
    cpu.set_irq(false);
    4
}

// EI
// ----
fn ei_fb(cpu: &mut Cpu) -> u8 {
    cpu.set_irq(true);
    4
}
```

## Halting and Stopping

The Game Boy has two power saving modes, both of which have a CPU instruction to activate them. 0x76 `HALT` pauses CPU execution until an interrupt occurs, which will resume normal operation. 0x10 `STOP` also pauses the CPU, but has different conditions for re-awakening. To perfectly honest, the `STOP` instruction was never used for commercially released hardware, and has shown to have bizarre side effects on real hardware if the utmost care isn't taken ([more information here for those curious](https://gbdev.io/pandocs/Reducing_Power_Consumption.html)). Given our goal is a functional but not completely accurate emulator, we're not going to bother handling `STOP` correctly (or at all), as it's more trouble than it's worth. To implement `HALT` then, we'll need to add another boolean to our `Cpu` object.

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
    irq_enabled: bool,
    halted: bool,
}

impl Cpu {
    // Unchanged code omitted

    pub fn set_halted(&mut self, halted: bool) {
        self.halted = halted;
    }
}
```

For `STOP`, we'll simply return the number of cycles and perform no other operation. For `HALT`, we will need to set the `halted` boolean in our CPU.

```rust
// In cpu/opcodes.rs

// STOP
// ----
fn stop_10(_cpu: &mut Cpu) -> u8 {
    // Do nothing
    1
}

// HALT
// ----
fn halt_76(cpu: &mut Cpu) -> u8 {
    cpu.set_halted(true);
    1
}
```

## Unused Instructions

Let's now turn our attention to the largest remaining collection of instructions, those which the reference table lists as "invalid". As their name implies, these instructions are unused. They have no defined purpose and attempting to use them would've gotten you a rejection letter during your Nintendo quality approval process. Thus, while some researchers have attempted to document what actually happens if you try to utilize these instructions, our emulator will throw an assertion, as their usage would indicate a problem in our implementation.

```rust
// In cpu/opcodes.rs

fn invalid(_cpu: &mut Cpu) -> u8 {
    panic!("Invalid opcode");
}
```

This new `invalid` function can be used in the eleven opcode indices without a defined purpose.

## DAA

*Much of this explanation was based off this [excellent article](https://ehaskins.com/2018-01-30%20Z80%20DAA/)*

Our `OPCODES` table should now only have one `todo` remaining, that which belongs to instruction 0x27 `DAA`. I have saved `DAA` to last because it is quite different than the other instructions we have looked at, and it requires a bit of a math lesson. `DAA` is an acronym standing for "Decimal Adjust Accumulator", with "Accumulator" in this case referring to our A register. This instruction tells the CPU to take the value in the A register and treat it as a format known as "Binary Coded Decimal" (BCD). By this point in the project, you are very familiar with the typical binary representation of numbers -- using powers of two to represent a number.

While normal base 2 is by far the most prevalent way to encode a decimal value into binary, there are other methods, such as instead of converting the whole number, you treat each digit separately. For example, take the decimal number 123. That number takes up a single byte and can be written as 0111 1011 (broken into groups of four bits to make it easier to read). Instead, BCD converts each digit of our decimal value into binary separately. Since there are ten possible decimal digits, that requires four bits, so 123 would require 12 bits to convert to BCD, and would be written as 0001 0010 0011. This is equal to 0x0123, illustrating why BCD has its uses as a more human-readably binary format.

What happens if you perform addition and subtraction in BCD? It actually works more or less fine, as since we've separated all of the decimal values into their own 4-bit chunks, adding them together produces another BCD value. It doesn't work for all values however, as adding larger digits together will either overflow into the next digit's space, or produce a 4-bit subsection larger than 9, and thus no longer a valid BCD value. Some correction is needed to ensure that decimal carrying works properly.

As an example, let's add 17 + 25 together.

```
  17 |   0001 0111
+ 25 | + 0010 0101
  --     ---------
  42 |   0011 1100
```

Our result should have been 42, which is 0100 0010, but our BCD sum ended up being 0011 1100 which is 3 in the tens place and 12 in the ones place, which is not a valid BCD number. Instead of carrying in the normal binary way, we should have carried a 1 into the upper four bits once we passed nine in the lower nibble. To put it another way, if we cross nine for any BCD digit, we should add one to the next digit, and add six to correctly roll us over (adding six gets us back to 0-9 space).

So what does this have to do with the `DAA` instruction? The `DAA` instruction performs that addition correction we just described. It is designed to be used after some other operation has placed a BCD value into the A register, at which point it performs the needed corrections to ensure the A register is still holding a valid and accurate BCD value. These corrections utilize the H, C, and N flags to make the proper adjustments, and require `DAA` to be called immediately after the addition/subtraction operation to ensure the flags are still correct.

To do so, the CPU needs to perform the same checks as we did in our example. Utilizing both the carry/half carry flags along with seeing if any BCD "digit" over or underflowed, the appropriate corrections can be made. You're welcome to make your own attempt at this implementation, but if you're having troubles, one is shown below.

```rust
// In cpu/opcodes.rs

// DAA
// Z-0C
fn daa_27(cpu: &mut Cpu) -> u8 {
    let mut a = cpu.get_r8(Regs::A) as i32;

    if cpu.get_flag(Flags::N) {
        if cpu.get_flag(Flags::H) {
            a = (a - 6) & 0xFF;
        }
        if cpu.get_flag(Flags::C) {
            a -= 0x60;
        }
    } else {
        if cpu.get_flag(Flags::H) || (a & 0x0F) > 0x09 {
            a += 0x06;
        }
        if cpu.get_flag(Flags::C) || a > 0x9F {
            a +=  0x60;
        }
    }

    if (a & 0x100) == 0x100 {
        cpu.set_flag(Flags::C, true);
    }
    a &= 0xFF;
    cpu.set_r8(Regs::A, a as u8);
    cpu.set_flag(Flags::Z, a == 0);
    cpu.set_flag(Flags::H, false);
    1
}
```

## Conclusion

With `DAA` in place, the entire CPU instruction set has been implemented. Ensure that your `OPCODES` table is entirely filled out and resembles something like this.

```rust
const OPCODES: [fn(&mut Cpu) -> u8; 256] = [
//  0x00,    0x01,   0x02,   0x03,    0x04,    0x05,    0x06,    0x07,    0x08,   0x09,    0x0A,   0x0B,      0x0C,    0x0D,    0x0E,   0x0F
    nop_00,  ld_01,  ld_02,  inc_03,  inc_04,  dec_05,  ld_06,   rlca_07, ld_08,  add_09,  ld_0a,  dec_0b,    inc_0c,  dec_0d,  ld_0e,  rrca_0f, // 0x00
    stop_10, ld_11,  ld_12,  inc_13,  inc_14,  dec_15,  ld_16,   rla_17,  jr_18,  add_19,  ld_1a,  dec_1b,    inc_1c,  dec_1d,  ld_1e,  rra_1f,  // 0x10
    jr_20,   ld_21,  ld_22,  inc_23,  inc_24,  dec_25,  ld_26,   daa_27,  jr_28,  add_29,  ld_2a,  dec_2b,    inc_2c,  dec_2d,  ld_2e,  cpl_2f,  // 0x20
    jr_30,   ld_31,  ld_32,  inc_33,  inc_34,  dec_35,  ld_36,   scf_37,  jr_38,  add_39,  ld_3a,  dec_3b,    inc_3c,  dec_3d,  ld_3e,  ccf_3f,  // 0x30
    ld_40,   ld_41,  ld_42,  ld_43,   ld_44,   ld_45,   ld_46,   ld_47,   ld_48,  ld_49,   ld_4a,  ld_4b,     ld_4c,   ld_4d,   ld_4e,  ld_4f,   // 0x40
    ld_50,   ld_51,  ld_52,  ld_53,   ld_54,   ld_55,   ld_56,   ld_57,   ld_58,  ld_59,   ld_5a,  ld_5b,     ld_5c,   ld_5d,   ld_5e,  ld_5f,   // 0x50
    ld_60,   ld_61,  ld_62,  ld_63,   ld_64,   ld_65,   ld_66,   ld_67,   ld_68,  ld_69,   ld_6a,  ld_6b,     ld_6c,   ld_6d,   ld_6e,  ld_6f,   // 0x60
    ld_70,   ld_71,  ld_72,  ld_73,   ld_74,   ld_75,   halt_76, ld_77,   ld_78,  ld_79,   ld_7a,  ld_7b,     ld_7c,   ld_7d,   ld_7e,  ld_7f,   // 0x70
    add_80,  add_81, add_82, add_83,  add_84,  add_85,  add_86,  add_87,  adc_88, adc_89,  adc_8a, adc_8b,    adc_8c,  adc_8d,  adc_8e, adc_8f,  // 0x80
    sub_90,  sub_91, sub_92, sub_93,  sub_94,  sub_95,  sub_96,  sub_97,  sbc_98, sbc_99,  sbc_9a, sbc_9b,    sbc_9c,  sbc_9d,  sbc_9e, sbc_9f,  // 0x90
    and_a0,  and_a1, and_a2, and_a3,  and_a4,  and_a5,  and_a6,  and_a7,  xor_a8, xor_a9,  xor_aa, xor_ab,    xor_ac,  xor_ad,  xor_ae, xor_af,  // 0xA0
    or_b0,   or_b1,  or_b2,  or_b3,   or_b4,   or_b5,   or_b6,   or_b7,   cp_b8,  cp_b9,   cp_ba,  cp_bb,     cp_bc,   cp_bd,   cp_be,  cp_bf,   // 0xB0
    ret_c0,  pop_c1, jp_c2,  jp_c3,   call_c4, push_c5, add_c6,  rst_c7,  ret_c8, ret_c9,  jp_ca,  prefix_cb, call_cc, call_cd, adc_ce, rst_cf,  // 0xC0
    ret_d0,  pop_d1, jp_d2,  invalid, call_d4, push_d5, sub_d6,  rst_d7,  ret_d8, reti_d9, jp_da,  invalid,   call_dc, invalid, sbc_de, rst_df,  // 0xD0
    ld_e0,   pop_e1, ld_e2,  invalid, invalid, push_e5, and_e6,  rst_e7,  add_e8, jp_e9,   ld_ea,  invalid,   invalid, invalid, xor_ee, rst_ef,  // 0xE0
    ld_f0,   pop_f1, ld_f2,  di_f3,   invalid, push_f5, or_f6,   rst_f7,  ld_f8,  ld_f9,   ld_fa,  ei_fb,     invalid, invalid, cp_fe,  rst_ff,  // 0xF0
];
```

Also delete the `todo` function, as it no longer should be used anywhere. The `execute` function inside of `opcodes.rs` is now fully functional, so our final step should be to call it. Return to `cpu/mod.rs` and add a new `tick` function. This function will eventually be the "heartbeat" of the entire emulator. When told to tick, the CPU will fetch and execute the next instruction (or do nothing if halted).

The `execute` function does return a cycles count, but we're not quite ready to utilize it, so for now we'll leave it hanging and return to it later. Likewise, `tick` will eventually need to return a boolean, to let its frontend caller know if its time to render a frame or not. For now, we'll just have it always return false.

```rust
// In cpu/mod.rs

// Unchanged code omitted

impl Cpu {
    pub fn tick(&mut self) -> bool {
        let cycles = if self.halted { 1 } else { opcodes::execute(self) };
        false
    }
}
```

We also need to update the constructor for our `Cpu` object. When we first added it, I mentioned that the initial values for our CPU registers aren't obvious. You would think that they would all be set to 0x00 when a game begins to run, but that's not quite the case. All Game Boys have a small, 256 byte piece of code embedded in the system that runs at start up. This *boot ROM* performs verification of the game cartridge and scrolls the Nintendo boot graphic with the "ding" noise. Because this piece of code always runs and always leaves the Game Boy in the same initial state at the beginning of game execution, we can just pretend that it ran and initialized our system to those conditions. It also allows us to avoid including and running proprietary Nintendo code and graphics. Here then is the constructor for our `Cpu` object, complete with the "magic values" for the registers and RAM that all Game Boy titles begin with.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted
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
}
```

These magic values aren't obvious by any means, but these are the changes that the bootloader makes, and this is the state all Game Boys expect themselves to be in when a game starts running. One interesting thing to note is that the PC doesn't start at 0x0000, but instead at 0x0100, meaning that the entry point to all Game Boy games needs to be placed at that address in ROM. If you plan on continuing your emulator after this tutorial and implementing Super Game Boy functionality, its bootloader sets RAM address 0xFF26 as 0xF0, not 0xF1, hence the comment.

Despite implementing roughly 500 different instructions, we've yet to test if any of them work as they should. Fortunately, the CPU should actually be quite straight-forward to test. We cycle through all the different operations, providing a series of different inputs and ensure that the registers, flags, and cycle count all match what they should. While you're welcome to implement a test suite yourself, it might be somewhat tricky to know what the correct values actually are for each instruction. Fortunately, the Game Boy community has created just a test suite in the form of authentic Game Boy ROMs. These "Blargg tests" (named after their author) will check the results of a wide range of CPU inputs, verifying that the result is correct, even for some uncommon edge cases. We'll focus on the tests for the CPU, although there are other, more sophisticated ones for testing things like system timing.

Unfortunately, while these tests are a great tool, our emulator is not ready to actually run them. As I said, they're to be run as an actual Game Boy game, which means they require at least some basic system functionality to begin. This includes a functional CPU, which we hopefully have now, but also the RAM, cartridge loading, and some of the video rendering needs to be complete as well. It will be our immediate goal to run these CPU verification tests, so next we will turn our attention to completing these three sub-systems.
