use crate::catridge;

// ref. https://gbdev.gg8.se/wiki/articles/Memory_Bank_Controllers

pub fn read(catridge: &catridge::Catridge, address: u16) -> u8 {
    match address {
        0x0000..=0x3fff => catridge.rom[address as usize],
        // ROM Bank 01-7F (Read Only)
        0x4000..=0x7fff => {
            // the size of a bank is 16KB
            let rom_offset = (16 * 1024) * catridge.rom_bank as u16;
            catridge.rom[(rom_offset as usize) + ((address - 0x4000) as usize)]
        }
        // RAM Bank 00-03, if any
        0xa000..=0xbfff => {
            if !catridge.ram_enabled {
                return 0xff;
            }

            let ram_offset = (8 * 1024) * catridge.ram_bank as u16;
            catridge.ram[(ram_offset as usize) + (address - 0xa000) as usize]
        }
        _ => panic!("invalid catridge read access {:#X}", address),
    }
}

pub fn write(catridge: &mut catridge::Catridge, address: u16, value: u8) {
    match address {
        // RAM Enable (Write Only)
        0x0000..=0x1fff => {
            // any value with 0Ah in the lower 4 bits enables RAM, and any other value disables RAM
            catridge.ram_enabled = (value & 0x0f) == 0x0a;
        }
        // ROM Bank Number (Write Only)
        0x2000..=0x3fff => {
            // Writing to this address space selects the lower 5 bits of the ROM Bank Number (in range 01-1Fh).
            catridge.rom_bank = (catridge.rom_bank & 0xe0) | (value & 0x1f);
            catridge.update_rom_bank();
        }
        // RAM Bank Number - or - Upper Bits of ROM Bank Number (Write Only)
        0x4000..=0x5fff => {
            // This 2bit register can be used to select a RAM Bank in range from 00-03h,
            // or to specify the upper two bits (Bit 5-6) of the ROM Bank number
            if catridge.rom_banking {
                catridge.rom_bank = (catridge.rom_bank & 0x1f) | (value & 0xe0);
                catridge.update_rom_bank();
            } else {
                catridge.ram_bank = value & 0x03;
            }
        }
        // ROM/RAM Mode Select (Write Only)
        0x6000..=0x7fff => {
            // 0x0: ROM Banking mode
            // 0x1: RAM Banking mode
            catridge.rom_banking = (value & 0x01) == 0;
            if catridge.rom_banking {
                catridge.ram_bank = 0;
            }
        }
        // RAM Bank 00-03, if any
        0xa000..=0xbfff => {
            if !catridge.ram_enabled {
                return;
            }

            let ram_offset = (8 * 1024) * catridge.ram_bank as u16;
            catridge.ram[(ram_offset as usize) + (address - 0xa000) as usize] = value;
        }

        _ => panic!("invalid catridge write access {:#X}", address),
    }
}
