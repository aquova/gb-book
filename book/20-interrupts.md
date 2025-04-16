# Chapter XX. Interrupts

[*Return to Index*](../README.md)

[*Previous Chapter*](19-ppu-modes.md)

In the last chapter, we implemented the rendering modes of the system, so we know when we have a new frame to render. This implementation is not quite complete, as another mechanism is engaged when the system enters VBlank mode -- an "interrupt". While implementing our CPU instructions, we encountered several that dealt with interrupts, a mechanism for pausing normal CPU operation to execute some special functions. These functions are for things like handling button input or rendering the screen, tasks that need to occur either on demand or on a fixed schedule, regardless of what the CPU might be doing at that time. The Game Boy has five different interrupt types in total, each of which has an associated "vector", or memory address, that the CPU will jump to when the interrupt conditions are met.

| Name   | Vector | Details                                                         |
| ------ | ------ | --------------------------------------------------------------- |
| VBLANK | 0x0040 | When the PPU enters VBlank mode (rendering is restarting)       |
| STAT   | 0x0048 | Conditions are met according to the values in the STAT register |
| TIMER  | 0x0050 | When the CPU timer fires                                        |
| SERIAL | 0x0058 | When information traveling across a link cable is ready         |
| JOYPAD | 0x0060 | When the player presses a button                                |

If you aren't familiar with some of these concepts, don't worry. We'll cover what the STAT register is in the next chapter, we'll get to the CPU timer and button inputs in time, and Game Boy to Game Boy communication is beyond this scope of this tutorial, so we won't cover very much of the serial interrupt. Each of these five interrupts covers situations that we would think of happening asynchronously to the CPU, but given the limitations of its hardware, need to happen synchronously.

When a subsystem decides it needs to interrupt, it sets a corresponding bit in a special memory address, `IF` ("Interrupt Flag"), located at 0xFF0F. When the CPU sees this, it performs a similar process to when calling a subroutine -- it pushes the PC onto the stack, jumps to the corresponding interrupt vector, then executes the code there. It is up to the game developer to ensure that there is a correctly written program waiting for the Game Boy, and that it correctly returns execution to where it began.

| 7 - 5  |   4    |   3    |   2   |  1   |   0    |
| ------ | ------ | ------ | ----- | ---- | ------ |
| Unused | JOYPAD | SERIAL | TIMER | STAT | VBLANK |

Defined bits for RAM address 0xFF0F - the Interrupt Flag register

While the systems may signal they want an interrupt, the Game Boy doesn't necessarily have to listen. If you recall, there were some instructions that enabled/disabled interrupts entirely, such as `EI` and `DI`. These toggled the interrupt "master enable", which we represented with the `irq_enabled` boolean in the `Cpu` object. If that is set to false, no interrupts can occur at all. This is a bit of a blunt instrument, and fortunately (for the game developer, not for the emulator developer) there is a second method to disable individual interrupts. This is the `IE` register ("Interrupt Enable") at address 0xFFFF, the very last byte of RAM. This register strongly resembles `IF` where only if the corresponding bit flag is set can that interrupt be allowed to trigger. It should be noted that if an interrupt wants to run but is unable to, it isn't automatically cleared. It will remain waiting until the time where the corresponding interrupt is allowed again, or it no longer wishes to execute.

| 7 - 5  |   4    |   3    |   2   |  1   |   0    |
| ------ | ------ | ------ | ----- | ---- | ------ |
| Unused | JOYPAD | SERIAL | TIMER | STAT | VBLANK |

Defined bits for RAM address 0xFFFF - the Interrupt Enable register

While the interrupts can be sent from a few different systems, they're handled by the CPU, and thus we shall define their behavior in `cpu/mod.rs`. First, let's create an enum for the different interrupt types, and add some constants for the two interrupt memory registers. We'll also create a function to retrieve the corresponding vector for each of the interrupt types, as detailed in the table above.

```rust
// In cpu/mod.rs

// Unchanged code omitted

const IF: u16 = 0xFF0F;
const IE: u16 = 0xFFFF;

#[derive(Copy, Clone)]
pub enum Interrupts {
    Vblank,
    Stat,
    Timer,
    Serial,
    Joypad,
}

impl Interrupts {
    pub fn get_vector(&self) -> u16 {
        match *self {
            Interrupts::Vblank => { 0x0040 },
            Interrupts::Stat =>   { 0x0048 },
            Interrupts::Timer =>  { 0x0050 },
            Interrupts::Serial => { 0x0058 },
            Interrupts::Joypad => { 0x0060 },
        }
    }
}
```

