# IX. Stack & Program Flow Instructions

The stack is a special area of RAM used for "pushing" and "popping" 16-bit values, either manually or as part of other instructions. It's useful for instances where the developer needs to store data without caring about its memory location, only that it's saved temporarily. The name "stack" is apt, and you can think of it like a stack of objects, where the user can only access the item on the very top. The stack obeys a principle often referred to as "First In, Last Out" (FIFO). So if you have a stack of five items and want the fourth one down, you'll need to pop off the top three before it can be popped itself. Some systems have the ability to "peek" at items on the stack without removing them, but the Game Boy doesn't offer that instruction<sup>1</sup>.

Following our usual process, let's set up `push` and `pop` functions to be used in their respective instructions. Firstly though, where is the stack? For the Game Boy, the stack is defined to be at the very end of RAM -- at address 0xFFFE. When a 16-bit value is pushed, it is stored in little endian order, and the Stack Pointer decreases by two. When the system pops a value, the opposite occurs, and the 16-bit value is reconstructed and returned to the user. The 16-bits of data pointed to by the SP is read, and SP is decreased by two. Attempting to pop a value when the stack is empty (back at 0xFFFE) is undefined behavior, but we will treat it as an error, as it would likely be a bug in our emulator. Let's go ahead and define pushing and popping to our CPU.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn pop(&mut self) -> u16 {
        assert_ne!(self.sp, 0xFFFE, "Trying to pop when the stack is empty");
        let low = self.read_ram(self.sp);
        let high = self.read_ram(self.sp + 1);
        let val = merge_bytes(high, low);
        self.sp += 2;
        return val;
    }

    pub fn push(&mut self, val: u16) {
        self.sp -= 2;
        self.write_ram(self.sp, val.low_byte());
        self.write_ram(self.sp + 1, val.high_byte());
    }
}
```

It's important to note that the SP points *at* the top value on the stack. Thus we need to be careful when reading and writing to the stack that the SP is updated at the right time. With these in place, implementing the `PUSH` and `POP` instructions are trivial. We could create another helper that would wrap the functions with a call to get one of the 16-bit register values, but I think that's a bit overkill. Instead it'll look something like the following.

```rust
// In cpu/opcodes.rs

fn pop_c1(cpu: &mut Cpu) -> u8 {
    let val = cpu.pop();
    cpu.set_r16(Regs16::BC, val);
    1
}
```

There are four `PUSH` and `POP` instructions each to complete, and don't forget to update the `OPCODES` table as well.

<sup>1</sup> At least not natively. Since the top of the stack is stored in the Stack Pointer, it's simple enough to calculate where in RAM your value would be located and to use a `LOAD` instruction to take a look at it.

## Program Flow

So far, we have only seen the Program Counter moving continuously from address to address. This is fine for the instructions we've covered so far, but it would require the game developer to write their game entirely one line after the other, with no way to add functions or conditionals. Fortunately, this is not the case, and there are a number of instructions that deal with moving the PC to new areas.

First, we have the *jump instructions*, abbreviated as `JP`. These move the PC to a newly specified location in RAM, with some of the instructions only allowing the jump to occur if some condition is met. Look at 0xC3, `JP u16`, for example. This instruction fetches twice to get a 16-bit value, and then sets the PC to that address, effectively jumping there. 0x18 is a *relative jump*, abbreviated `JR`, meaning it doesn't move to an absolute address, but instead moves X number of bytes back or forth. In this case, `JR i8` means it fetches the next value and treats it as a signed 8-bit integer, so it can jump either forward or back. In addition to these unconditional jumps, conditional jumps are also supported. 0xC2 is an example of this, `JP NZ, u16`. This instruction looks at the Z flag, and if it is *not* true then the PC will jump to the 16-bit fetched value. There are several of these conditional instructions, both for relative and absolute jumps, but they only check if the Z or C flags are true or false, the other two flags are never considered.

What about subroutines? It would be fairly easy to jump to the start of some function, but how would we ever get back? While there are ways for a programmer to handle this manually, the Game Boy CPU conveniently has instructions for subroutines, a combination of the `CALL` instructions to enter a subroutine, and the `RET` instruction to return back. `CALL` does what it says, it's calls a function, either unconditionally (like with 0xCD, `CALL u16`) or conditionally by checking a flag condition, like the jump instructions do (for example, 0xCC, `CALL Z, u16`). These differ from the Jump instructions by incorporating the stack, saving the previous instruction of execution for when the subroutine completes. When the Game Boy receives a `CALL` instruction, it takes the current PC value, pushes it to the stack, then jumps to the desired location to begin a function. When that function returns with `RET`, the previous Program Counter value is popped back out, effectively moving execution back to where we previously were. It's a simple yet elegant system, and saves the developer the trouble of doing it manually, although it does require that the stack be returned to the same state that the function started with, otherwise it might not jump back to the right location.

There are also several `RST` or "reset" instructions. These are special `CALL` instructions that, depending on the instruction, have a dedicated destination rather than fetching where to go. For example, instruction 0xCF is `RST 0x08`, meaning the CPU will follow the normal `CALL` procedure of putting the PC onto the stack, and moving to address 0x0008. It's the responsibility of the developer to ensure that there is a proper function waiting for it there.

All of these functions deal with manipulating the Program Counter, something we haven't done outside of our `fetch` function. We'll begin with the `JP` and `JR` instructions, which don't require any helper functions themselves, but I am going to add a getter and setter for manipulating the PC. You could add this as part of `set_r16` if you wish, but given its importance, I'm going to give it dedicated functions.

```rust
// In cpu/mod.rs

