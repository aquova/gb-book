pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const DISPLAY_BUFFER: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;

pub const GB_PALETTE: [[u8; 4]; 4] = [
    [255, 255, 255, 255],
    [128, 128, 128, 255],
    [64,  64,  64,  255],
    [0,   0,   0,   255],
];

#[derive(Clone, Copy)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}

impl Point {
    pub fn new(x: u8, y: u8) -> Self {
        Point{x, y}
    }
}

pub trait BitOps {
    fn get_bit(&self, bit: u8) -> bool;
    fn set_bit(&mut self, bit: u8, set: bool);
}

macro_rules! impl_bitops {
    ($T:ty) => {
        impl BitOps for $T {
            fn get_bit(&self, bit: u8) -> bool {
                let mask = 0b1 << bit;
                (self & mask) != 0
            }

            fn set_bit(&mut self, bit: u8, set: bool) {
                let mask = 0b1 << bit;
                if set {
                    *self |= mask;
                } else {
                    *self &= !mask;
                }
            }
        }
    }
}

impl_bitops!(u8);
impl_bitops!(u16);

pub trait ByteOps {
    fn high_byte(&self) -> u8;
    fn low_byte(&self) -> u8;
}

impl ByteOps for u16 {
    fn high_byte(&self) -> u8 {
        (self >> 8) as u8
    }

    fn low_byte(&self) -> u8 {
        (self & 0xFF) as u8
    }
}

pub fn merge_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | (low as u16)
}

pub fn check_h_carry_u8(lhs: u8, rhs: u8) -> bool {
    ((lhs & 0xF) + (rhs & 0xF)) & 0xF0 != 0
}

pub fn check_h_carry_u16(lhs: u16, rhs: u16) -> bool {
    ((lhs & 0xFFF) + (rhs & 0xFFF)) & 0xF000 != 0
}

pub fn check_h_borrow_u8(lhs: u8, rhs: u8) -> bool {
    (lhs & 0xF).checked_sub(rhs & 0xF).is_none()
}

pub fn check_h_borrow_u16(lhs: u16, rhs: u16) -> bool {
    (lhs & 0xF).checked_sub(rhs & 0xF).is_none()
}

pub fn check_c_carry_u8(lhs: u8, rhs: u8) -> bool {
    lhs.checked_add(rhs).is_none()
}

pub fn check_c_carry_u16(lhs: u16, rhs: u16) -> bool {
    lhs.checked_add(rhs).is_none()
}

pub fn check_c_borrow_u8(lhs: u8, rhs: u8) -> bool {
    lhs.checked_sub(rhs).is_none()
}

pub fn check_c_borrow_u16(lhs: u16, rhs: u16) -> bool {
    lhs.checked_sub(rhs).is_none()
}

pub fn unpack_u8(val: u8) -> [u8; 4] {
    let mut output = [0; 4];
    output[0] = val & 0b0000_0011;
    output[1] = (val & 0b0000_1100) >> 2;
    output[2] = (val & 0b0011_0000) >> 4;
    output[3] = (val & 0b1100_0000) >> 6;
    output
}

pub fn pack_u8(a: &[u8]) -> u8 {
    let mut output = a[0];
    output |= a[1] << 2;
    output |= a[2] << 4;
    output |= a[3] << 6;
    output
}
