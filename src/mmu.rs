use std::fs::File;
use std::io::Read;

use crate::ppu::PPU;

pub struct MMU {
    boot_rom: Vec<u8>, // 0x0000 to 0x00FF
    rom: Vec<u8>,
    ram: [u8; 65536], // 0x0000 to 0xffff
    pub ppu: PPU,
}

impl MMU {
    pub fn new(boot_rom_name: &str, rom_name: &str) -> MMU {
        let mut file = File::open(rom_name).unwrap();
        let mut rom = Vec::<u8>::new();

        file.read_to_end(&mut rom).unwrap();

        for &byte in rom.iter() {
            println!("{:#x}", (byte as u16));
        }

        let mut boot_rom_file = File::open(boot_rom_name).unwrap();
        let mut boot_rom = Vec::<u8>::new();

        boot_rom_file.read_to_end(&mut boot_rom).unwrap();

        for &byte in rom.iter() {
            println!("{:#x}", (byte as u16));
        }

        return MMU {
            boot_rom: boot_rom,
            rom: rom,
            ram: [0; 65536],
            ppu: PPU::new(),
        };
    }

    pub fn step(&mut self, clocks: usize) {
        self.ppu.step(clocks);
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
            0xff42 | 0xff44 => {
                self.ppu.write(address, value);
            }
            _ => {
                self.ram[address as usize] = value;
            }
        }
    }

    pub fn read_byte(&mut self, address: u16) -> u8 {
        match address {
            0x0000..=0x00ff => {
                return self.boot_rom[address as usize];
            }
            0x0100..=0x7fff => {
                return self.rom[address as usize];
            }
            // PPU
            0x8000..=0x9fff => {
                return self.ppu.read(address);
            }
            0xff42 | 0xff43 | 0xff44 => {
                return self.ppu.read(address);
            }
            _ => {
                return self.ram[address as usize];
            }
        }
    }
}
