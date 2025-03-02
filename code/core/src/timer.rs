use crate::utils::BitOps;

pub const DIV: u16      = 0xFF04;
pub const TIMA: u16     = 0xFF05;
pub const TMA: u16      = 0xFF06;
pub const TAC: u16      = 0xFF07;

const TAC_ENABLE_BIT: u8 = 3;

const TIMA_COOLDOWN_OVERFLOW: u8 = 4;

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

    pub fn tick(&mut self, m_cycles: u8) -> bool {
        let mut interrupt = false;
        let t_cycles = 4 * m_cycles;

        for _ in 0..t_cycles {
            let (counter, overflow) = self.counter.overflowing_add(1);
            self.counter = counter;
            if !overflow {
                continue;
            }

            let old_bit = self.tima_tick();
            self.div = self.div.wrapping_add(1);
            let new_bit = self.tima_tick();
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
                    self.tima = 0;
                    self.tima_cooldown = TIMA_COOLDOWN_OVERFLOW;
                }
            }
        }

        interrupt
    }

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
            DIV => { self.div = 0 },
            TIMA => {
                self.tima = val;
                self.tima_cooldown = 0;
            },
            TMA => { self.tma = val },
            TAC => { self.tac = val },
            _ => unreachable!("Trying to write to a non-timer register")
        }
    }

    fn get_tima_period(&self) -> u16 {
        match self.tac & 0b11 {
            0b00 => 1 << 9,
            0b01 => 1 << 3,
            0b10 => 1 << 5,
            0b11 => 1 << 7,
            _ => unreachable!()
        }
    }

    fn tima_tick(&self) -> bool {
        (self.div as u16 & self.get_tima_period()) != 0
    }
}
