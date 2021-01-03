use std::fmt;
use std::thread;

pub struct PPU {
    mode: u8,
    // 8KB Video RAM(VRAM)
    vram: Vec<u8>,
    pub buffer: Vec<u32>,
    clocks: usize,
    // I/O Registers
    // LCD Control Register
    lcdc: u8,
    // Scroll X
    scx: u8,
    // Scroll Y
    scy: u8,
    // Y-Coordinate (R)
    ly: u8,
    // Window X Position minus 7
    wx: u8,
    // Window Y Position minus 7
    wy: u8,
    scanline: [u8; WIDTH],
    // VBlank
    pub vblank: bool,
    pub debug: bool,
}

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

// PG Palette Data
pub const DARKEST_GREEN: u32 = 0xFF0F380F;
pub const DARK_GREEN: u32 = 0xFF306230;
pub const LIGHT_GREEN: u32 = 0xFF8BAC0F;
pub const LIGHTEST_GREEN: u32 = 0xFF9BBC0F;

impl fmt::Debug for PPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PPU {{ mode: {:#X}, clocks: {:#X}}} \n
            I/O Registers: {{ lcdc: {:?}, ly: {:?} }} \n
            scx: {}
            scy: {}
            ",
            self.mode, self.clocks, self.lcdc, self.ly, self.scx, self.scy,
        )
    }
}

impl PPU {
    pub fn new() -> PPU {
        return PPU {
            mode: 2,
            vram: vec![0; 0x2000],
            buffer: vec![DARKEST_GREEN; WIDTH * HEIGHT],
            clocks: 0,
            lcdc: 0b1000_0000,
            scx: 0,
            scy: 0,
            ly: 0,
            wx: 0,
            wy: 0,
            scanline: [0; WIDTH],
            vblank: false,
            debug: false,
        };
    }

    pub fn step(&mut self, clocks: usize) {
        if self.lcdc & 0b1000_0000 == 0 {
            return;
        }

        self.clocks += clocks;

        // ref. http://imrannazar.com/GameBoy-Emulation-in-JavaScript:-GPU-Timings

        match self.mode {
            // OAM read mode
            2 => {
                if self.clocks >= 80 {
                    self.mode = 3;
                    self.clocks = 0;
                }
            }
            // VRAM read mode
            3 => {
                if self.clocks >= 172 {
                    self.mode = 0;
                    self.clocks = 0;

                    self.redner_scanline();
                }
            }
            // Hblank
            0 => {
                if self.clocks >= 204 {
                    self.clocks = 0;
                    self.ly += 1;

                    if self.ly == 143 {
                        // Enter vblank
                        self.mode = 1;
                        self.vblank = true;
                    } else {
                        self.mode = 2;
                    }
                }
            }
            // Vblank (10 lines)
            1 => {
                if self.clocks >= 456 {
                    self.clocks = 0;
                    self.ly += 1;

                    if self.ly > 153 {
                        self.mode = 2;
                        self.ly = 0;
                    }
                }
            }
            _ => {
                panic!("not implemented");
            }
        }

        if self.debug {
            println!("#{:?}", self);
        }
    }

