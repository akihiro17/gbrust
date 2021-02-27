use std::fs::File;
use std::io::Read;

use crate::catridge::Catridge;
use crate::ppu::PPU;
use crate::timer::Timer;

pub struct MMU {
    boot_rom: Vec<u8>, // 0x0000 to 0x00FF
    catridge: Catridge,
    ram: [u8; 65536], // 0x0000 to 0xffff
    /// High RAM
    hram: [u8; 0x7f],
    pub ppu: PPU,
    pub timer: Timer,
    pub boot_rom_enabled: bool,
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,

    pub serial_port: String,
}

impl MMU {
    pub fn new(boot_rom_name: &str, rom_name: &str) -> Self {
        let mut file = File::open(rom_name).unwrap();
        let mut rom = Vec::<u8>::new();

        file.read_to_end(&mut rom).unwrap();

        let mut boot_rom_file = File::open(boot_rom_name).unwrap();
        let mut boot_rom = Vec::<u8>::new();
        boot_rom_file.read_to_end(&mut boot_rom).unwrap();

        return MMU {
            boot_rom: boot_rom,
            catridge: Catridge::new(rom_name),
            ram: [0; 65536],
            hram: [0; 0x7f],
            ppu: PPU::new(),
            timer: Timer::new(),
            boot_rom_enabled: true,
            interrupt_flag: 0,
            interrupt_enable: 0,
            serial_port: "".to_string(),
        };
    }

    pub fn step(&mut self, clocks: usize) {
        self.ppu.step(clocks);
        self.timer.step(clocks);

        // V-Blank interrupt Request
        if self.ppu.vblank {
            self.interrupt_flag |= 0x01;
            self.ppu.vblank = false;
        }

        // Timer interrupt Request
        if self.timer.irq {
            self.interrupt_flag |= 0x04;
            self.timer.irq = false;
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // rom
            0x0000..=0x00ff => {
                self.catridge.write(address, value);
            }

            // rom
            0x0100..=0x7fff => {
                self.catridge.write(address, value);
            }

            // PPU
            // VRAM
            0x8000..=0x9fff => {
                self.ppu.write(address, value);
            }

            // External RAM
            0xa000..=0xbfff => {
                self.catridge.write(address, value);
            }

            // OAM
            0xfe00..=0xfe9f => {
                self.ppu.write(address, value);
            }

            // for console
            0xff01 => {
                self.serial_port.push(value as char);
                // print!("{}", value as char);
            }

            // I/O Registers
            0xff40 | 0xff42 | 0xff43 | 0xff44 | 0xff47 | 0xff4a | 0xff4b => {
                self.ppu.write(address, value);
            }

            // DMA
            0xff46 => {
                let xx = (value as u16 & 0x00ff) << 8;
                for i in 0..0xa0 {
                    // source
                    let v = self.read_byte(xx | i);
                    // dest: oam
                    self.write_byte(0xfe00 | i, v);
                }
            }

            // Timer
            0xff04..=0xff07 => self.timer.write_byte(address, value),

            // Interrupt Flag
            0xff0f => {
                self.interrupt_flag = value;
            }
            // Interrupt Enable
            0xffff => {
                self.interrupt_enable = value;
            }

            0xff50 => {
                // Reset boot rom
                self.boot_rom_enabled = false;
            }

            // HRAM
            0xff80..=0xfffe => self.hram[(address & 0x7f) as usize] = value,

            _ => {
                if address == 0xff46 {
                    panic!("should implement dma");
                }
                self.ram[address as usize] = value;
            }
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x00ff => {
                if self.boot_rom_enabled {
                    return self.boot_rom[address as usize];
                }

                return self.catridge.read(address);
            }

            // ROM
            0x0100..=0x7fff => {
                // return self.rom[address as usize];
                return self.catridge.read(address);
            }

            // PPU
            0x8000..=0x9fff => {
                return self.ppu.read(address);
            }

            // RAM
            // External RAM
            0xa000..=0xbfff => {
                return self.catridge.read(address);
            }

            // OAM
            0xfe00..=0xfe9f => {
                return self.ppu.read(address);
            }

            0xff40 | 0xff42 | 0xff43 | 0xff44 | 0xff47 | 0xff4a | 0xff4b | 0xff50 => {
                return self.ppu.read(address);
            }

            // Timer
            0xff04..=0xff07 => self.timer.read_byte(address),

            // Interrupt Flag
            0xff0f => {
                return self.interrupt_flag;
            }
            // Interrupt Enable
            0xffff => {
                return self.interrupt_enable;
            }

            // HRAM
            0xff80..=0xfffe => self.hram[(address & 0x7f) as usize],

            _ => {
                return self.ram[address as usize];
                // 0xff
            }
        }
    }
}
