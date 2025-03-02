pub mod modes;
mod sprite;
mod tile;

use crate::utils::*;

use modes::{Lcd, LcdModeType, LcdResults};
use sprite::Sprite;
use tile::Tile;

pub const VRAM_START: u16           = 0x8000;
pub const VRAM_STOP: u16            = 0x9FFF;
pub const OAM_START: u16            = 0xFE00;
pub const OAM_STOP: u16             = 0xFE9F;
pub const LCD_REG_START: u16        = 0xFF40;
pub const LCD_REG_STOP: u16         = 0xFF4B;

const TILE_SET_START: u16           = 0x8000;
const TILE_SET_STOP: u16            = 0x97FF;
const TILE_MAP_START: u16           = 0x9800;
const TILE_MAP_STOP: u16            = 0x9FFF;

const BYTES_PER_TILE: u16           = 16;
const NUM_TILES: usize              = 384;
const TILE_MAP_SIZE: usize          = (TILE_MAP_STOP - TILE_MAP_START + 1) as usize;
const LCD_REG_SIZE: usize           = (LCD_REG_STOP - LCD_REG_START + 1) as usize;
const TILE_MAP_TABLE_SIZE: usize    = TILE_MAP_SIZE / 2;

const NUM_OAM_SPRITES: usize        = 40;
const BYTES_PER_SPRITE: u16         = 4;

const TILESIZE: usize               = 8;
const LAYERSIZE: usize              = 32;
const MAP_PIXELS: usize             = 256;

const LCDC: u16                     = 0xFF40;
const STAT: u16                     = 0xFF41;
const SCY: u16                      = 0xFF42;
const SCX: u16                      = 0xFF43;
const LY: u16                       = 0xFF44;
const LYC: u16                      = 0xFF45;
const BGP: u16                      = 0xFF47;
const OBP0: u16                     = 0xFF48;
const OBP1: u16                     = 0xFF49;
const WY: u16                       = 0xFF4A;
const WX: u16                       = 0xFF4B;

// Bit flags for LCDC
const LCDC_LCD_ENABLED_BIT: u8      = 7;
const LCDC_WNDW_MAP_BIT: u8         = 6;
const LCDC_WNDW_ENABLED_BIT: u8     = 5;
const LCDC_BG_WNDW_TILE_BIT: u8     = 4;
const LCDC_BG_MAP_BIT: u8           = 3;
const LCDC_SPR_SIZE_BIT: u8         = 2;
const LCDC_SPR_ENABLED_BIT: u8      = 1;
const LCDC_BG_WNDW_ENABLED_BIT: u8  = 0;

// Bit flags for STAT
const STAT_LY_LYC_IRQ_BIT: u8       = 6;
const STAT_OAM_IRQ_BIT: u8          = 5;
const STAT_VBLANK_IRQ_BIT: u8       = 4;
const STAT_HBLANK_IRQ_BIT: u8       = 3;
const STAT_LY_EQ_LYC_BIT: u8        = 2;

pub struct PpuUpdateResult {
    pub lcd_result: LcdResults,
    pub irq: bool,
}

