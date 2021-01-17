use std::fs::File;
use std::io::Read;

use crate::ppu::PPU;

pub struct MMU {
    boot_rom: Vec<u8>, // 0x0000 to 0x00FF
    rom: Vec<u8>,
    ram: [u8; 65536], // 0x0000 to 0xffff
    pub ppu: PPU,
    boot_rom_enabled: bool,
    pub interrupt_flag: u8,
    pub interrupt_enable: u8,
}

impl MMU {
    pub fn new(boot_rom_name: &str, rom_name: &str) -> MMU {
        let mut file = File::open(rom_name).unwrap();
        let mut rom = Vec::<u8>::new();

        file.read_to_end(&mut rom).unwrap();

        for &byte in rom.iter() {
            // println!("{:#x}", (byte as u16));
        }

        let mut boot_rom_file = File::open(boot_rom_name).unwrap();
        let mut boot_rom = Vec::<u8>::new();

        boot_rom_file.read_to_end(&mut boot_rom).unwrap();

        for &byte in rom.iter() {
            // println!("{:#x}", (byte as u16));
        }

        return MMU {
            boot_rom: boot_rom,
            rom: rom,
            ram: [0; 65536],
            ppu: PPU::new(),
            boot_rom_enabled: true,
            interrupt_flag: 0,
            interrupt_enable: 0,
        };
    }

    pub fn step(&mut self, clocks: usize) {
        self.ppu.step(clocks);

        // V-Blank interrupt Request
        if self.ppu.vblank {
            self.interrupt_flag |= 0x01;
            self.ppu.vblank = false;
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            // boot rom
            0x0000..=0x00ff => {
                self.boot_rom[address as usize] = value;
            }
            // rom
            0x0100..=0x7fff => {
                self.rom[address as usize] = value;
            }
            // PPU
            // VRAM
            0x8000..=0x9fff => {
                self.ppu.write(address, value);
            }
            // OAM
            0xfe00..=0xfe9f => {
                self.ppu.write(address, value);
            }

            // for console
            0xff01 => {
                // println!("console: {}", value as char);
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

                return self.rom[address as usize];
            }
            0x0100..=0x7fff => {
                return self.rom[address as usize];
            }
            // PPU
            0x8000..=0x9fff => {
                return self.ppu.read(address);
            }

            // OAM
            0xfe00..=0xfe9f => {
                return self.ppu.read(address);
            }

            0xff40 | 0xff42 | 0xff43 | 0xff44 | 0xff47 | 0xff4a | 0xff4b | 0xff50 => {
                return self.ppu.read(address);
            }

            // Interrupt Flag
            0xff0f => {
                return self.interrupt_flag;
            }
            // Interrupt Enable
            0xffff => {
                return self.interrupt_enable;
            }

            _ => {
                return self.ram[address as usize];
            }
        }
    }
}
