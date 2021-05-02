use std::fmt;

pub struct PPU {
    mode: u8,
    // 8KB Video RAM(VRAM)
    vram: Vec<u8>,
    // OAM
    // from $FE00-$FE9F
    oam: Vec<u8>,
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
    // BGP - BG Palette Data
    bgp: u8,
    scanline: [u8; WIDTH],
    // VBlank
    pub vblank: bool,
    pub debug: bool,
}

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub const VRAM_ADDRESS_BASE: u16 = 0x8000;

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
            oam: vec![0; 0xa0],
            buffer: vec![DARKEST_GREEN; WIDTH * HEIGHT],
            clocks: 0,
            lcdc: 0b1000_0000,
            scx: 0,
            scy: 0,
            ly: 0,
            wx: 0,
            wy: 0,
            bgp: 0,
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
        // TODO: lcdc
        let tile_map_base: u16 = 0x9800 - VRAM_ADDRESS_BASE;

        // each pixel in scanline
        for x in 0..WIDTH {
            let pixel_x = self.scx + x as u8;
            let pixel_y = self.scy.wrapping_add(self.ly);

            let index_x = pixel_x / 8;
            let index_y = pixel_y / 8;

            // offsets within tile (0 ~ 7)
            let offset_x = pixel_x % 8;
            let mut offset_y = pixel_y % 8;

            // LCDC: Bit 5 - Window Display Enable          (0=Off, 1=On)
            if self.lcdc & 0b0010_0000 > 0 {
                // top left corner of a window are wx-7,wy.
                if self.wy <= self.ly && self.wx == (x + 7) as u8 {
                    offset_y = (self.ly - self.wy) % 8;
                }
            }

            // calculate tile map address
            let tile_map_address = tile_map_base | ((index_x as u16) % 32 + (index_y as u16) * 32);

            // get the tile No
            let tile_no = self.vram[tile_map_address as usize];

            // calculate tile set address
            let tile_set_address = 0x8000 + (tile_no as u16) * 16 - VRAM_ADDRESS_BASE;

            // calculate tile data address
            let tile_data_address = tile_set_address + (offset_y * 2) as u16;

            // get tile data
            let tile_data_low = self.vram[tile_data_address as usize];
            let tile_data_high = self.vram[(tile_data_address + 1) as usize];

            let mask = 1 << (7 - offset_x);
            let lsb = tile_data_low & mask;
            let msb = tile_data_high & mask;

            let palette_no = match (lsb != 0, msb != 0) {
                (true, true) => 3,
                (false, true) => 2,
                (true, false) => 1,
                (false, false) => 0,
            };

            // Bit 7-6 - Shade for Color Number 3
            // Bit 5-4 - Shade for Color Number 2
            // Bit 3-2 - Shade for Color Number 1
            // Bit 1-0 - Shade for Color Number 0
            let color_no = (self.bgp >> (palette_no * 2)) & 0x03;

            self.scanline[x] = color_no;
        }
    }

    fn render_sprites(&mut self) {
        // ref. https://hacktix.github.io/GBEDG/ppu/#sprites

        // 40 sprites

        for i in 0..40 {
            // OAM
            // Byte 0: Y-Position
            // Byte 1: X-Position
            // Byte 2: Tile Number
            // Byte 3: Sprite Flags

            let address = i * 4;
            if self.oam[address] == 0 {
                continue;
            }
            let x = self.oam[address] - 8;
            let y = self.oam[address + 1] - 16;
            let tile_number = self.oam[address + 2];

            if self.ly < y || y + 8 <= self.ly {
                continue;
            }

            // render tile(8 x 8 pixels)
            // offset within tile
            let offset_y = (self.ly - y) % 8;

            let tile_set_address = (tile_number as u16) * 16;

            // 1 tile 2 bytes
            let row_address = tile_set_address + (offset_y << 1) as u16;

            let tile0 = self.vram[row_address as usize];
            let tile1 = self.vram[(row_address + 1) as usize];

            for offset_x in 0..8 {
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

                self.scanline[(x + offset_x) as usize] = color;
            }
        }
    }

    fn redner_scanline(&mut self) {
        // update scanline

        // BG/Window Display 0=off 1=on
        if self.lcdc & 0x01 > 0 {
            self.render_background();
        }

        // Sprite Display Enable
        if self.lcdc & 0x02 > 0 {
            self.render_sprites();
        }

        for x in 0..WIDTH {
            let index = (x as usize) + (self.ly as usize) * WIDTH;
            let color_no = self.scanline[x];
            self.buffer[index] = self.color_no_to_rgb(color_no);
        }
    }

    fn color_no_to_rgb(&self, no: u8) -> u32 {
        match no {
            0 => LIGHTEST_GREEN,
            1 => LIGHT_GREEN,
            2 => DARK_GREEN,
            3 => DARKEST_GREEN,
            _ => panic!("unrecognized color no #{:?}", no),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x8000..=0x9fff => {
                // 0x0800 & 0x1fff -> 0x0000 means tile set 1
                // 0x0900 & 0x1fff -> 0x1000 means tile set 0
                return self.vram[(address - VRAM_ADDRESS_BASE) as usize];
            }

            // 0AM
            0xfe00..=0xfe9f => {
                return self.oam[(address & 0x00ff) as usize];
            }

            0xff40 => return self.lcdc,
            0xff42 => return self.scy,
            0xff43 => return self.scx,
            0xff44 => return self.ly,
            0xff47 => return self.bgp,
            0xff4a => return self.wy,
            0xff4b => return self.wx,

            // TODO:
            0xff41 => {
                panic!("stat read");
            }
            0xff45 => {
                panic!("lyc read");
            }
            0xff48 => {
                panic!("obp0 read");
            }
            0xff49 => {
                panic!("obp1 read");
            }

            _ => panic!("unexpected address #{:x}", address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9fff => {
                self.vram[(address - VRAM_ADDRESS_BASE) as usize] = value;
            }

            // OAM
            0xfe00..=0xfe9f => {
                self.oam[(address & 0x00ff) as usize] = value;
            }

            0xff40 => self.lcdc = value,
            0xff42 => self.scy = value,
            0xff43 => self.scx = value,
            0xff44 => self.ly = value,
            0xff47 => self.bgp = value,
            0xff4a => self.wy = value,
            0xff4b => self.wx = value,

            // TODO:
            0xff41 => {
                println!("stat write {:X}", value);
            }
            0xff45 => {
                println!("lyc write {:X}", value);
            }
            0xff48 => {
                println!("obp0 write {:X}", value);
            }
            0xff49 => {
                println!("obp1 write {:X}", value);
            }

            _ => panic!("unexpected address #{:x}", address),
        }
    }
}
