pub const WRAM_START: u16       = 0xC000;
pub const WRAM_STOP: u16        = 0xDFFF;
pub const ECHO_START: u16       = 0xE000;
pub const ECHO_STOP: u16        = 0xFDFF;

const WRAM_SIZE: usize          = (WRAM_STOP - WRAM_START + 1) as usize;

pub struct WRAM {
    wram: [u8; WRAM_SIZE],
}

impl WRAM {
    pub fn new() -> Self {
        Self {
            wram: [0; WRAM_SIZE],
        }
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        match addr {
            WRAM_START..=WRAM_STOP => {
                let relative_addr = addr - WRAM_START;
                self.wram[relative_addr as usize]
            },
            ECHO_START..=ECHO_STOP => {
                let relative_addr = addr - ECHO_START;
                self.wram[relative_addr as usize]
            },
            _ => { unreachable!() }
        }
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            WRAM_START..=WRAM_STOP => {
                let relative_addr = addr - WRAM_START;
                self.wram[relative_addr as usize] = val;
            },
            ECHO_START..=ECHO_STOP => {
                let relative_addr = addr - ECHO_START;
                self.wram[relative_addr as usize] = val;
            },
            _ => { unreachable!() }
        }
    }
}
