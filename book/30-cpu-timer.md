# XXX. Timers

[*Return to Index*](../README.md)

[*Previous Chapter*](29-wram.md)

If you fire up a game of *Tetris*, you should by this point be able to get past the copyright screen, through the menus, and begin some actual gameplay. You can move the blocks left and right as they fall, completed lines are removed, but you'll notice something odd pretty quickly. *Tetris* normally has seven different possible pieces that are randomly selected to fall. When using our emulator there is... only one. The only piece that is ever chosen to fall is the 2x2 square piece<sup>1</sup>. This seems like a baffling and potentially severe bug. Is there an issue with our opcode implementation somewhere?

Fortunately, it's simpler than that. While internal elements of the hardware have utilized CPU cycles to keep things in sync, there is also a mechanism to allow a developer access to monitor how much time has passed. This is known as the *Timer*, and it increments with the passage of CPU cycles and is assigned several addresses in RAM, from which several pieces of useful information can be read. It is the driver of the Timer interrupt, which was included in our interrupt handling, but never had a chance to be handled until now. It's also commonly used as a source of pseudo-random number generation (RNG). Nearly everything we've dealt with thus far is deterministic, meaning that every time the game is run, it executes exactly the same. This is ideal under most conditions, but many games benefit from some amount of RNG. Basing it upon the passage of time is a simple yet effect way to add randomness to a game. It's very difficult for a human player to exploit a game in this fashion.

<sup>1</sup> I would argue this is an improvement for someone with my skill level.

The timer data can be accessed by developers via four bytes in RAM from 0xFF04 through 0xFF07, which lies within the I/O section. Each of these four registers have a distinct purpose that update on their own schedule. The Game Boy Emulator Development Guide has an [excellent article]("https://hacktix.github.io/GBEDG/timers/") which goes over the usage of the timers in good detail. Rather than poorly paraphrase their explanation, I would strongly suggest that you read over their descriptions. The following is their description of the timer registers, in case the original resource becomes unavailable in the future.

```
Timer Register Overview

$FF04 - Divider Register (DIV)
To the software, the DIV register appears as an 8-bit register which is incremented every 256 T-cycles. However, the DIV register is the fundamental basis of the entire timer system. Internally, the DIV register is a 16-bit counter which is incremented every single T-cycle, only the upper 8 bits are mapped to memory. The DIV register can be read from at any point in time. However, writing to $FF04 resets the whole internal 16-bit DIV counter to 0 instantly.

$FF05 - Timer Counter (TIMA)
While the aforementioned DIV register is the “core” of the whole timer system, the TIMA register is the most commonly used software interface. It can be configured using the TAC register at $FF07 to increment at different rates. However, keep in mind that all TIMA increments are bound to the DIV register, as is explained in the Timer Operation section. This register is fully read- and write-capable.

$FF06 - Timer Modulo (TMA)
The TIMA register can only store 8-bit values. If it overflows, it is reset to the value in this TMA register. Again, there are some oddities to the timing of this operation which is explained in the TIMA Overflow Behavior section of this document. Like TIMA, this register is fully readable and writeable.

$FF07 - Timer Control (TAC)
As the name may suggest, this register controls the behavior of the TIMA register and is also fully readable and writeable. The structure of this register is as follows:
Bit 2 : Timer Enable - This bit should be responsible for enabling/disabling increments of the TIMA register, however, it does not quite work as expected all the time. Details can be found further below.
Bits 1-0 : Clock Select - These two bits determine at which rate the TIMA register should be incremented, going by the following table:
0b00 : CPU Clock / 1024
0b01 : CPU Clock / 16
0b10 : CPU Clock / 64
0b11 : CPU Clock / 256
For example: A value of 0b101 would cause the TIMA register to be incremented every 16 T-cycles.
```

To summarize, there are four different timer registers, DIV, TIMA, TMA, and TAC, which are stored as a byte each between 0xFF04-0xFF07 respectively. DIV increases by one every 256 cycles, which forms the backbone for when the other registers update, and is reset to 0 if ever written to. TIMA also updates whenever DIV does, but operates based on the settings stored in the TAC register and when overflows is reset to the value stored in the TMA register. It sounds complicated on paper, but isn't too complex when laid out.

