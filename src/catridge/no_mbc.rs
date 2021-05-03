use crate::catridge;

// ref. https://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers

pub fn read(catridge: &catridge::Catridge, address: u16) -> u8 {
    catridge.rom[address as usize]
}

pub fn write(_catridge: &mut catridge::Catridge, _address: u16, _value: u8) {
    // only rom
}
