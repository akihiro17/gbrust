use crate::mmu::MMU;
use std::fmt;

mod clock;
mod instructions;

#[derive(Debug)]
enum Register {
    A,
    B,
    C,
    D,
    E,
    F,
    H,
    L,
}

#[derive(Debug)]
enum Register16 {
    AF,
    BC,
    DE,
    HL,
    SP,
}

pub struct Reg {
    // The value of the register
    value: u16,

    mask: u16,
}

impl Reg {
    pub fn new(value: u16, mask: u16) -> Self {
        return Reg {
            value: value,
            mask: mask,
        };
    }

    pub fn high(&self) -> u8 {
        return (self.value >> 8) as u8;
    }

    pub fn low(&self) -> u8 {
        return (self.value & 0x00ff) as u8;
    }

    pub fn value(&self) -> u16 {
        return self.value;
    }

    pub fn set_high(&mut self, value: u8) {
        self.value = (self.value & 0x00ff) | ((value as u16) << 8);

        if self.mask != 0 {
            self.value &= self.mask;
        }
    }

    pub fn set_low(&mut self, value: u8) {
        self.value = (self.value & 0xff00) | (value as u16);

        if self.mask != 0 {
            self.value &= self.mask;
        }
    }

    pub fn set(&mut self, value: u16) {
        self.value = value;

        if self.mask != 0 {
            self.value &= self.mask;
        }
    }
}

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

    af: Reg,
    bc: Reg,
    de: Reg,
    hl: Reg,
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
            af: Reg::new(0, 0xfff0),
            bc: Reg::new(0, 0),
            de: Reg::new(0, 0),
            hl: Reg::new(0, 0),
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

        let opecode = self.pop_pc();
        if self.mmu.boot_rom_enabled {
            // println!("pc {:#X} instructions {:#X}", self.pc - 1, instruction);
        }

        instructions::execute(opecode, self);
        self.t = self.t.wrapping_add(clock::clock(opecode) as usize);
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

    fn read_r8(&self, register: &Register) -> u8 {
        match register {
            Register::A => {
                return self.af.high();
            }
            Register::B => {
                return self.bc.high();
            }
            Register::C => {
                return self.bc.low();
            }
            Register::D => {
                return self.de.high();
            }
            Register::E => {
                return self.de.low();
            }
            Register::F => {
                return self.af.low();
            }
            Register::H => {
                return self.hl.high();
            }
            Register::L => {
                return self.hl.low();
            }
        }
    }

    fn write_r8(&mut self, register: &Register, value: u8) {
        match register {
            Register::A => {
                self.af.set_high(value);
            }
            Register::B => {
                self.bc.set_high(value);
            }
            Register::C => {
                self.bc.set_low(value);
            }
            Register::D => {
                self.de.set_high(value);
            }
            Register::E => {
                self.de.set_low(value);
            }
            Register::F => {
                self.af.set_low(value);
            }
            Register::H => {
                self.hl.set_high(value);
            }
            Register::L => {
                self.hl.set_low(value);
            }
        }
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

    fn ld_from_r8_to_r8(&mut self, r1: &Register, r2: &Register) {
        let value = self.read_r8(r2);
        self.write_r8(r1, value);
    }

    fn ld_from_r8_to_m8(&mut self, address: u16, r: &Register) {
        let value = self.read_r8(r);
        self.write_byte(address, value);

        // println!("lD address({:x}) = {}", address, value);
    }

    fn ld_from_d16_to_m8(&mut self, address: u16) {
        let value = self.pop_pc();
        self.write_byte(address, value);

        // println!("lD address({:x}) = {}", address, value);
    }

    fn ld_from_r8_to_d16(&mut self, address: u16, r: &Register) {
        let value = self.read_r8(r);
        self.write_byte(address, value);
    }

    fn ld_from_memory_to_r8(&mut self, r1: &Register, address: u16) {
        let value = self.read_byte(address);
        self.write_r8(r1, value);
    }

    fn ld_from_d16_to_r8(&mut self, r1: &Register) {
        let d16 = self.pop_pc16();
        let value = self.read_byte(d16);
        self.write_r8(r1, value);
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

    fn get_af(&self) -> u16 {
        return self.af.value();
    }

    fn set_af(&mut self, value: u16) {
        self.af.set(value);
    }

    fn get_bc(&self) -> u16 {
        return self.bc.value();
    }

    fn set_bc(&mut self, value: u16) {
        self.bc.set(value);
    }

    fn get_de(&self) -> u16 {
        return self.de.value();
    }

    fn set_de(&mut self, value: u16) {
        self.de.set(value);
    }

    fn get_hl(&self) -> u16 {
        return self.hl.value();
    }

    fn set_hl(&mut self, value: u16) {
        self.hl.set(value);
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
