use crate::mmu::MMU;
use std::fmt;

mod clock;
mod instruction;
mod opcode;
mod operation;
mod register;

pub struct CPU {
    pub mmu: MMU,
    pc: u16,
    sp: u16,
    t: usize, // T-cycle
    // m: usize, // M-cycle
    // Interrupt Master Enable Flag
    ime: bool,
    debug: bool,
    halt: bool,

    af: register::Register,
    bc: register::Register,
    de: register::Register,
    hl: register::Register,
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPU {{ A: {:#X}, F: {:#X} B: {:#X}, C: {:#X}, D: {:#X}, E: {:#X}, H: {:#X}, L: {:#X} }} \nflags: {{ Z: {:?}, N: {:?}, H: {:?}, C: {:?} }}\n{{ pc: {:#X}, sp: {:#X} }}\n{{t: {:#X}}}",
            self.af.high(),
            self.af.low(),
            self.bc.high(),
            self.bc.low(),
            self.de.high(),
            self.de.low(),
            self.hl.high(),
            self.hl.low(),
            self.get_z_flag(),
            self.get_n_flag(),
            self.get_h_flag(),
            self.get_c_flag(),
            self.pc,
            self.sp,
            self.t,
        )
    }
}

impl CPU {
    pub fn new(boot_rom_name: &str, rom_name: &str) -> CPU {
        return CPU {
            mmu: MMU::new(boot_rom_name, rom_name),
            pc: 0,
            sp: 0,
            t: 0,
            ime: false,
            debug: false,
            halt: false,
            af: register::Register::new(0, 0xfff0),
            bc: register::Register::new(0, 0),
            de: register::Register::new(0, 0),
            hl: register::Register::new(0, 0),
        };
    }

    pub fn step(&mut self) -> usize {
        self.t = 0;
        if self.halt {
            if self.mmu.interrupt_flag > 0 {
                self.halt = false;
            } else {
                self.t += 4;
            }
        } else {
            self.fetch_and_execute();
        }

        self.mmu.step(self.t);

        // self.mmu.ppu.debug = true;
        // check for interrupts
        if self.ime {
            for i in 0..5 {
                let irq = (self.mmu.interrupt_flag & (0x01 << i)) > 0;
                let ie = (self.mmu.interrupt_enable & (0x01 << i)) > 0;

                if irq && ie {
                    // call isr(Interrupt Serivce Routine)
                    // disable further interrupts
                    self.ime = false;
                    // stop halting because interrupt occurs
                    self.halt = false;
                    // reset interrupt reqeust flag
                    self.mmu.interrupt_flag &= !(0x01 << i);

                    self.sp = self.sp.wrapping_sub(2);
                    self.write_byte16(self.sp, self.pc);
                    self.pc = match i {
                        // 0: V-Blank handler
                        0 => 0x40,
                        // 1: LCDC
                        1 => 0x48,
                        2 => 0x50,
                        3 => 0x58,
                        4 => 0x60,
                        _ => panic!("unrecognized irq {}", i),
                    };

                    self.t += 12;

                    break;
                }
            }
        }

        return self.t;
    }

    pub fn fetch_and_execute(&mut self) {
        if self.mmu.boot_rom_enabled {
            if self.pc == 0x0100 {
                self.af.set(0);
                self.bc.set(0);
                self.de.set(0);
                self.hl.set(0);
            }

            // println!(
            //     "CPU {{ A: {:#X}, F: {:#X} B: {:#X}, C: {:#X}, D: {:#X}, E: {:#X}, H: {:#X}, L: {:#X} }} \nflags: {{ Z: {:?}, N: {:?}, H: {:?}, C: {:?} }}\nsp: {:#X} }}\n{{t: {:#X}}}",
            //     self.af.high(),
            //     self.af.low(),
            //     self.bc.high(),
            //     self.bc.low(),
            //     self.de.high(),
            //     self.de.low(),
            //     self.hl.high(),
            //     self.hl.low(),
            //     self.get_z_flag(),
            //     self.get_n_flag(),
            //     self.get_h_flag(),
            //     self.get_c_flag(),
            //     self.sp,
            //     self.t,
            // )
        }

        let opcode = opcode::Opcode::new(self.pop_pc());
        if self.mmu.boot_rom_enabled {
            // println!("pc {:#X} instructions {:#X}", self.pc - 1, instruction);
        }

        instruction::execute(&opcode, self);
        self.t = self.t.wrapping_add(opcode.clock() as usize);
    }

    fn read_byte(&self, address: u16) -> u8 {
        return self.mmu.read_byte(address);
    }

    fn read_byte16(&self, address: u16) -> u16 {
        let low = self.mmu.read_byte(address);
        let high = self.mmu.read_byte(address.wrapping_add(1));

        return ((high as u16) << 8) | low as u16;
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        self.mmu.write_byte(address, value);
    }

    fn write_byte16(&mut self, address: u16, value: u16) {
        // little endian
        self.mmu.write_byte(address, (value & 0x00ff) as u8);
        let next = address.wrapping_add(1);
        self.mmu.write_byte(next, (value >> 8) as u8);
    }

    fn pop_pc(&mut self) -> u8 {
        let v = self.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        return v;
    }

    fn pop_pc16(&mut self) -> u16 {
        let v = self.read_byte16(self.pc);
        self.pc = self.pc.wrapping_add(2);
        return v;
    }

    fn get_flag(&self, bit_mask: u8) -> bool {
        (self.af.low() & bit_mask) != 0
    }
    fn get_z_flag(&self) -> bool {
        self.get_flag(0b1000_0000)
    }
    fn get_n_flag(&self) -> bool {
        self.get_flag(0b0100_0000)
    }
    fn get_h_flag(&self) -> bool {
        self.get_flag(0b0010_0000)
    }
    fn get_c_flag(&self) -> bool {
        self.get_flag(0b0001_0000)
    }

    fn set_z_flag(&mut self) {
        self.af.set_low(self.af.low() | 0b1000_0000);
    }

    fn reset_z_flag(&mut self) {
        self.af.set_low(self.af.low() & 0b0111_1111);
    }
    fn set_n_flag(&mut self) {
        self.af.set_low(self.af.low() | 0b0100_0000);
    }
    fn reset_n_flag(&mut self) {
        self.af.set_low(self.af.low() & 0b1011_1111);
    }
    fn set_h_flag(&mut self) {
        self.af.set_low(self.af.low() | 0b0010_0000);
    }
    fn reset_h_flag(&mut self) {
        self.af.set_low(self.af.low() & 0b1101_1111);
    }
    fn set_c_flag(&mut self) {
        self.af.set_low(self.af.low() | 0b0001_0000);
    }
    fn reset_c_flag(&mut self) {
        self.af.set_low(self.af.low() & 0b1110_1111);
    }

    fn set_z_flag_if(&mut self, condition: bool) {
        if condition {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
    }

    fn set_n_flag_if(&mut self, condition: bool) {
        if condition {
            self.set_n_flag();
        } else {
            self.reset_n_flag();
        }
    }

    fn set_h_flag_if(&mut self, condition: bool) {
        if condition {
            self.set_h_flag();
        } else {
            self.reset_h_flag();
        }
    }

    fn set_c_flag_if(&mut self, condition: bool) {
        if condition {
            self.set_c_flag();
        } else {
            self.reset_c_flag();
        }
    }
}
