# Chapter XXIII. Optional - Creating a Debugger

[*Return to Index*](../README.md)

[*Previous Chapter*](22-constructing-background.md)

Sometimes on your emulation development journey, things don't go your way. You carefully craft the CPU, checking that every flag and register is updated just as it should. You lay out the memory map, connecting all the pieces together. You assemble the graphics, ensuring each layer is exactly right. Then you go and attempt to run the blasted thing, and nothing shows up at all. Not only is this incredibly frustrating, but it can be very obtuse where to even begin looking. Not only are you attempting to run your own emulator, but there's some other program running as well, doing who knows what.

While using an established Rust debugger like `gdb` can greatly help with this process, it's a bit cumbersome to use in this situation. When debugging an emulator, there's some items that are very useful to be able to examine quickly, such as the register contents and RAM addresses, which can be a bit of a pain to access. However, we are in complete control of the features of our emulator, and we can add any developmental tools we need. Using `gdb` as a guide, we're going to create a basic command line prompt which can examine memory and registers, set break and watchpoints, and pause execution. While both of the frontends could theoretically implement these features, I'm going to only cover how to add this to the `desktop` program. If you are only using the WebAssembly version, the basic principles are sound, but you'll need envision a useful UI yourself.

If you don't feel that this is a feature you would find yourself using, feel free to move on. This is solely meant to help with the developmental process, but we won't be relying upon it in any future steps.

## Introduction

Before we begin, let's set out the goal for our debugger. Much like `gdb`, the emulator program should run normally until paused by the user. A command prompt will then appear, where we can type in a number of commands to provide different functionality. These include:

- Setting a breakpoint for a PC value. Program execution will automatically pause when that point it hit.
- Many emulators offer the read and write breakpoints as well, but for now we'll stay with execution breakpoints only
- Listing which breakpoints have been set, and being able to delete any breakpoints.
- Printing out the register contents
- Printing out sections of RAM. If the user gives a memory address, it will print (for example) the 16 bytes starting at that address.
- Stepping to the next instruction. We want to be able to navigate through our game slowly, examining the behavior as it runs.
- Printing a disassembly. So that we don't have to constantly reference our Opcode table, we'll add some limited ability to printout the next operations.
- It should also be able to resume execution or quit if the user wants.

These are lofty goals, but with these features, the debugger will be a useful developmental tool.

Since this feature will utilize quite a bit of text output, it's not really appropriate for it to live in the `core`. That module makes no assumption on the platform that it's being run on, and things like printing out text need to be platform-dependent. Thus, we will add a new module to the `desktop` frontend which implements the debugging behavior.

Create a new `desktop/src/debug.rs` file, where we'll add a new struct.

```rust
// In desktop/src/debug.rs

pub struct Debugger {
    debugging: bool,
    breakpoints: Vec<u16>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            debugging: false,
            breakpoints: Vec::new(),
        }
    }
}
```

The two members of this struct should be pretty self-explanatory. First we'll have a boolean for whether we are currently debugging the program (as opposed to letting gameplay run normally), and the second will be a list of memory addresses for breaking upon. Before implementing anything further, let's create this object in our main executable.

```rust
// In desktop/src/main.rs
// Unchanged code omitted

mod debug;

use crate::debug::Debugger;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        println!("Please specify a ROM location: cargo run path/to/game");
        return;
    }

    let mut gbd = Debugger::new();
    let mut gb = Cpu::new();
    let filename = &args[1];
    let rom = load_rom(filename);
    gb.load_rom(&rom);

    // etc..
}

```

For now, we'll just create a new debugger object when we initialize the emulator. I've called mine `gbd` for "Game Boy Debugger", and because I thought it was a clever play on `gdb`. When the debugger's `debugging` variable is false, the emulator will execute gameplay normally.

There are two situations where it should pause execution and allow for text commands to be sent to the debugger -- either when a breakpoint has been hit, or when the user manually engages it. Before we tackle those situations, let's create the function that will handle the user commands. Back in `debug.rs`, create a new function called `debugloop`. This function will wait for the user to type a command, then print out the appropriate response. Since we want to be able to use more than one command without re-triggering the debugger, this should happen in a loop. Since this function is more or less the main loop of the debugger, we'll also need to pass in a reference to the Game Boy, since some of our commands will fetch information from it.

