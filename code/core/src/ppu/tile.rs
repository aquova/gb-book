use crate::utils::*;

#[derive(Copy, Clone)]
pub struct Tile {
    pub pixels: [[u8; 8]; 8]
}

impl Tile {
    pub fn new() -> Self {
        Self {
            pixels: [[0; 8]; 8]
        }
    }

    pub fn get_row(&self, row: usize) -> [u8; 8] {
        self.pixels[row]
    }

    pub fn read_u8(&self, offset: u16) -> u8 {
        if offset > 16 {
            panic!("Offset too large to fit in this tile");
        }
        let row = (offset / 2) as usize;
        let bit = (offset % 2) as u8;
        let mut ret = 0;
        for i in 0..8 {
            ret <<= 1;
            ret |= if self.pixels[row][7 - i].get_bit(bit) { 1 } else { 0 };
        }
        ret
    }

    pub fn write_u8(&mut self, offset: u16, val: u8) {
        if offset > 16 {
            panic!("Offset too large to fit in this tile");
        }
        let row = (offset / 2) as usize;
        let bit = (offset % 2) as u8;
        for i in 0..8 {
            self.pixels[row][7 - i].set_bit(bit, val.get_bit(i as u8))
        }
    }
}