impl Cpu {
    // Unchanged code omitted

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    pub fn set_pc(&mut self, val: u16) {
        self.pc = val;
    }
}
```

The `JP` and `JR` instructions are not difficult conceptually. `JP` will fetch a 16-bit value and move the PC to that address, while `JR` will fetch an 8-bit signed value and add that to the current PC value. Both types also have several opcodes which reference the value of a flag (the first time the flags are actually being used). If the flag in question is set, then the jump takes place. If not, the PC stays where it's at. This difference in behavior also affects the number of cycles returned from each instruction. It understandably takes more cycles to complete the jump than if it doesn't happen.

Let's give some examples. First, the simple cases without using the flags, which are 0x18 `JR i8` and 0xC3 `JP u16`.

```rust
// In cpu/opcodes.rs

// JR i8
// ----
fn jr_18(cpu: &mut Cpu) -> u8 {
    let offset = cpu.fetch() as i8 as u16;
    let mut pc = cpu.get_pc();
    pc = pc.wrapping_add(offset);
    cpu.set_pc(pc);
    3
}

// JP u16
// ----
fn jp_c3(cpu: &mut Cpu) -> u8 {
    let addr = cpu.fetch_u16();
    cpu.set_pc(addr);
    2
}
```

These are the basics of performing the jump instructions. For those that access a flag, we perform a similar operation, but don't actually make the jump unless the correct conditions are met.

```rust
// In cpu/opcodes.rs

// JP NZ, u16
// ----
fn jp_c2(cpu: &mut Cpu) -> u8 {
    let addr = cpu.fetch_u16();
    if !cpu.get_flag(Flags::Z) {
        cpu.set_pc(addr);
        4
    } else {
        3
    }
}
```

Two things to note here. The instruction name, `JP NZ, u16` uses the `N` to designate that the following flag should *not* be set. In this case, the jump will only occur if the Z flag is false. Secondly, it's very important to note that although the jump doesn't occur if the Z flag is set, the 16-bit address is *still fetched*. This is important, because fetching will alter the position of the PC register. Regardless of whether the PC is later changed for the jump, it still needs to move as part of reading the address. Failure to do so will introduce an error in the emulator's PC by two bytes whenever the jump is not executed.

With those complete, let's look at `CALL`, `RET`, and `RST`. `CALL` is the instruction used to enter a subroutine, and return to the original position when that routine finishes with `RET`. It's similar process to a `JP` instruction, but in order to know where to return to, it first pushes the PC onto the stack. The basic `CALL` instruction is 0xCD `CALL u16` and like the `JP` instructions, there are versions of `CALL` that look at whether flags are set or not, and enter the subroutine if the conditions match -- such as 0xCC `CALL Z, u16`.

```rust
// In cpu/opcodes.rs

// CALL u16
// ----
fn call_cd(cpu: &mut Cpu) -> u8 {
    let addr = cpu.fetch_u16();
    cpu.push(cpu.get_pc());
    cpu.set_pc(addr);
    6
}

// CALL Z, u16
// ----
fn call_cc(cpu: &mut Cpu) -> u8 {
    let addr = cpu.fetch_u16();
    if cpu.get_flag(Flags::Z) {
        cpu.push(cpu.get_pc());
        cpu.set_pc(addr);
        6
    } else {
        3
    }
}
```

They're basically the same operations as our `JP` functions, with the addition of pushing the PC. The `RET` instructions perform this process in reverse; popping the value off the stack and using that as the PC address. It too has version that utilize flag values.

```rust
// In cpu/opcodes.rs

// RET
// ----
fn ret_c9(cpu: &mut Cpu) -> u8 {
    let addr = cpu.pop();
    cpu.set_pc(addr);
    4
}
```

Finally, there are the `RST` instructions. We'll discuss later the purpose of these instructions, but for now they're similar to `CALL` instructions, except instead of fetching to learn where the new PC address should be, each of the instructions has a specifically assigned address. For example, 0xCF `RST 08` tells the CPU to call a function at memory address 0x0008. As far as our emulator is concerned, we will simply follow the instruction and call the subroutine there. It is up to the developer to ensure the correct behavior is waiting for us there. There are several `RST` instructions, each with an assigned hardcoded address. These hardcoded addresses are noted in the instruction's name. For example, `RST 10` will jump to 0x0010, `RST 18` to 0x0018, and so on.

```rust
// In cpu/opodes.rs