    fn render_background(&mut self) {
        // pixel to tile_index. (0 ~ 31)
        let mut tile_index_x = self.scx / 8;
        let mut tile_index_y = (self.scy.wrapping_add(self.ly)) / 8;

        // offsets within tile (0 ~ 7)
        let mut offset_x = self.scx % 8;
        let mut offset_y = (self.scy.wrapping_add(self.ly)) % 8;

        let mut tile = self.fetch_background_tile(tile_index_x, tile_index_y, offset_y);

        let mut should_render_window = false;

        for x in 0..WIDTH {
            // LCDC: Bit 5 - Window Display Enable          (0=Off, 1=On)
            if self.lcdc & 0b0010_0000 > 0 {
                // top left corner of a window are wx-7,wy.
                if self.wy <= self.ly && self.wx == (x + 7) as u8 {
                    tile_index_x = 0;
                    tile_index_y = (self.ly - self.wy) / 8;
                    offset_x = 0;
                    offset_y = (self.ly - self.wy) % 8;
                    tile = self.fetch_window_tile(tile_index_x, tile_index_y, offset_y);
                    should_render_window = true;
                }
            }

            let tile0 = tile.0;
            let tile1 = tile.1;

            // color number (0, 1, 2, 3)
            let mask = 1 << (7 - offset_x);
            let lsb = tile0 & mask;
            let msb = tile1 & mask;

            let color = match (lsb != 0, msb != 0) {
                (true, true) => 3,
                (false, true) => 2,
                (true, false) => 1,
                (false, false) => 0,
            };

            self.scanline[x] = color;

            offset_x += 1;
            if offset_x >= 8 {
                offset_x = 0;
                tile_index_x += 1;

                if should_render_window {
                    tile = self.fetch_window_tile(tile_index_x, tile_index_y, offset_y);
                } else {
                    tile = self.fetch_background_tile(tile_index_x, tile_index_y, offset_y);
                }
            }
        }
    }

    fn fetch_background_tile(&self, tile_index_x: u8, tile_index_y: u8, offset_y: u8) -> (u8, u8) {
        // calculate tile map address
        let tile_map_base: u16 = 0x1800;

        let tile_map_address =
            tile_map_base | ((tile_index_x as u16) % 32 + (tile_index_y as u16) * 32);

        // get tile no (0 ~ 255)
        let tile_number = self.vram[tile_map_address as usize];

        // calculate tile set address
        let tile_set_address = (tile_number as u16) * 16;

        // 1 tile 2 bytes
        let row_address = tile_set_address + (offset_y << 1) as u16;

        let tile0 = self.vram[row_address as usize];
        let tile1 = self.vram[(row_address + 1) as usize];

        return (tile0, tile1);
    }

    fn fetch_window_tile(&self, tile_index_x: u8, tile_index_y: u8, offset_y: u8) -> (u8, u8) {
        // calculate tile map address
        //  LCDC: Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
        let tile_map_base: u16 = 0x1800;

        let tile_map_address =
            tile_map_base | ((tile_index_x as u16) % 32 + (tile_index_y as u16) * 32);

        // get tile no (0 ~ 255)
        let tile_number = self.vram[tile_map_address as usize];

        // calculate tile set address
        let tile_set_address = (tile_number as u16) * 16;

        // 1 tile 2 bytes
        let row_address = tile_set_address + (offset_y << 1) as u16;

        let tile0 = self.vram[row_address as usize];
        let tile1 = self.vram[(row_address + 1) as usize];

        return (tile0, tile1);
    }

    fn redner_scanline(&mut self) {
        // update scanline
        self.render_background();

        for x in 0..WIDTH {
            let index = (x as usize) + (self.ly as usize) * WIDTH;
            let color_no = self.scanline[x];
            self.buffer[index] = self.color_no_to_rgb(color_no);
        }
    }

    fn color_no_to_rgb(&self, no: u8) -> u32 {
        match no {
            0 => DARKEST_GREEN,
            1 => DARK_GREEN,
            2 => LIGHT_GREEN,
            3 => LIGHTEST_GREEN,
            _ => panic!("unrecognized color no #{:?}", no),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => {
                // 0x0800 & 0x1fff -> 0x0000 means tile set 1
                // 0x0900 & 0x1fff -> 0x1000 means tile set 0
                return self.vram[(address & 0x1fff) as usize];
            }
            0xff40 => return self.lcdc,
            0xff42 => return self.scy,
            0xff43 => return self.scx,
            0xff44 => return self.ly,
            0xff4a => return self.wy,
            0xff4b => return self.wx,

            _ => panic!("unexpected address #{:x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9fff => {
                self.vram[(address & 0x1fff) as usize] = value;
            }
            0xff40 => self.lcdc = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => self.ly = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,
            _ => panic!("unexpected address #{:x}", address),
        }
    }
}