```rust
// In debug.rs
// Unchanged code omitted

use std::io::*;
use gb_core::cpu::*;

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                // TODO
            }
        }
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        // For Windows
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
```

The first thing the `debugloop` function does is prints some feedback to the user, then awaits its command. To accomplish this, we use the `stdin()` function, which blocks execution until the user enters some text and hits enter. That text is saved to the `input` variable, which first removes any newline characters that might be appended (taking special note for how Windows OS handles things differently), then splits the command into different words. This does enforce some assumptions into how the commands should be structured, but given this is really only meant for advanced use, it doesn't need to be very user friendly.

Inside the `match` statement will go each of the different commands, as outlined above. Let's start with the easiest, quitting. As you probably noticed, `debugloop` returns a boolean, which will signal to its caller whether we should completely quit out of the emulator or not.

```rust
// In debug.rs
// Unchanged code omitted

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "q" => {
                    return true;
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }
}
```

I'm going to mimic `gdb`'s UI and use "q" as a shortcut for "quit". You're welcome to use different abbreviations or the entire word if you prefer. Since this is a `match` statement, I've also added the default case, which should just inform the user that we didn't understand their command.

Next, another easy one, continuing execution. If we've reached this point, then the `debugging` flag has been set to true, and we will need to disable it before exiting the function as well, this time returning false so that the emulator doesn't entirely close.

```rust
// In debug.rs
// Unchanged code omitted

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "q" => {
                    return true;
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }
}
```

Now for something a bit more substantial. There are a few functions related to breakpoints that we want to implement, namely adding, removing, and listing them. We'll add functions to handle each of these, then plug the matching command into our `match` statement. Note that adding and removing breakpoints will need to listen for a parameter for the command, namely the hexadecimal memory address. When we split the user input into words, that should be the second item (if it exists), and we will need to convert it from a string into a `u16`, which we'll do with a helper function.

```rust
// In debug.rs
// Unchanged code omitted

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "b" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr);
                },
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "d" => {
                    let addr = parse_address(words[1]);
                    self.remove_breakpoint(addr);
                },
                "l" => {
                    self.print_breakpoints();
                },
                "q" => {
                    return true;
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    fn add_breakpoint(&mut self, bp: Option<u16>) {
        if let Some(addr) = bp {
            if !self.breakpoints.contains(&addr) {
                self.breakpoints.push(addr);
            }
        }
    }

    fn print_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("There are no set breakpoints");
            return;
        }
        let mut output = "Breakpoints:".to_string();
        for bp in &self.breakpoints {
            output = format!("{} 0x{:04x}", output, bp);
        }
        println!("{}", output);
    }

    fn remove_breakpoint(&mut self, bp: Option<u16>) {
        if let Some(addr) = bp {
            for i in 0..self.breakpoints.len() {
                if self.breakpoints[i] == addr {
                    self.breakpoints.remove(i);
                    break;
                }
            }
        }
    }
}

fn parse_address(input: &str) -> Option<u16> {
    let hex = u16::from_str_radix(input, 16);
    if let Ok(addr) = hex {
        Some(addr)
    } else {
        None
    }
}
```

The `parse_address` function accepts a string slice and returns an optional `u16` value, with it being set to `None` if we didn't receive a valid hex value. The `add_breakpoint` and `remove_breakpoint` functions need to ensure that it is a `Some` value before using it. Their functionality is also pretty straight-forward. `add_breakpoint` checks that there are no duplicates before adding the address to the vector, while `remove_breakpoint` loops through the items and removes the matching address if one is found. I didn't bother printing an error message if one wasn't found, but you can if you prefer. The `print_breakpoints` function loops through each address and does a bit of string formatting to make things a bit more appealing to the user. I use the "b", "d" and "l" shortcuts for these commands, which does differ slightly from `gdb`'s syntax.