## M-Cycles vs T-Cycles

In the quoted article, they mention something called "T-Cycles" a few times, a concept that we've touched on previously but haven't really gone into detail. Essentially, the internal crystal clock ticks at roughly 4 MHz, these are the hardware T-Cycles. However, even the simplest `NOP` instruction cannot be completed in a single clock cycle, all instructions are completed as a multiple of four T-Cycles. These are the M-Cycles, and they are what we used as the return values for our opcodes. If you want to have the utmost accuracy in your emulator, you need to do things on the T-Cycle level, since it has the higher precision. For most of our implementation, we have simply dealt with M-Cycles, as this is good enough for the vast majority of titles. However, the timer operates on a higher granularity, meaning that every T-Cycle something could potentially happen. Fortunately, this is a relatively straight-forward thing to support, but one we do need to be conscious of.

## Implementation

Let's begin by creating a new `timer.rs` file to contain our timer code. In it, we will define a new struct to hold each of the four registers we mentioned above, as well as some constants that will be of use later

```rust
// In timer.rs

pub const DIV: u16      = 0xFF04;
pub const TIMA: u16     = 0xFF05;
pub const TMA: u16      = 0xFF06;
pub const TAC: u16      = 0xFF07;

pub struct Timer {
    counter: u8,
    div: u8,
    tima: u8,
    tma: u8,
    tac: u8,
    tima_cooldown: u8,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            counter: 0,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            tima_cooldown: 0,
        }
    }
}
```

We have a `u8` value for each of the four registers -- DIV, TIMA, TMA, and TAC -- along with constants defining each of their memory addresses. There are also two other `u8` member variables, whose existence is mentioned in the quoted section above. The `DIV` register only increases once every 256 cycles, the progress of which we will store in the `counter` variable.

Next, we'll add some simple API to query our `Timer` object for the register values. This will look very similar to our many RAM read/write functions.

```rust
// In timer.rs

// Unchanged code omitted

impl Timer {
    pub fn read_timer(&self, addr: u16) -> u8 {
        match addr {
            DIV => self.div,
            TIMA => self.tima,
            TMA => self.tma,
            TAC => self.tac,
            _ => unreachable!("Trying to read a non-timer register")
        }
    }

    pub fn write_timer(&mut self, addr: u16, val: u8) {
        match addr {
            DIV => {
                self.div = 0
            },
            TIMA => {
                self.tima = val;
                self.tima_cooldown = 0;
            },
            TMA => {
                self.tma = val
            },
            TAC => {
                self.tac = val
            },
            _ => unreachable!("Trying to write to a non-timer register")
        }
    }
}
```

Most of this is pretty self-explanatory, and nothing we haven't seen before. The one exception is when we write to the `TIMA` register. In that case, we also reset the mysterious `tima_cooldown` variable to zero. We'll discuss the exact purpose of it in a moment, but as the name implies, it's a companion variable to `tma`.

### Timer Ticking

Onto the main function of the struct, yet another `tick` function. Here, we accept the number of cycles that have elapsed as a paraamter (which should give a hint as to where this function will be hooked in). For each cycle, we'll need to increment the counter, checking if it has overflowed. Only during an overflow are our registers affected, so we can bail early if not.

```rust
// In timer.rs

// Unchanged code omitted

impl Timer {
    pub fn tick(&mut self, m_cycles: u8) -> bool {
        let mut interrupt = false;
        let t_cycles = 4 * m_cycles;
        for _ in 0..t_cycles {
            let (counter, overflow) = self.counter.overflowing_add(1);
            self.counter = counter;
            if !overflow {
                continue;
            }
            // TODO
        }

        interrupt
    }
}
```

As I mentioned above, this is one of the rare cases where we're going to be operating on a T-Cycle basis. We'll pass in the M-Cycle value as an input, but multiply it by four to get the T-Cycle count. This means that for every call of the `tick` function, the counter will increase four times for every M-Cycle that passes.

Let's now describe now what happens to each of the four registers. `DIV` is the easiest to understand. Now that the `counter` has overflowed, it simply increases by one, although we need to take care to allow it to wrap around if it too needs to overflow. `TMA` and `TAC` too are fairly simple to implement, as they don't increase alongside the counter. Instead, they simply hold values that will be used when calculating the advance of the fourth, and most complicated, register -- `TIMA`.

