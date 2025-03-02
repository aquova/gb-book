use crate::utils::*;

const Y_OFFSET: isize = 16;
const X_OFFSET: isize = 8;

const BG_PRIORITY_BIT: u8   = 7;
const Y_FLIP_BIT: u8        = 6;
const X_FLIP_BIT: u8        = 5;
const PALETTE_BIT: u8       = 4;

#[derive(Clone, Copy)]
pub struct Sprite {
    pos: Point,
    tile_num: u8,
    bg_priority: bool,
    x_flip: bool,
    y_flip: bool,
    palette1: bool,
}

impl Sprite {
    pub fn new() -> Self {
        Self {
            pos: Point::new(0, 0),
            tile_num: 0,
            bg_priority: false,
            x_flip: false,
            y_flip: false,
            palette1: false,
        }
    }

    pub fn get_bg_priority(&self) -> bool {
        self.bg_priority
    }

    pub fn get_coords(&self) -> (isize, isize) {
        (self.pos.x as isize - X_OFFSET, self.pos.y as isize - Y_OFFSET)
    }

    pub fn get_tile_num(&self) -> u8 {
        self.tile_num
    }

    pub fn is_x_flipped(&self) -> bool {
        self.x_flip
    }

    pub fn is_y_flipped(&self) -> bool {
        self.y_flip
    }

    pub fn read_u8(&self, addr: u16) -> u8 {
        let offset = addr % 4;
        match offset {
            0 => {
                self.pos.y
            },
            1 => {
                self.pos.x
            },
            2 => {
                self.tile_num
            },
            3 => {
                let mut ret = 0;
                ret.set_bit(BG_PRIORITY_BIT, self.bg_priority);
                ret.set_bit(Y_FLIP_BIT, self.y_flip);
                ret.set_bit(X_FLIP_BIT, self.x_flip);
                ret.set_bit(PALETTE_BIT, self.palette1);
                ret
            }
            _ => { unreachable!() }
        }
    }

    pub fn use_palette1(&self) -> bool {
        self.palette1
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        let offset = addr % 4;
        match offset {
            0 => {
                self.pos.y = val;
            },
            1 => {
                self.pos.x = val;
            },
            2 => {
                self.tile_num = val;
            },
            3 => {
                self.bg_priority = val.get_bit(BG_PRIORITY_BIT);
                self.y_flip = val.get_bit(Y_FLIP_BIT);
                self.x_flip = val.get_bit(X_FLIP_BIT);
                self.palette1 = val.get_bit(PALETTE_BIT);
            },
            _ => { unreachable!(); }
        }
    }
}
