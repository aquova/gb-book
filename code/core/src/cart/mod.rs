mod rtc;

use std::str::from_utf8;

use rtc::Rtc;
use crate::utils::BitOps;

pub const ROM_START: u16        = 0x0000;
pub const ROM_STOP: u16         = 0x7FFF;
pub const EXT_RAM_START: u16    = 0xA000;
pub const EXT_RAM_STOP: u16     = 0xBFFF;

const RAM_ENABLE_START: u16     = 0x0000;
const RAM_ENABLE_STOP: u16      = 0x1FFF;
const ROM_BANK_NUM_START: u16   = 0x2000;
const ROM_BANK_NUM_STOP: u16    = 0x3FFF;
const RAM_BANK_NUM_START: u16   = 0x4000;
const RAM_BANK_NUM_STOP: u16    = 0x5FFF;
const ROM_RAM_MODE_START: u16   = 0x6000;
const ROM_RAM_MODE_STOP: u16    = 0x7FFF;

const ROM_BANK_LOW_START: u16   = 0x2000;
const ROM_BANK_LOW_STOP: u16    = 0x2FFF;
const ROM_BANK_HIGH_START: u16  = 0x3000;
const ROM_BANK_HIGH_STOP: u16   = 0x3FFF;

const TITLE_START: usize        = 0x0134;
const TITLE_STOP: usize         = 0x0142;
const CART_TYPE_ADDR: usize     = 0x0147;
const RAM_SIZE_ADDR: usize      = 0x0149;

const ROM_BANK_SIZE: usize      = 0x4000;
const RAM_BANK_SIZE: usize      = 0x2000;

const MBC2_ROM_CONTROL_BIT: u8  = 8;

#[derive(Clone, Copy, PartialEq)]
pub enum MBC {
    NONE,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    INV,
}

const RAM_SIZES: [usize; 6] = [
    0,
    2,
    8,
    32,
    128,
    64
];

/*
 * ROM Header Layout
 * Header runs from $0100-$014F
 *
 * +-------------------------+ $100
 * |       Start Vector      |
 * +-------------------------+ $104
 * |      Nintendo Logo      |
 * +-------------------------+ $134
 * |       Game Title        |
 * +-------------------------+ $13F
 * | Manufacturer Code (GBC) |
 * +-------------------------+ $143
 * |        GBC Flag         |
 * +-------------------------+ $144
 * |    New Licensee Code    |
 * +-------------------------+ $146
 * |        SGB Flag         |
 * +-------------------------+ $147
 * |     Cartridge Type      |
 * +-------------------------+ $148
 * |        ROM Size         |
 * +-------------------------+ $149
 * |        RAM Size         |
 * +-------------------------+ $14A
 * |     Destination Code    |
 * +-------------------------+ $14B
 * |    Old Licensee Code    |
 * +-------------------------+ $14C
 * |      ROM Version        |
 * +-------------------------+ $14D
 * |    Header Checksum      |
 * +-------------------------+ $14E
 * |    Global Checksum      |
 * +-------------------------+ $14F
 *
 */

pub struct Cart {
    rom: Vec<u8>,
    ram: Vec<u8>,
    rom_bank: u16,
    ram_bank: u8,
    mbc: MBC,
    rtc: Rtc,
    rom_mode: bool,
    ram_enabled: bool,
}

