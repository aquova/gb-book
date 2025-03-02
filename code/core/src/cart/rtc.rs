extern crate wasm_timer;
use wasm_timer::Instant;

use crate::utils::BitOps;

const SECS_IN_MIN: u64  = 60;
const MINS_IN_HOUR: u64 = 60;
const HOURS_IN_DAY: u64 = 24;

const DAY_HIGH_BIT: u8      = 0;
const HALT_BIT: u8          = 6;
const DAY_OVERFLOW_BIT: u8  = 7;

pub struct Rtc {
    start: Instant,
    seconds: u8,
    minutes: u8,
    hours: u8,
    days: u16,
    enabled: bool,
    // TODO: Check this
    halted: bool,
}

impl Rtc {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            seconds: 0,
            minutes: 0,
            hours: 0,
            days: 0,
            enabled: false,
            halted: false,
        }
    }

    pub fn latch_time(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.start);
        let d_sec = delta.as_secs();

        self.seconds = (d_sec % SECS_IN_MIN) as u8;

        let d_min = d_sec / SECS_IN_MIN;
        self.minutes = (d_min % MINS_IN_HOUR) as u8;

        let d_hour = d_min / MINS_IN_HOUR;
        self.hours = (d_hour % HOURS_IN_DAY) as u8;

        let d_days = d_hour / HOURS_IN_DAY;
        self.days = d_days as u16;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn read_byte(&self, bank: u8) -> u8 {
        match bank {
            0x08 => { self.seconds },
            0x09 => { self.minutes },
            0x0A => { self.hours },
            0x0B => { (self.days & 0xFF) as u8 },
            0x0C => {
                let mut ret = 0;
                ret.set_bit(DAY_HIGH_BIT, self.days.get_bit(9));
                ret.set_bit(HALT_BIT, self.halted);
                ret.set_bit(DAY_OVERFLOW_BIT, self.days.get_bit(10));
                ret
            },
            _ => { unreachable!() }
        }
    }

    pub fn write_byte(&mut self, bank: u8, val: u8) {
        match bank {
            0x08 => { self.seconds = val; },
            0x09 => { self.minutes = val; },
            0x0A => { self.hours = val; },
            0x0B => {
                self.days = (self.days & 0xFF00) | (val as u16);
            },
            0x0C => {
                self.days.set_bit(9, val.get_bit(DAY_HIGH_BIT));
                self.halted = val.get_bit(HALT_BIT);
                self.days.set_bit(10, val.get_bit(DAY_OVERFLOW_BIT));
            },
            _ => {
                if val == 0x00 {
                    self.enabled = false;
                } else if val == 0x01 && !self.enabled {
                    self.enabled = true;
                    self.latch_time();
                }
            }
        }
    }
}
