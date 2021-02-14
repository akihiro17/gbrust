use std::fs::File;
use std::io::Read;

mod banking_controller;
mod mbc1;

pub struct Catridge {
    banking_controller: Box<dyn banking_controller::BankingController>,
}

impl Catridge {
    pub fn new(name: &str) -> Self {
        let mut file = File::open(name).unwrap();
        let mut rom = Vec::<u8>::new();

        file.read_to_end(&mut rom).unwrap();

        // カートリッジヘッダ
        // https://w.atwiki.jp/gbspec/pages/30.html

        let cartridge_type = rom[0x147];
        if cartridge_type != 0x01 {
            //  panic!("not supported catridge type {:#X}", cartridge_type);
        }

        // let number_of_rom_banks =

        // 0149 - RAM サイズ
        let ram_size = match rom[0x0149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024,
            _ => panic!("invalid ram size {}", rom[0x0149]),
        };

        Catridge {
            banking_controller: Box::new(mbc1::MBC1::new(rom, ram_size)),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        return self.banking_controller.read(address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.banking_controller.write(address, value);
    }
}