impl Cart {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            ram: Vec::new(),
            rom_bank: 1,
            ram_bank: 0,
            mbc: MBC::NONE,
            rtc: Rtc::new(),
            rom_mode: true,
            ram_enabled: false,
        }
    }

    pub fn get_battery_data(&self) -> &[u8] {
        &self.ram
    }

    fn get_mbc(&self) -> MBC {
        let cart_type = self.rom[CART_TYPE_ADDR];
        match cart_type {
            0x00 =>         { MBC::NONE },
            0x01..=0x03 =>  { MBC::MBC1 },
            0x05..=0x06 =>  { MBC::MBC2 },
            0x0F..=0x13 =>  { MBC::MBC3 },
            0x19..=0x1E =>  { MBC::MBC5 },
            _ =>            { MBC::INV },
        }
    }

    pub fn get_title(&self) -> &str {
        let data = &self.rom[TITLE_START..TITLE_STOP];
        from_utf8(data).unwrap().trim_end_matches(char::from(0))
    }

    pub fn has_battery(&self) -> bool {
        let has_battery = [
            0x03, 0x06, 0x09,
            0x0D, 0x0F, 0x10,
            0x13, 0x1B, 0x1E,
        ];

        let cart_type = self.rom[CART_TYPE_ADDR];
        has_battery.contains(&cart_type)
    }

    fn has_external_ram(&self) -> bool {
        let has_ext_ram = [
            0x02, 0x03, 0x08,
            0x09, 0x0C, 0x0D,
            0x10, 0x12, 0x13,
            0x16, 0x17, 0x1A,
            0x1B, 0x1D, 0x1E,
        ];

        let cart_type = self.rom[CART_TYPE_ADDR];
        has_ext_ram.contains(&cart_type)
    }

    fn init_ext_ram(&mut self) {
        let mut ram_size_idx = self.rom[RAM_SIZE_ADDR] as usize;

        // Some headers don't report their external RAM capacity correctly
        if self.has_external_ram() && ram_size_idx == 0 {
            ram_size_idx = 1;
        }

        if self.mbc == MBC::MBC2 {
            // MBC2 always has 512 bytes of RAM directly on chip
            self.ram = vec![0; 512];
        } else {
            let ram_size = RAM_SIZES[ram_size_idx] * 1024;
            self.ram = vec![0; ram_size];
        }
    }

    pub fn load_cart(&mut self, rom: &[u8]) {
        self.rom = rom.to_vec();
        self.mbc = self.get_mbc();
        self.init_ext_ram();
    }

    pub fn read_cart(&self, addr: u16) -> u8 {
        if (addr as usize) < ROM_BANK_SIZE {
            self.rom[addr as usize]
        } else {
            let rel_addr = (addr as usize) - ROM_BANK_SIZE;
            let bank_addr = (self.rom_bank as usize) * ROM_BANK_SIZE + rel_addr;
            self.rom[bank_addr]
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        match self.mbc {
            MBC::NONE | MBC::MBC1 | MBC::MBC2 | MBC::MBC5 => {
                self.read_ram_helper(addr)
            },
            MBC::MBC3 => {
                self.mbc3_read_ram(addr)
            }
            _ => unimplemented!()
        }
    }

    fn mbc3_read_ram(&self, addr: u16) -> u8 {
        if self.rtc.is_enabled() && (0x08 <= self.ram_bank && self.ram_bank <= 0x0C) {
            self.rtc.read_byte(self.ram_bank)
        } else {
            self.read_ram_helper(addr)
        }
    }

    fn read_ram_helper(&self, addr: u16) -> u8 {
        let rel_addr = (addr - EXT_RAM_START) as usize;
        let bank_addr = (self.ram_bank as usize) * RAM_BANK_SIZE + rel_addr;
        self.ram[bank_addr]
    }

    pub fn set_battery_data(&mut self, data: &[u8]) {
        self.ram.copy_from_slice(data);
    }

    pub fn write_cart(&mut self, addr: u16, val: u8) {
        match self.mbc {
            MBC::NONE => {},
            MBC::MBC1 => { self.mbc1_write_rom(addr, val); },
            MBC::MBC2 => { self.mbc2_write_rom(addr, val); },
            MBC::MBC3 => { self.mbc3_write_rom(addr, val); },
            MBC::MBC5 => { self.mbc5_write_rom(addr, val); },
            _ => unimplemented!()
        }
    }

    pub fn write_ram(&mut self, addr: u16, val: u8) {
        match self.mbc {
            MBC::NONE => {
                let rel_addr = addr - EXT_RAM_START;
                self.ram[rel_addr as usize] = val;
            },
            MBC::MBC1 | MBC::MBC5 => {
                self.write_ram_helper(addr, val)
            },
            MBC::MBC3 => self.mbc3_write_ram(addr, val),
            _ => unimplemented!()
        }
    }

    fn mbc1_write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            RAM_ENABLE_START..=RAM_ENABLE_STOP => {
                self.ram_enabled = val == 0x0A;
            },
            ROM_BANK_NUM_START..=ROM_BANK_NUM_STOP => {
                let bank = (val & 0x1F) as u16;
                match bank {
                    // Bank numbers 0x00, 0x20, 0x40, 0x60 aren't used
                    // Instead they load the next bank
                    0x00 | 0x20 | 0x40 | 0x60 => {
                        self.rom_bank = bank + 1;
                    },
                    _ => {
                        self.rom_bank = bank;
                    }
                }
            },
            RAM_BANK_NUM_START..=RAM_BANK_NUM_STOP => {
                let bits = val & 0b11;

                if self.rom_mode {
                    self.rom_bank |= (bits << 5) as u16;
                } else {
                    self.ram_bank = bits;
                }
            },
            ROM_RAM_MODE_START..=ROM_RAM_MODE_STOP => {
                self.rom_mode = val == 0;
            },
            _ => unreachable!()
        }
    }

    fn mbc2_write_rom(&mut self, addr: u16, val: u8) {
        let bank_swap = addr.get_bit(MBC2_ROM_CONTROL_BIT);
        if bank_swap {
            self.rom_bank = (val & 0x0F) as u16;
        } else {
            self.ram_enabled = val == 0x0A;
        }
    }

    fn mbc3_write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            RAM_ENABLE_START..=RAM_ENABLE_STOP => {
                self.ram_enabled = val == 0x0A;
            },
            ROM_BANK_NUM_START..=ROM_BANK_NUM_STOP => {
                if val == 0x00 {
                    self.rom_bank = 0x01;
                } else {
                    self.rom_bank = val as u16;
                }
            },
            RAM_BANK_NUM_START..=RAM_BANK_NUM_STOP => {
                self.ram_bank = val;
            },
            ROM_RAM_MODE_START..=ROM_RAM_MODE_STOP => {
                self.rtc.write_byte(self.ram_bank, val);
            },
            _ => unreachable!()
        }
    }

    fn mbc5_write_rom(&mut self, addr: u16, val: u8) {
        match addr {
            RAM_ENABLE_START..=RAM_ENABLE_STOP => {
                self.ram_enabled = val == 0x0A;
            },
            ROM_BANK_LOW_START..=ROM_BANK_LOW_STOP => {
                self.rom_bank &= 0xFF00;
                self.rom_bank |= val as u16;
            },
            ROM_BANK_HIGH_START..=ROM_BANK_HIGH_STOP => {
                self.rom_bank.set_bit(9, val != 0);
            },
            RAM_BANK_NUM_START..=RAM_BANK_NUM_STOP => {
                self.ram_bank = val & 0x0F;
            },
            _ => unreachable!()
        }
    }

    fn mbc3_write_ram(&mut self, addr: u16, val: u8) {
        match self.ram_bank {
            0x00..=0x03 => {
                self.write_ram_helper(addr, val);
            },
            0x08..=0x0C => {
                if self.ram_enabled {
                    self.rtc.write_byte(self.ram_bank, val);
                }
            },
            _ => {}
        }
    }

    fn write_ram_helper(&mut self, addr: u16, val: u8) {
        if self.ram_enabled {
            let rel_addr = (addr - EXT_RAM_START) as usize;
            let ram_addr = (self.ram_bank as usize) * RAM_BANK_SIZE + rel_addr;
            self.ram[ram_addr] = val;
        }
    }
}