// RST 08
// ----
fn rst_cf(cpu: &mut Cpu) -> u8 {
    cpu.push(cpu.get_pc());
    cpu.set_pc(0x0008);
    4
}
```

The desired jump address is signified in the name of the instruction. In this case, `RST 08` will jump to address 0x0008. Once all of the different flag and reset instructions are added, don't forget to update the `OPCODES` table, which will look something like the following.

```rust
const OPCODES: [fn(&mut Cpu) -> u8; 256] = [
//  0x00,   0x01,   0x02,   0x03,   0x04,    0x05,    0x06,   0x07,   0x08,   0x09,   0x0A,   0x0B,   0x0C,    0x0D,    0x0E,   0x0F
    nop_00, ld_01,  ld_02,  inc_03, inc_04,  dec_05,  ld_06,  todo,   ld_08,  add_09, ld_0a,  dec_0b, inc_0c,  dec_0d,  ld_0e,  todo,   // 0x00
    todo,   ld_11,  ld_12,  inc_13, inc_14,  dec_15,  ld_16,  todo,   jr_18,  add_19, ld_1a,  dec_1b, inc_1c,  dec_1d,  ld_1e,  todo,   // 0x10
    jr_20,  ld_21,  ld_22,  inc_23, inc_24,  dec_25,  ld_26,  todo,   jr_28,  add_29, ld_2a,  dec_2b, inc_2c,  dec_2d,  ld_2e,  todo,   // 0x20
    jr_30,  ld_31,  ld_32,  inc_33, inc_34,  dec_35,  ld_36,  todo,   jr_38,  add_39, ld_3a,  dec_3b, inc_3c,  dec_3d,  ld_3e,  todo,   // 0x30
    ld_40,  ld_41,  ld_42,  ld_43,  ld_44,   ld_45,   ld_46,  ld_47,  ld_48,  ld_49,  ld_4a,  ld_4b,  ld_4c,   ld_4d,   ld_4e,  ld_4f,  // 0x40
    ld_50,  ld_51,  ld_52,  ld_53,  ld_54,   ld_55,   ld_56,  ld_57,  ld_58,  ld_59,  ld_5a,  ld_5b,  ld_5c,   ld_5d,   ld_5e,  ld_5f,  // 0x50
    ld_60,  ld_61,  ld_62,  ld_63,  ld_64,   ld_65,   ld_66,  ld_67,  ld_68,  ld_69,  ld_6a,  ld_6b,  ld_6c,   ld_6d,   ld_6e,  ld_6f,  // 0x60
    ld_70,  ld_71,  ld_72,  ld_73,  ld_74,   ld_75,   todo,   ld_77,  ld_78,  ld_79,  ld_7a,  ld_7b,  ld_7c,   ld_7d,   ld_7e,  ld_7f,  // 0x70
    add_80, add_81, add_82, add_83, add_84,  add_85,  add_86, add_87, adc_88, adc_89, adc_8a, adc_8b, adc_8c,  adc_8d,  adc_8e, adc_8f, // 0x80
    sub_90, sub_91, sub_92, sub_93, sub_94,  sub_95,  sub_96, sub_97, sbc_98, sbc_99, sbc_9a, sbc_9b, sbc_9c,  sbc_9d,  sbc_9e, sbc_9f, // 0x90
    and_a0, and_a1, and_a2, and_a3, and_a4,  and_a5,  and_a6, and_a7, xor_a8, xor_a9, xor_aa, xor_ab, xor_ac,  xor_ad,  xor_ae, xor_af, // 0xA0
    or_b0,  or_b1,  or_b2,  or_b3,  or_b4,   or_b5,   or_b6,  or_b7,  cp_b8,  cp_b9,  cp_ba,  cp_bb,  cp_bc,   cp_bd,   cp_be,  cp_bf,  // 0xB0
    ret_c0, pop_c1, jp_c2,  jp_c3,  call_c4, push_c5, add_c6, rst_c7, ret_c8, ret_c9, jp_ca,  todo,   call_cc, call_cd, adc_ce, rst_cf, // 0xC0
    ret_d0, pop_d1, jp_d2,  todo,   call_d4, push_d5, sub_d6, rst_d7, ret_d8, todo,   jp_da,  todo,   call_dc, todo,    sbc_de, rst_df, // 0xD0
    ld_e0,  pop_e1, ld_e2,  todo,   todo,    push_e5, and_e6, rst_e7, add_e8, jp_e9,  ld_ea,  todo,   todo,    todo,    xor_ee, rst_ef, // 0xE0
    ld_f0,  pop_f1, ld_f2,  todo,   todo,    push_f5, or_f6,  rst_f7, ld_f8,  ld_f9,  ld_fa,  todo,   todo,    todo,    cp_fe,  rst_ff, // 0xF0
];
```

[*Next Chapter*](10-cb-prefix.md)
