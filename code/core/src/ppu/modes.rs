const HBLANK_LEN: usize = 204;
const VBLANK_LEN: usize = 456;
const OAM_READ_LEN: usize = 80;
const VRAM_READ_LEN: usize = 172;

const VBLANK_LINE_START: u8 = 143;
const VBLANK_LINE_END: u8   = VBLANK_LINE_START + 10;

#[derive(PartialEq)]
pub enum LcdResults {
    NoAction,
    RenderFrame,
    RenderLine,
}

#[derive(PartialEq, Clone, Copy)]
pub enum LcdModeType {
    HBLANK,
    VBLANK,
    OAMReadMode,
    VRAMReadMode,
}

impl LcdModeType {
    pub fn get_idx(&self) -> u8 {
        match *self {
            LcdModeType::HBLANK => { 0 },
            LcdModeType::VBLANK => { 1 },
            LcdModeType::OAMReadMode => { 2 },
            LcdModeType::VRAMReadMode => { 3 },
        }
    }
}

pub struct Lcd {
    mode: LcdModeType,
    cycles: usize,
    line: u8,
}

impl Lcd {
    pub fn new() -> Self {
        Self {
            mode: LcdModeType::HBLANK,
            cycles: 0,
            line: 0,
        }
    }

    pub fn get_line(&self) -> u8 {
        self.line
    }

    pub fn get_mode(&self) -> LcdModeType {
        self.mode
    }

    pub fn step(&mut self, cycles: u8) -> LcdResults {
        self.cycles += cycles as usize;
        let mut result = LcdResults::NoAction;

        match self.mode {
            LcdModeType::HBLANK => {
                if self.cycles >= HBLANK_LEN {
                    self.cycles = 0;
                    self.line += 1;

                    // If we've finished line 143, we've finished a frame, time for VBLANK
                    if self.line == VBLANK_LINE_START {
                        self.mode = LcdModeType::VBLANK;
                        result = LcdResults::RenderFrame;
                    } else {
                        self.mode = LcdModeType::OAMReadMode;
                    }
                }
            },
            LcdModeType::VBLANK => {
                if self.cycles >= VBLANK_LEN {
                    self.cycles = 0;
                    self.line += 1;

                    if self.line > VBLANK_LINE_END {
                        self.mode = LcdModeType::OAMReadMode;
                        self.line = 0;
                    }
                }
            },
            LcdModeType::OAMReadMode => {
                if self.cycles >= OAM_READ_LEN {
                    self.cycles = 0;
                    self.mode = LcdModeType::VRAMReadMode;
                }
            },
            LcdModeType::VRAMReadMode => {
                if self.cycles >= VRAM_READ_LEN {
                    self.cycles = 0;
                    self.mode = LcdModeType::HBLANK;
                    result = LcdResults::RenderLine;
                }
            }
        }

        return result;
    }
}