Next, let's print some items from the Game Boy itself. As mentioned, we want to be able to print out the contents of the registers, as well as values from RAM. Rather than print the entire RAM content, we'll have the user specify an address and print the 16 bytes starting from that address (you're welcome to print more if you like, or to specify a range to print).

```rust
// In debug.rs
// Unchanged code omitted

use std::cmp::min;

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "b" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr);
                },
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "d" => {
                    let addr = parse_address(words[1]);
                    self.remove_breakpoint(addr);
                },
                "l" => {
                    self.print_breakpoints();
                },
                "p" => {
                    let addr = parse_address(words[1]);
                    self.print_ram(&gb, addr);
                },
                "q" => {
                    return true;
                },
                "reg" => {
                    self.print_registers(&gb);
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    fn print_ram(&self, gb: &Cpu, mem: Option<u16>) {
        if let Some(addr) = mem {
            // Print 16 bytes starting at addr
            let end = min(addr + 16, 0xFFFF);
            let mut output = String::new();
            for i in addr..end {
                let val = gb.read_ram(i);
                output = format!("{} {:02x}", output, val);
            }
            println!("0x{:04x}: {}", addr, output);
        }

    }

    fn print_registers(&self, gb: &Cpu) {
        let mut output = format!("PC: 0x{:04x}\n", gb.get_pc());
        output = format!("{}SP: 0x{:04x}\n", output, gb.get_r16(Regs16::SP));
        output = format!("{}AF: 0x{:04x}\n", output, gb.get_r16(Regs16::AF));
        output = format!("{}BC: 0x{:04x}\n", output, gb.get_r16(Regs16::BC));
        output = format!("{}DE: 0x{:04x}\n", output, gb.get_r16(Regs16::DE));
        output = format!("{}HL: 0x{:04x}\n", output, gb.get_r16(Regs16::HL));
        println!("{}", output);
    }
}
```

If you haven't seen it before, the `{:04x}` notation is for formatting a string into a four digit hexadecimal value in Rust. These functions use our Game Boy's `get_r16` and `read_ram` functions to access emulator data directly. For `print_ram`, we append new values in a loop (ensuring we don't go past 0xFFFF), while `print_registers` grabs the values from all our different 16-bit registers. These have been set to "p" (for "print") and "reg", respectively.

Next, we'll need to allow the emulator to execute the next instruction. While this sounds daunting, it's actually very simple, as everything the Game Boy does in a single instruction is handled via the `tick` function. We'll set that to "n" (for "next") and also print out the PC value for a bit of user feedback.

```rust
// In debug.rs
// Unchanged code omitted

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "b" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr);
                },
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "d" => {
                    let addr = parse_address(words[1]);
                    self.remove_breakpoint(addr);
                },
                "l" => {
                    self.print_breakpoints();
                },
                "n" => {
                    gb.tick();
                    println!("PC: 0x{:04x}", gb.get_pc());
                },
                "p" => {
                    let addr = parse_address(words[1]);
                    self.print_ram(&gb, addr);
                },
                "q" => {
                    return true;
                },
                "reg" => {
                    self.print_registers(&gb);
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }
}
```

The most complicated functionality we're going to add to the debugger is to print out some disassembly. The goal here is for the emulator to look at the PC and the next (for example) five instructions, and to print out both the name of that opcode, but also what parameters that opcode is going to use, if any. For example, if the byte at PC was 01, the disassembly would print out "LD BC, u16". We could get really fancy with this if we wanted, formatting the inputs properly, signifying flag values, and so on, but I'm going to keep this pretty simple. Also for the sake of brevity, I'm not going to implement the 0xCB table, only the main one, although it should be pretty easy to add.

As you might have noticed, we don't actually store the names of the instructions anywhere, only in the function comments and in the opcode table of this book. We need to know how many bytes each instruction fetches, so we stay correctly aligned, which we also don't store anywhere. Thus, we will need to add lookup tables for both of these items. Implementation will then just require looking up the correct instruction and formatting in a loop.

```rust
// In debug.rs
// Unchanged code omitted

const OPCODE_NAMES: [&str; 0x100] = [
    "NOP",          "LD BC, u16",   "LD (BC), A",   "INC BC",       "INC B",        "DEC B",        "LD B, u8",     "RLCA",         // $00
    "LD (u16), SP", "ADD HL, BC",   "LD A, (BC)",   "DEC BC",       "INC C",        "DEC C",        "LD C, u8",     "RRCA",         // $08
    "STOP",         "LD DE, u16",   "LD (DE), A",   "INC DE",       "INC D",        "DEC D",        "LD D, u8",     "RLA",          // $10
    "JR i8",        "ADD HL, DE",   "LD A, (DE)",   "DEC DE",       "INC E",        "DEC E",        "LD E, u8",     "RRA",          // $18
    "JR NZ, i8",    "LD HL, u16",   "LD (HL+), A",  "INC HL",       "INC H",        "DEC H",        "LD H, u8",     "DAA",          // $20
    "JR Z, i8",     "ADD HL, HL",   "LD A, (HL+)",  "DEC HL",       "INC L",        "DEC L",        "LD L, u8",     "CPL",          // $28
    "JR NC, i8",    "LD SP, u16",   "LD (HL-), A",  "INC SP",       "INC (HL)",     "DEC (HL)",     "LD (HL), u8",  "SCF",          // $30
    "JR C, i8",     "ADD HL, SP",   "LD A, (HL-)",  "DEC SP",       "INC A",        "DEC A",        "LD A, u8",     "CCF",          // $38
    "LD B, B",      "LD B, C",      "LD B, D",      "LD B, E",      "LD B, H",      "LD B, L",      "LD B, (HL)",   "LD B, A",      // $40
    "LD C, B",      "LD C, C",      "LD C, D",      "LD C, E",      "LD C, H",      "LD C, L",      "LD C, (HL)",   "LD C, A",      // $48
    "LD D, B",      "LD D, C",      "LD D, D",      "LD D, E",      "LD D, H",      "LD D, L",      "LD D, (HL)",   "LD D, A",      // $50
    "LD E, B",      "LD E, C",      "LD E, D",      "LD E, E",      "LD E, H",      "LD E, L",      "LD E, (HL)",   "LD E, A",      // $58
    "LD H, B",      "LD H, C",      "LD H, D",      "LD H, E",      "LD H, H",      "LD H, L",      "LD H, (HL)",   "LD H, A",      // $60
    "LD L, B",      "LD L, C",      "LD L, D",      "LD L, E",      "LD L, H",      "LD L, L",      "LD L, (HL)",   "LD L, A",      // $68
    "LD (HL), B",   "LD (HL), C",   "LD (HL), D",   "LD (HL), E",   "LD (HL), H",   "LD (HL), L",   "HALT",         "LD (HL), A",   // $70
    "LD A, B",      "LD A, C",      "LD A, D",      "LD A, E",      "LD A, H",      "LD A, L",      "LD A, (HL)",   "LD A, A",      // $78
    "ADD A, B",     "ADD A, C",     "ADD A, D",     "ADD A, E",     "ADD A, H",     "ADD A, L",     "ADD A, (HL)",  "ADD A, A",     // $80
    "ADC A, B",     "ADC A, C",     "ADC A, D",     "ADC A, E",     "ADC A, H",     "ADC A, L",     "ADC A, (HL)",  "ADC A, A",     // $88
    "SUB B",        "SUB C",        "SUB D",        "SUB E",        "SUB H",        "SUB L",        "SUB (HL)",     "SUB A",        // $90
    "SBC B",        "SBC C",        "SBC D",        "SBC E",        "SBC H",        "SBC L",        "SBC (HL)",     "SBC A",        // $98
    "AND B",        "AND C",        "AND D",        "AND E",        "AND H",        "AND L",        "AND (HL)",     "AND A",        // $A0
    "XOR B",        "XOR C",        "XOR D",        "XOR E",        "XOR H",        "XOR L",        "XOR (HL)",     "XOR A",        // $A8
    "OR B",         "OR C",         "OR D",         "OR E",         "OR H",         "OR L",         "OR (HL)",      "OR A",         // $B0
    "CP B",         "CP C",         "CP D",         "CP E",         "CP H",         "CP L",         "CP (HL)",      "CP A",         // $B8
    "RET NZ",       "POP BC",       "JP NZ, u16",   "JP u16",       "CALL NZ, u16", "PUSH BC",      "AND A, u8",    "RST 00",       // $C0
    "RET Z",        "RET",          "JP Z, u16",    "PREFIX CB",    "CALL Z, u16",  "CALL u16",     "ADC A, u8",    "RST 08",       // $C8
    "RET NC",       "POP DE",       "JP NC, u16",   "INVALID",      "CALL NC, u16", "PUSH DE",      "SUB u8",       "RST 10",       // $D0
    "RET C",        "RETI",         "JP C, u16",    "INVALID",      "CALL C, u16",  "INVALID",      "SBC A, u8",    "RST 18",       // $D8
    "LDH (a8), A",  "POP HL",       "LD (C), A",    "INVALID",      "INVALID",      "PUSH HL",      "AND u8",       "RST 20",       // $E0
    "ADD SP, i8",   "JP (HL)",      "LD (u16), A",  "INVALID",      "INVALID",      "INVALID",      "XOR u8",       "RST 28",       // $E8
    "LDH A, (a8)",  "POP AF",       "LD A, (C)",    "DI",           "INVALID",      "PUSH AF",      "OR u8",        "RST 30",       // $F0
    "LD HL, SP+i8", "LD SP, HL",    "LD A, (u16)",  "EI",           "INVALID",      "INVALID",      "CP u8",        "RST 38"        // $F8
];

const OPCODE_LENGTH: [u8; 0x100] = [
    1, 3, 1, 1, 1, 1, 2, 1, 3, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1, 2, 3, 1, 1, 1, 1, 2, 1, 2, 1, 1, 1, 1, 1, 2, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 3, 3, 3, 1, 2, 1, 1, 1, 3, 1, 3, 3, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1, 1, 1, 3, 1, 3, 1, 2, 1,
    2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1, 2, 1, 2, 1, 1, 1, 2, 1, 2, 1, 3, 1, 1, 1, 2, 1,
];

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "b" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr);
                },
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "d" => {
                    let addr = parse_address(words[1]);
                    self.remove_breakpoint(addr);
                },
                "disass" => {
                    self.disassemble(&gb);
                },
                "l" => {
                    self.print_breakpoints();
                },
                "n" => {
                    gb.tick();
                    println!("PC: 0x{:04x}", gb.get_pc());
                },
                "p" => {
                    let addr = parse_address(words[1]);
                    self.print_ram(&gb, addr);
                },
                "q" => {
                    return true;
                },
                "reg" => {
                    self.print_registers(&gb);
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    fn disassemble(&self, gb: &Cpu) {
        let mut pc = gb.get_pc();
        for _ in 0..5 {
            let op = gb.read_ram(pc) as usize;
            let name = OPCODE_NAMES[op];
            let len = OPCODE_LENGTH[op] as u16;
            let mut printout = format!("0x{:04x} | {} |", pc, name);
            for i in 0..len {
                let arg = gb.read_ram(pc + i);
                printout = format!("{} {:02x}", printout, arg);
            }
            println!("{}", printout);
            pc += len;
        }
    }

}
```

I couldn't think of a better abbreviation than "disass", which is what `gdb` supports. The `disassemble` function grabs the next five instructions in a loop, doing some formatting so they look nice for the user, and using the table values to ensure everything is correct (except for the 0xCB table, sorry).

The last thing we're going to add is a help function, which will printout the syntax for these operations, in case we forget. I'm also going to take a moment to add a few public functions that we will need back in `main.rs`.

```rust
// In debug.rs
// Unchanged code omitted

impl Debugger {
    pub fn debugloop(&mut self, gb: &mut Cpu) -> bool {
        loop {
            print!("(gbd) ");
            stdout().flush().unwrap();

            let mut input = String::new();
            let stdin = stdin();
            stdin.read_line(&mut input).expect("Unable to parse user input");
            trim_newline(&mut input);
            let words: Vec<&str> = input.split(' ').collect();

            match words[0] {
                "b" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr);
                },
                "c" => {
                    self.debugging = false;
                    return false;
                },
                "d" => {
                    let addr = parse_address(words[1]);
                    self.remove_breakpoint(addr);
                },
                "disass" => {
                    self.disassemble(&gb);
                },
                "h" => {
                    self.print_help();
                },
                "l" => {
                    self.print_breakpoints();
                },
                "n" => {
                    gb.tick();
                    println!("PC: 0x{:04x}", gb.get_pc());
                },
                "p" => {
                    let addr = parse_address(words[1]);
                    self.print_ram(&gb, addr);
                },
                "q" => {
                    return true;
                },
                "reg" => {
                    self.print_registers(&gb);
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    pub fn check_breakpoints(&mut self, pc: u16) {
        if self.breakpoints.contains(&pc) {
            self.debugging = true;
        }
    }

    pub fn is_debugging(&self) -> bool {
        self.debugging
    }

    fn print_help(&self) {
        let help = "'b XXXX' to add a breakpoint at that address\n\
                    'c' to continue execution\n\
                    'd XXXX' to delete breakpoint at that address\n\
                    'disass' to show disassembly of next 5 instructions\n\
                    'h' to print this message\n\
                    'l' to print list of breakpoints\n\
                    'n' to execute the next instruction\n\
                    'p XXXX' to print 16 bytes at that address\n\
                    'q' to quit debugging\n\
                    'reg' to print register contents\n";
        println!("{}", help);
    }

    pub fn print_info(&self) {
        println!("gbd - The Game Boy Debugger");
        println!();
    }

    pub fn set_debugging(&mut self, debug: bool) {
        self.debugging = debug;
    }
}
```

These new public functions are a getter and setter for the `debugging` flag, a `check_breakpoints` function to check if a memory address is in the breakpoints list (and break if it is), and a function to print an introductory message to the user.

## Connecting to main.rs

With `debug.rs` handling all the functionality for us, we only need to handle entering and leaving the debugger inside of `main.rs`. As mentioned, this will be done whenever a breakpoint is tripped (as determined by our `check_breakpoints` function), and if the user manually engages it. For the former, we currently allow the emulator to `tick` repeatedly, only interrupting when it is time to render a frame. We'll replace that with a function that also checks for breakpoints after each tick, and enters the `debugloop` if there's a match.

```rust
// In main.rs
// Unchanged code omitted

use std::process::exit;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        println!("Please specify a ROM location: cargo run path/to/game");
        return;
    }

    let mut gbd = Debugger::new();
    let mut gb = Cpu::new();
    let filename = &args[1];
    let rom = load_rom(filename);
    gb.load_rom(&rom);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("My Game Boy Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered().opengl().build().unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut events = sdl_context.event_pump().unwrap();
    'gameloop: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit{..} |
                Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
                _ => {}
            }
        }

        // Keep ticking until told to stop
        tick_until_draw(&mut gb, &mut gbd);
        let frame = gb.render();
        draw_screen(&frame, &mut canvas);
    }
}

fn tick_until_draw(gb: &mut Cpu, gbd: &mut Debugger) {
    loop {
        let render = gb.tick();

        gbd.check_breakpoints(gb.get_pc());
        if gbd.is_debugging() {
            gbd.print_info();
            let quit = gbd.debugloop(gb);
            if quit {
                exit(0);
            }
        }

        if render {
            break;
        }
    }
}
```

`tick_until_draw` also runs in a loop, only breaking when the `tick` function informs us we need to proceed to the rendering stage. Before that though, we use the value in the PC to check if it matches any breakpoint values, and if it has, enter into the debug loop. The `debugloop` itself can break on two situations -- if the user decides to continue execution, in which case we should proceed as normal, of if they want to quit, in which case we will use the `exit` function to kill the program entirely. Being able to quit saves some time over continuing then hitting escape to quit.

This entry path only works if there are breakpoints to hit upon, but we'll need a way to enter into the debugger for the first time. For that, we'll add a keyboard shortcut, in this case the space bar (feel free to set it to whatever you wish).

```rust
// In main.rs
// Unchanged code omitted

// Inside of main()
'gameloop: loop {
    for event in events.poll_iter() {
        match event {
            Event::Quit{..} |
            Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                break 'gameloop;
            },
            Event::KeyDown{keycode: Some(Keycode::Space), ..} => {
                gbd.set_debugging(true);
            },
            _ => {}
        }
    }

    // Keep ticking until told to stop
    tick_until_draw(&mut gb, &mut gbd);
    let frame = gb.render();
    draw_screen(&frame, &mut canvas);
}
```

If the user presses the space bar at any time, then the debugger is activated. Note that the key input is only listened to during VBlank, as otherwise the emulation is inside of `tick_until_draw`. This happens 60 times per second though, which is good enough for a utility tool.

A good debugger can greatly improve development experience, and the one we've added here should provide enough tools to inspect what exactly the Game Boy itself is doing. There are extra features that could be added, such as a trace printout, watchpoints, read and write breakpoints, and more; but I think this is a good introduction to how debuggers work in conjunction with emulation. This is the limit that we'll explore this within this tutorial, but I encourage you to utilize it and expand upon it as we continue with development.

[*Next Chapter*](24-background-viewport.md)