While we don't know about where most of these interrupts actually originate from, there is one we're familiar with already -- VBlank. When the PPU state machine reaches the VBlank state, it's time to render a frame. We're handing that already in `tick` by returning a boolean for the frontend to use, but the CPU will also go through the process of the VBlank interrupt at this point; an excellent opportunity for us to see how the process works. Since the interrupt could happen at any time, we will need to check it after every instruction execution, in the CPU's `tick` function. We'll create a `check_irq` function that looks at the `IF`, `IE`, and master interrupt controls and decides which (if any) of the interrupts need to happen. One thing to note is that if more than one interrupt wants to happen at once, there's a priority for which one gets chosen, which is the order I've listed them in the table above (from VBlank as the most important to Joypad as the least important).

```rust
// In cpu/mod.rs

// Unchanged code omitted

const IRQ_PRIORITIES: [Interrupts; 5] = [
    Interrupts::Vblank,
    Interrupts::Stat,
    Interrupts::Timer,
    Interrupts::Serial,
    Interrupts::Joypad,
];

impl Cpu {
    pub fn tick(&mut self) -> bool {
        let mut draw_time = false;
        let cycles = if self.halted { 1 } else { opcodes::execute(self) };
        let ppu_result = self.bus.update_ppu(cycles);
        match ppu_result.lcd_result {
            Lcd_Results::RenderFrame => {
                self.enable_irq_type(Interrupts::Vblank, true);
                draw_time = true;
            },
            _ => {},
        }
        if let Some(irq) = self.check_irq() {
            self.trigger_irq(irq);
        }
        draw_time
    }

    fn check_irq(&mut self) -> Option<Interrupts> {
        if !self.irq_enabled && !self.halted {
            return None;
        }

        let if_reg = self.read_ram(IF);
        let ie_reg = self.read_ram(IE);
        let irq_flags = if_reg & ie_reg;
        for (i, irq) in IRQ_PRIORITIES.iter().enumerate() {
            if irq_flags.get_bit(i as u8) {
                return Some(*irq);
            }
        }
        None
    }

    fn enable_irq_type(&mut self, irq: Interrupts, enabled: bool) {
        // TODO
    }

    fn trigger_irq(&mut self, irq: Interrupts) {
        // TODO
    }
}
```

`tick` still returns a boolean indicating whether the `RenderFrame` result was given, but it also now activates the VBlank interrupt at that time as well, using an `enable_irq_type` function we'll write in a moment. The `check_irq` loops through each of the interrupts, in priority order, and checks if the corresponding bits in the `IF` and `IE` registers are set. If so, that's the interrupt that needs to occur. We'll call this in our `tick` function after we update the PPU state, and if there's a match, we'll pass it along to a `trigger_irq` function, which will do the handling. To handle an interrupt, the system is always taken out of halt mode (even if the master flag isn't set), the corresponding interrupt flag in `IF` is switched off, and the CPU moves to the interrupt vector, storing its previous PC onto the stack.

```rust
// In cpu/mod.rs

// Unchanged code omitted

impl Cpu {
    fn enable_irq_type(&mut self, irq: Interrupts, enabled: bool) {
        let mut if_reg = self.read_ram(IF);
        match irq {
            Interrupts::Vblank =>   { if_reg.set_bit(0, enabled) },
            Interrupts::Stat =>     { if_reg.set_bit(1, enabled) },
            Interrupts::Timer =>    { if_reg.set_bit(2, enabled) },
            Interrupts::Serial =>   { if_reg.set_bit(3, enabled) },
            Interrupts::Joypad =>   { if_reg.set_bit(4, enabled) },
        }
        self.write_ram(IF, if_reg);
    }

    fn trigger_irq(&mut self, irq: Interrupts) {
        // We always wake up from HALT if there's a waiting interrupt, even if the master control is turned off
        self.halted = false;

        if self.irq_enabled {
            self.irq_enabled = false;

            let vector = irq.get_vector();
            self.push(self.pc);
            self.set_pc(vector);

            self.enable_irq_type(irq, false);
        }
    }
}
```

This is the end of the interrupt structure for now. It might seem odd to end here, but this is infrastructure that we will need later when we want to accept button presses or render the screen. Speaking of that last item, let's return to the PPU and look at its own control registers, including one that directly deals with interrupts.

[*Next Chapter*](21-control-registers.md)
