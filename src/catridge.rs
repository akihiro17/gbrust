use std::fs::File;
use std::io::Read;

mod mbc1;
mod no_mbc;

enum CatridgeType {
    NoMbc,
    Mbc1,
}

pub struct Catridge {
    cartridge_type: CatridgeType,
    rom: Vec<u8>,
    rom_bank: u8,

    ram: Vec<u8>,
    ram_enabled: bool,
    ram_bank: u8,

    rom_banking: bool,
}

impl Catridge {
    pub fn new(name: &str) -> Self {
        let mut file = File::open(name).unwrap();
        let mut rom = Vec::<u8>::new();

        file.read_to_end(&mut rom).unwrap();

        // カートリッジヘッダ
        // https://w.atwiki.jp/gbspec/pages/30.html

        let cartridge_type = match rom[0x147] {
            0x00 => CatridgeType::NoMbc,
            0x01 => CatridgeType::Mbc1,
            _ => panic!("not supported catridge type {:#X}", rom[0x147]),
        };

        // 0149 - RAM サイズ
        let ram_size = match rom[0x0149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            _ => panic!("invalid ram size {}", rom[0x0149]),
        };

        Catridge {
            cartridge_type,
            rom,
            rom_bank: 1,
            ram: vec![0; ram_size as usize],
            ram_enabled: false,
            ram_bank: 0,
            rom_banking: false,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match &self.cartridge_type {
            CatridgeType::NoMbc => no_mbc::read(self, address),
            CatridgeType::Mbc1 => mbc1::read(self, address),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match &self.cartridge_type {
            CatridgeType::NoMbc => no_mbc::write(self, address, value),
            CatridgeType::Mbc1 => mbc1::write(self, address, value),
        }
    }

    fn update_rom_bank(&mut self) {
        // When 00h is written, the MBC translates that to bank 01h also
        // the same happens for Bank 20h, 40h, and 60h
        match self.rom_bank {
            0x00 | 0x20 | 0x40 | 0x60 => {
                self.rom_bank += 1;
            }
            _ => {}
        }
    }
}