pub struct Ppu {
    screen_buffer: [u8; DISPLAY_BUFFER],
    mode: Lcd,
    tiles: [Tile; NUM_TILES],
    maps: [u8; TILE_MAP_SIZE],
    lcd_regs: [u8; LCD_REG_SIZE],
    oam: [Sprite; NUM_OAM_SPRITES],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            screen_buffer: [0; DISPLAY_BUFFER],
            mode: Lcd::new(),
            tiles: [Tile::new(); NUM_TILES],
            maps: [0; TILE_MAP_SIZE],
            lcd_regs: [0; LCD_REG_SIZE],
            oam: [Sprite::new(); NUM_OAM_SPRITES],
        }
    }

    pub fn update(&mut self, cycles: u8) -> PpuUpdateResult {
        let old_mode = self.mode.get_mode();
        let old_line = self.mode.get_line();
        let lcd_result = self.mode.step(cycles);
        let mut stat = self.read_lcd_reg(STAT);
        let mut irq = false;

        let scanline = self.mode.get_line();
        if old_line != scanline {
            let lyc = self.read_lcd_reg(LYC);
            stat.set_bit(STAT_LY_EQ_LYC_BIT, scanline == lyc);
            irq = (scanline == lyc) && stat.get_bit(STAT_LY_LYC_IRQ_BIT);
            self.write_lcd_reg(LY, scanline);
        }

        let mode = self.mode.get_mode();
        if old_mode != mode {
            match mode {
                LcdModeType::HBLANK => {
                    irq |= stat.get_bit(STAT_HBLANK_IRQ_BIT);
                },
                LcdModeType::VBLANK => {
                    irq |= stat.get_bit(STAT_VBLANK_IRQ_BIT);
                },
                LcdModeType::OAMReadMode => {
                    irq |= stat.get_bit(STAT_OAM_IRQ_BIT);
                }
                _ => {},
            }
        }

        stat &= 0b1111_1100;
        stat |= mode.get_idx();
        self.write_lcd_reg(STAT, stat);

        PpuUpdateResult{ lcd_result, irq }
    }

    pub fn render(&self) -> [u8; DISPLAY_BUFFER] {
        if self.is_lcd_enabled() {
            self.screen_buffer
        } else {
            [0; DISPLAY_BUFFER]
        }
    }

    pub fn render_scanline(&mut self) {
        let line = self.read_lcd_reg(LY);
        let mut row = [0xFF; SCREEN_WIDTH * 4];

        if self.is_bg_layer_displayed() {
            self.render_bg(&mut row, line);
        }

        if self.is_window_layer_displayed() {
            self.render_window(&mut row, line);
        }

        if self.is_sprite_layer_displayed() {
            self.render_sprites(&mut row, line);
        }

        let start_idx = line as usize * SCREEN_WIDTH * 4;
        let end_idx = (line + 1) as usize * SCREEN_WIDTH * 4;
        self.screen_buffer[start_idx..end_idx].copy_from_slice(&row);
    }

    fn render_bg(&self, buffer: &mut [u8], line: u8) {
        let map_offset = self.get_bg_tile_map_index() as usize * TILE_MAP_TABLE_SIZE;
        let palette = self.get_bg_palette();
        let viewport = self.get_viewport_coords();
        let current_y = viewport.y as usize + line as usize;
        let y = current_y % MAP_PIXELS;
        let row = current_y % TILESIZE;
        for px in 0..SCREEN_WIDTH {
            let current_x = viewport.x as usize + px as usize;
            let x = current_x % MAP_PIXELS;
            let col = current_x % TILESIZE;
            let map_num = (y / TILESIZE) * LAYERSIZE + (x / TILESIZE);
            let tile_index = self.maps[map_offset + map_num] as usize;
            let adjusted_tile_index = if self.get_bg_wndw_tile_set_index() == 1 {
                tile_index as usize
            } else {
                (256 + tile_index as i8 as isize) as usize
            };
            let tile = self.tiles[adjusted_tile_index];
            let data = tile.get_row(row);
            let cell = data[col];
            let color_idx = palette[cell as usize];
            let color = GB_PALETTE[color_idx as usize];
            for i in 0..4 {
                buffer[4 * px + i] = color[i];
            }
        }
    }

    fn render_window(&self, buffer: &mut [u8], line: u8) {
        let map_offset = self.get_wndw_tile_map_index() as usize * TILE_MAP_TABLE_SIZE;
        let palette = self.get_bg_palette();
        let coords = self.get_window_coords();
        if (coords.x as usize > SCREEN_WIDTH) || (coords.y > line) {
            return;
        }
        let y = (line - coords.y) as usize;
        let row = y % TILESIZE;
        for x in (coords.x as usize)..SCREEN_WIDTH {
            let col = x % TILESIZE;
            let map_num = (y / TILESIZE) * LAYERSIZE + (x / TILESIZE);
            let tile_index = self.maps[map_offset + map_num] as usize;
            let adjusted_tile_index = if self.get_bg_wndw_tile_set_index() == 1 {
                tile_index as usize
            } else {
                (256 + tile_index as i8 as isize) as usize
            };
            let tile = self.tiles[adjusted_tile_index];
            let data = tile.get_row(row);
            let cell = data[col];
            let color_idx = palette[cell as usize];
            let color = GB_PALETTE[color_idx as usize];
            for i in 0..4 {
                buffer[4 * x + i] = color[i];
            }
        }
    }

    fn render_sprites(&self, buffer: &mut [u8], line: u8) {
        let sprites = self.sort_sprites();
        let bg_palette = self.get_bg_palette();
        let is_8x16 = self.are_sprites_8x16();
        for spr in sprites {
            let height = if is_8x16 { 16 } else { 8 };
            let coords = spr.get_coords();
            let signed_line = line as isize;
            if signed_line < coords.1 || coords.1 + height <= signed_line  {
                continue
            }
            let palette = self.get_sprite_palette(spr.use_palette1());
            let behind_bg = spr.get_bg_priority();
            let y = (signed_line - coords.1) as isize;
            let y_flipped = spr.is_y_flipped();
            let spr_idx = if is_8x16 {
                if (y < 8 && !y_flipped) || (8 < y && y_flipped) {
                    spr.get_tile_num() & 0xFE
                } else {
                    spr.get_tile_num() | 0x01
                }
            } else {
                spr.get_tile_num()
            };
            let tile = self.tiles[spr_idx as usize];
            let screen_y = y + coords.1;
            if screen_y < 0 || screen_y >= SCREEN_HEIGHT as isize {
                continue;
            }
            let mut data_y = if y_flipped { height - y - 1 } else { y };
            data_y %= 8;
            let row = tile.get_row(data_y as usize);
            for x in 0..8 {
                let data_x = if spr.is_x_flipped() { 7 - x } else { x };
                let cell = row[data_x as usize];
                // Continue if pixel is transparent
                if cell == 0 {
                    continue;
                }
                let screen_x = x + coords.0;
                if screen_x < 0 || screen_x >= SCREEN_WIDTH as isize {
                    continue;
                }
                let buffer_idx = 4 * (screen_x as usize);
                let current_rgba = &buffer[buffer_idx..(buffer_idx + 4)];
                // If current RGBA value isn't the transparent color, continue
                if behind_bg && current_rgba != GB_PALETTE[bg_palette[0] as usize] {
                    continue;
                }
                let color_idx = palette[cell as usize];
                let color = GB_PALETTE[color_idx as usize];
                for i in 0..4 {
                    buffer[buffer_idx + i] = color[i];
                }
            }
        }
    }

    pub fn read_lcd_reg(&self, addr: u16) -> u8 {
        let relative_addr = addr - LCD_REG_START;
        self.lcd_regs[relative_addr as usize]
    }

    pub fn read_oam(&self, addr: u16) -> u8 {
        let relative_addr = addr - OAM_START;
        let oam_idx = relative_addr / BYTES_PER_SPRITE;
        self.oam[oam_idx as usize].read_u8(addr)
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        match addr {
            TILE_SET_START..=TILE_SET_STOP => {
                let relative_addr = addr - TILE_SET_START;
                let tile_idx = relative_addr / BYTES_PER_TILE;
                let offset = relative_addr % BYTES_PER_TILE;
                self.tiles[tile_idx as usize].read_u8(offset)
            },
            TILE_MAP_START..=TILE_MAP_STOP => {
                let relative_addr = addr - TILE_MAP_START;
                self.maps[relative_addr as usize]
            },
            _ => { unreachable!() }
        }
    }

    fn sort_sprites(&self) -> Vec<Sprite> {
        let mut sprites = self.oam.to_vec();
        sprites.reverse();
        sprites.sort_by(|a, b| b.get_coords().0.cmp(&a.get_coords().0));
        sprites
    }

    pub fn write_lcd_reg(&mut self, addr: u16, val: u8) {
        let relative_addr = addr - LCD_REG_START;
        self.lcd_regs[relative_addr as usize] = val;
    }

    pub fn write_oam(&mut self, addr: u16, val: u8) {
        let relative_addr = addr - OAM_START;
        let oam_idx = relative_addr / BYTES_PER_SPRITE;
        self.oam[oam_idx as usize].write_u8(addr, val);
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        match addr {
            TILE_SET_START..=TILE_SET_STOP => {
                let relative_addr = addr - TILE_SET_START;
                let tile_idx = relative_addr / BYTES_PER_TILE;
                let offset = relative_addr % BYTES_PER_TILE;
                self.tiles[tile_idx as usize].write_u8(offset, val);
            },
            TILE_MAP_START..=TILE_MAP_STOP => {
                let relative_addr = addr - TILE_MAP_START;
                self.maps[relative_addr as usize] = val;
            },
            _ => { unreachable!() }
        }
    }

    fn are_sprites_8x16(&self) -> bool {
        let lcdc = self.read_lcd_reg(LCDC);
        lcdc.get_bit(LCDC_SPR_SIZE_BIT)
    }

    fn get_bg_palette(&self) -> [u8; 4] {
        unpack_u8(self.read_lcd_reg(BGP))
    }

    fn get_sprite_palette(&self, palette1: bool) -> [u8; 4] {
        if palette1 {
             unpack_u8(self.read_lcd_reg(OBP1))
        } else {
             unpack_u8(self.read_lcd_reg(OBP0))
        }
    }

    fn get_bg_wndw_tile_set_index(&self) -> u8 {
        let lcdc = self.read_lcd_reg(LCDC);
        if lcdc.get_bit(LCDC_BG_WNDW_TILE_BIT) { 1 } else { 0 }
    }

    fn get_bg_tile_map_index(&self) -> u8 {
        let lcdc = self.read_lcd_reg(LCDC);
        if lcdc.get_bit(LCDC_BG_MAP_BIT) { 1 } else { 0 }
    }

    fn get_viewport_coords(&self) -> Point {
        let x = self.read_lcd_reg(SCX);
        let y = self.read_lcd_reg(SCY);
        Point::new(x, y)
    }

    fn get_window_coords(&self) -> Point {
        let x = self.read_lcd_reg(WX);
        let y = self.read_lcd_reg(WY);
        Point::new(x.saturating_sub(7), y)
    }

    fn get_wndw_tile_map_index(&self) -> u8 {
        let lcdc = self.read_lcd_reg(LCDC);
        if lcdc.get_bit(LCDC_WNDW_MAP_BIT) { 1 } else { 0 }
    }

    fn is_lcd_enabled(&self) -> bool {
        let lcdc = self.read_lcd_reg(LCDC);
        lcdc.get_bit(LCDC_LCD_ENABLED_BIT)
    }

    fn is_bg_layer_displayed(&self) -> bool {
        let lcdc = self.read_lcd_reg(LCDC);
        lcdc.get_bit(LCDC_BG_WNDW_ENABLED_BIT)
    }

    fn is_sprite_layer_displayed(&self) -> bool {
        let lcdc = self.read_lcd_reg(LCDC);
        lcdc.get_bit(LCDC_SPR_ENABLED_BIT)
    }

    fn is_window_layer_displayed(&self) -> bool {
        let lcdc = self.read_lcd_reg(LCDC);
        lcdc.get_bit(LCDC_BG_WNDW_ENABLED_BIT) && lcdc.get_bit(LCDC_WNDW_ENABLED_BIT)
    }
}