### TAC

Let's put `TIMA` aside for now and focus just on `TAC`. It's understandable that the game developers might want the ability to alter how often their timer fires an interrupt, and one of the means they have to tweak this is through the `TAC` register. In here, two pieces of information are encoded. Firstly, the second bit serves as a simple flag to determine whether `TIMA` is even enabled at all (although it's unfortunately not that simple, as we will see). The zeroth and first bits determine the rate of the timer, defined in the following table.

| Bits 0-1 | T-Cycles per TIMA tick |
| -------- | ---------------------- |
| 0b00 | 1024 |
| 0b01 | 16 |
| 0b10 | 64 |
| 0b11 | 256 |

Let's create two new functions. The first will be to implement the table above, giving us what the current `TIMA` period should be, based on the value currently stored in `TAC`. The second function will be to use the first, masking the appropriate bit from `DIV` to see if it's time for `TIMA` to increment. We'll discuss how that process works in a moment.

```rust
// In timer.rs

// Unchanged code omitted

impl Timer {
    fn get_tima_period(&self) -> u16 {
        match self.tac & 0b11 {
            0b00 => 1 << 9,
            0b01 => 1 << 3,
            0b10 => 1 << 5,
            0b11 => 1 << 7,
            _ => unreachable!()
        }
    }

    fn tima_status(&self) -> bool {
        (self.div as u16 & self.get_tima_period()) != 0
    }
}
```

### TIMA

With all the pieces in place, we can now complete the behavior for `TIMA`. Like how `DIV` only increments on certain T-Cycles, `TIMA` only increments on certain values of `DIV`. Namely, when the bit specified by `get_tima_period()` goes from a 1 to a 0. This adds a slight bit of complexity, as we'll need to check the status of that function both before and after we increment `DIV`, to ensure that it does begin as a 1 and transition to a 0. This is the purpose of the `tima_status()` function, it's a small utility function for these checks. In addition to the status checks, we also need to check if `TIMA` is enabled at all via the bit flag in `TAC`. If these three conditions are true, then we can successfully increment `TIMA`.

```rust
// In timer.rs

// Unchanged code omitted

const TIMA_COOLDOWN_OVERFLOW: u8 = 4;

impl Timer {
    pub fn tick(&mut self, m_cycles: u8) -> bool {
        let mut interrupt = false;
        let t_cycles = 4 * m_cycles;

        for _ in 0..t_cycles {
            let (counter, overflow) = self.counter.overflowing_add(1);
            self.counter = counter;
            if !overflow {
                continue;
            }

            let old_bit = self.tima_status();
            self.div = self.div.wrapping_add(1);
            let new_bit = self.tima_status();
            let enabled = self.tac.get_bit(TAC_ENABLE_BIT);

            if enabled & old_bit & !new_bit {
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                self.tima = new_tima;
                if overflow {
                    self.tima_cooldown = TIMA_COOLDOWN_OVERFLOW;
                }
            }
        }

        interrupt
    }
}
```

Ah, yes, that business with an overflow cooldown. When `TIMA` overflows, it actually gets set to the value stored in `TMA`, it's the sole purpose of the `TMA` register. However, this does not happen immediately. Annoyingly, once the overflow occurs, `TIMA` will be set to zero for exactly four cycles, before finally being set to `TMA`'s value. Thus, the `tima_cooldown` variable. Following an overflow, it will be set to four, decrementing after every cycle until zero again, at which time `TIMA` will be set to the value in `TMA` and an interrupt will occur. This is slightly annoying behavior, but fortunately not terribly complex to implement.

```rust
// In timer.rs

// Unchanged code omitted

impl Timer {
    pub fn tick(&mut self, m_cycles: u8) -> bool {
        let mut interrupt = false;
        let t_cycles = 4 * m_cycles;

        for _ in 0..t_cycles {
            let (counter, overflow) = self.counter.overflowing_add(1);
            self.counter = counter;
            if !overflow {
                continue;
            }

            let old_bit = self.tima_status();
            self.div = self.div.wrapping_add(1);
            let new_bit = self.tima_status();
            let enabled = self.tac.get_bit(TAC_ENABLE_BIT);

            if self.tima_cooldown != 0 {
                self.tima_cooldown -= 1;
                if self.tima_cooldown == 0 {
                    self.tima = self.tma;
                    interrupt = true;
                }
            } else if enabled & old_bit & !new_bit {
                let (new_tima, overflow) = self.tima.overflowing_add(1);
                self.tima = new_tima;
                if overflow {
                    self.tima_cooldown = TIMA_COOLDOWN_OVERFLOW;
                }
            }
        }

        interrupt
    }
}
```

## Integration

With this, `Timer.rs` is effectively complete. All that remains is to integrate it into the rest of the program.

If you recall, our four Timer registers fall within the RAM block designated for I/O functionality, thus it will integrate it as part of that struct. To begin, we'll create a new `timer` member inside of `IO`, and perform the familiar changes to hook up the RAM access functions.

```rust
// In io.rs

// Unchanged code omitted

use crate::timer::*;

pub struct IO {
    buttons: [bool; 8],
    dpad_selected: bool,
    face_selected: bool,
    ram: [u8; IO_SIZE],
    timer: Timer,
}

impl IO {
    pub fn new() -> Self {
        Self {
            buttons: [false; 8],
            dpad_selected: false,
            face_selected: false,
            ram: [0; IO_SIZE],
            timer: Timer::new(),
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            DIV..=TAC => {
                self.timer.read_timer(addr)
            },
            JOYPAD_ADDR => {
                self.read_joypad()
            },
            _ => {
                let relative_addr = addr - IO_START;
                self.ram[relative_addr as usize]
            }
        }
    }

    pub fn update_timer(&mut self, cycles: u8) -> bool {
        self.timer.tick(cycles)
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            DIV..=TAC => {
                self.timer.write_timer(addr, val);
            },
            JOYPAD_ADDR => {
                self.face_selected = !val.get_bit(FACE_SELECT_BIT);
                self.dpad_selected = !val.get_bit(DPAD_SELECT_BIT);
            },
            _ => {
                let relative_addr = addr - IO_START;
                self.ram[relative_addr as usize] = val;
            }
        }
    }
}
```

For `read_u8` and `write_u8`, this is the only integration that is required. However, we will need to plug in our new `update_timer` function somewhere. Perhaps not surprisingly given this is the CPU timer, but that will fall to the CPU's `tick` function, via `Bus`. Let us once again perform the familiar action of hooking up some API through `Bus`.

```rust
// In bus.rs

// Unchanged code omitted

impl Bus {
    pub fn update_timer(&mut self, cycles: u8) -> bool {
        self.io.update_timer(cycles)
    }
}
```

Finally, `cpu/mod.rs`. Fortunately for us, integrating the CPU timer behavior only requires four additional lines -- calling `bus.update_timer`, then checking the output to see if an interrupt should happen, and if so, enabling the `Timer` interrupt type.

```rust
// In cpu/mod.rs

// Unchanged code omitted

impl Cpu {
    pub fn tick(&mut self) -> bool {
        self.last_read = None;
        self.last_write = None;
        let mut draw_time = false;
        let cycles = if self.halted { 1 } else { opcodes::execute(self) };
        let ppu_result = self.bus.update_ppu(cycles);
        if ppu_result.irq {
            self.enable_irq_type(Interrupts::Stat, true);
        }
        match ppu_result.lcd_result {
            LcdResults::RenderFrame => {
                // Render final scanline
                self.bus.render_scanline();
                self.enable_irq_type(Interrupts::Vblank, true);
                draw_time = true;
            },
            LcdResults::RenderLine => {
                self.bus.render_scanline();
            },
            _ => {},
        }

        let timer_irq = self.bus.update_timer(cycles);
        if timer_irq {
            self.enable_irq_type(Interrupts::Timer, true);
        }

        if let Some(irq) = self.check_irq() {
            self.trigger_irq(irq);
        }
        draw_time
    }
}
```

Compile this code, and give *Tetris* another spin. Assuming everything went well, you should see that the blocks are now randomly chosen. Good for us, but perhaps not as good for our high scores.

[*Next Chapter*](31-header.md)
