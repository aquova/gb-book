use std::cmp::min;
use std::io::*;

use gb_core::cpu::*;

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

#[derive(PartialEq, Clone, Copy)]
enum BreakpointTypes {
    READ,
    WRITE,
    EXEC,
}

#[derive(PartialEq)]
struct Breakpoint {
    addr: u16,
    kind: BreakpointTypes,
}

impl Breakpoint {
    pub fn new(addr: u16, kind: BreakpointTypes) -> Self {
        Self { addr, kind }
    }

    pub fn get_addr(&self) -> u16 {
        self.addr
    }

    pub fn get_type(&self) -> BreakpointTypes {
        self.kind
    }
}

pub struct Debugger {
    debugging: bool,
    breakpoints: Vec<Breakpoint>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            debugging: false,
            breakpoints: Vec::new(),
        }
    }

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
                    self.add_breakpoint(addr, BreakpointTypes::EXEC);
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
                    self.disassemble(gb);
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
                    self.print_ram(gb, addr);
                },
                "q" => {
                    return true;
                },
                "r" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr, BreakpointTypes::READ);
                },
                "reg" => {
                    self.print_registers(&gb);
                },
                "w" => {
                    let addr = parse_address(words[1]);
                    self.add_breakpoint(addr, BreakpointTypes::WRITE);
                },
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    fn add_breakpoint(&mut self, bp: Option<u16>, kind: BreakpointTypes) {
        if let Some(addr) = bp {
            let breakpoint = Breakpoint::new(addr, kind);
            if !self.breakpoints.contains(&breakpoint) {
                self.breakpoints.push(breakpoint);
            }
        }
    }

    pub fn check_exec_breakpoints(&mut self, pc: u16) {
        for bp in &self.breakpoints {
            if bp.get_addr() == pc && bp.get_type() == BreakpointTypes::EXEC {
                self.debugging = true;
                break;
            }
        }
    }

    pub fn check_read_breakpoints(&mut self, addr: u16) {
        for bp in &self.breakpoints {
            if bp.get_addr() == addr && bp.get_type() == BreakpointTypes::READ {
                self.debugging = true;
                break;
            }
        }
    }

    pub fn check_write_breakpoints(&mut self, addr: u16) {
        for bp in &self.breakpoints {
            if bp.get_addr() == addr && bp.get_type() == BreakpointTypes::WRITE {
                self.debugging = true;
                break;
            }
        }
    }

    fn disassemble(&self, gb: &mut Cpu) {
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

    pub fn is_debugging(&self) -> bool {
        self.debugging
    }

    fn print_breakpoints(&self) {
        if self.breakpoints.is_empty() {
            println!("There are no set breakpoints");
            return;
        }
        let mut output = "Breakpoints:".to_string();
        for bp in &self.breakpoints {
            output = format!("{} 0x{:04x}", output, bp.get_addr());
        }
        println!("{}", output);
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
    }

    fn print_ram(&self, gb: &mut Cpu, mem: Option<u16>) {
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

    fn remove_breakpoint(&mut self, bp: Option<u16>) {
        if let Some(addr) = bp {
            for i in 0..self.breakpoints.len() {
                if self.breakpoints[i].get_addr() == addr {
                    self.breakpoints.remove(i);
                    break;
                }
            }
        }
    }

    pub fn set_debugging(&mut self, debug: bool) {
        self.debugging = debug;
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

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        // For Windows
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
