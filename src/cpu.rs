use crate::mmu::MMU;
use std::fmt;

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

struct Reg {
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

    pub fn high_low(&self) -> u16 {
        return self.value;
    }

    pub fn set_high(&mut self, value: u8) {
        self.value = (value as u16) << 8 | (self.value & 0x00ff) as u16;

        if self.mask != 0 {
            self.value &= self.mask;
        }
    }

    pub fn set_low(&mut self, value: u8) {
        self.value = (self.value & 0xff00) | (value as u16) & 0x00ff;

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
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
    t: usize, // T-cycle
    m: usize, // M-cycle
    // Interrupt Master Enable Flag
    ime: bool,
    debug: bool,
    halt: bool,

    af: Reg,
}

impl fmt::Debug for CPU {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CPU {{ A: {:#X}, F: {:#X} B: {:#X}, C: {:#X}, D: {:#X}, E: {:#X}, H: {:#X}, L: {:#X} }} \nflags: {{ Z: {:?}, N: {:?}, H: {:?}, C: {:?} }}\n{{ pc: {:#X}, sp: {:#X} }}\n{{t: {:#X}}}",
            self.a,
            self.f,
            self.b,
            self.c,
            self.d,
            self.e,
            self.h,
            self.l,
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
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0,
            sp: 0,
            t: 0,
            m: 0,
            ime: false,
            debug: false,
            halt: false,
            af: Reg::new(0, 0xfff0),
        };
    }

    pub fn step(&mut self) -> usize {
        self.t = 0;
        if self.halt {
            if self.mmu.interrupt_flag > 0 {
                self.halt = false;
            } else {
                self.t += 4;
                self.m += 1;
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

                // println!("i: {} irq: {:} ie: {:}", i, irq, ie);

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
                    self.m += 3;

                    break;
                }
            }
        }

        return self.t;
    }

    pub fn fetch_and_execute(&mut self) {
        let instruction = self.mmu.read_byte(self.pc);

        if !self.mmu.boot_rom_enabled {
            if self.pc == 0x0100 {
                self.a = 0;
                self.f = 0;
                self.b = 0;
                self.c = 0;
                self.d = 0;
                self.e = 0;
                self.h = 0;
                self.l = 0;
            }
            // println!(
            //     "byte on {:#X}: {:#X} inst: {:#X}",
            //     self.pc,
            //     self.mmu.read_byte(self.pc),
            //     instruction
            // );
            // println!("CPU STATE {:?}", self);
            // self.mmu.ppu.debug = true;
            // println!(
            //     "pc: {:#X} A: {:#X} F: {:#X} B: {:#X} C: {:#X} D: {:#X} E: {:#X} H: {:#X} L: {:#X}",
            //     self.pc, self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l,
            // );
            // if self.debug {
            // println!(
            //         "instr: {:#X} pc: {:#X} A: {:#X} F: {:#X} B: {:#X} C: {:#X} D: {:#X} E: {:#X} H: {:#X} L: {:#X}",
            //         instruction,
            //         self.pc, self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l,
            //     );
            // }
        }

        match instruction {
            // nop
            0x0 => {
                self.pc = self.pc.wrapping_add(1);
                self.t += 4;
                self.m += 1;
            }
            // ref. http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
            // 8-bit loads
            // 1. LD nn,n
            0x06 => {
                // LD B, n
                self.b = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }
            0x0e => {
                // LD C, n
                self.c = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }
            0x16 => {
                // LD D, n
                self.d = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }
            0x1e => {
                // LD E, n
                self.e = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }
            0x26 => {
                // LD h, n
                self.h = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }
            0x2e => {
                // LD L, n
                self.l = self.read_byte(self.pc + 1);
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }

            // 4. LD n,A
            // LD A, A
            0x7f => self.ld_from_r8_to_r8(&Register::A, &Register::A),
            0x47 => self.ld_from_r8_to_r8(&Register::B, &Register::A),
            0x4f => self.ld_from_r8_to_r8(&Register::C, &Register::A),
            0x57 => self.ld_from_r8_to_r8(&Register::D, &Register::A),
            0x5f => self.ld_from_r8_to_r8(&Register::E, &Register::A),
            0x67 => self.ld_from_r8_to_r8(&Register::H, &Register::A),
            0x6f => self.ld_from_r8_to_r8(&Register::L, &Register::A),
            0x02 => self.ld_from_r8_to_m8(self.get_bc(), &Register::A),
            0x12 => self.ld_from_r8_to_m8(self.get_de(), &Register::A),
            0x77 => self.ld_from_r8_to_m8(self.get_hl(), &Register::A),
            0xea => {
                // LD (nn), A
                let address = self.read_byte16(self.pc + 1);
                self.ld_from_r8_to_d16(address, &Register::A);
            }

            // 2. LD r1,r2
            0x78 => self.ld_from_r8_to_r8(&Register::A, &Register::B),
            0x79 => self.ld_from_r8_to_r8(&Register::A, &Register::C),
            0x7a => self.ld_from_r8_to_r8(&Register::A, &Register::D),
            0x7b => self.ld_from_r8_to_r8(&Register::A, &Register::E),
            0x7c => self.ld_from_r8_to_r8(&Register::A, &Register::H),
            0x7d => self.ld_from_r8_to_r8(&Register::A, &Register::L),
            0x7e => self.ld_from_memory_to_r8(&Register::A, self.get_hl()),
            0x0a => self.ld_from_memory_to_r8(&Register::A, self.get_bc()),
            0xfa => self.ld_from_d16_to_r8(&Register::A),

            0x40 => self.ld_from_r8_to_r8(&Register::B, &Register::B),
            0x41 => self.ld_from_r8_to_r8(&Register::B, &Register::C),
            0x42 => self.ld_from_r8_to_r8(&Register::B, &Register::D),
            0x43 => self.ld_from_r8_to_r8(&Register::B, &Register::E),
            0x44 => self.ld_from_r8_to_r8(&Register::B, &Register::H),
            0x45 => self.ld_from_r8_to_r8(&Register::B, &Register::L),
            0x46 => self.ld_from_memory_to_r8(&Register::B, self.get_hl()),

            0x48 => self.ld_from_r8_to_r8(&Register::C, &Register::B),
            0x49 => self.ld_from_r8_to_r8(&Register::C, &Register::C),
            0x4a => self.ld_from_r8_to_r8(&Register::C, &Register::D),
            0x4b => self.ld_from_r8_to_r8(&Register::C, &Register::E),
            0x4c => self.ld_from_r8_to_r8(&Register::C, &Register::H),
            0x4d => self.ld_from_r8_to_r8(&Register::C, &Register::L),
            0x4e => self.ld_from_memory_to_r8(&Register::C, self.get_hl()),

            0x50 => self.ld_from_r8_to_r8(&Register::D, &Register::B),
            0x51 => self.ld_from_r8_to_r8(&Register::D, &Register::C),
            0x52 => self.ld_from_r8_to_r8(&Register::D, &Register::D),
            0x53 => self.ld_from_r8_to_r8(&Register::D, &Register::E),
            0x54 => self.ld_from_r8_to_r8(&Register::D, &Register::H),
            0x55 => self.ld_from_r8_to_r8(&Register::D, &Register::L),
            0x56 => self.ld_from_memory_to_r8(&Register::D, self.get_hl()),

            0x58 => self.ld_from_r8_to_r8(&Register::E, &Register::B),
            0x59 => self.ld_from_r8_to_r8(&Register::E, &Register::C),
            0x5a => self.ld_from_r8_to_r8(&Register::E, &Register::D),
            0x5b => self.ld_from_r8_to_r8(&Register::E, &Register::E),
            0x5c => self.ld_from_r8_to_r8(&Register::E, &Register::H),
            0x5d => self.ld_from_r8_to_r8(&Register::E, &Register::L),
            0x5e => self.ld_from_memory_to_r8(&Register::E, self.get_hl()),

            0x60 => self.ld_from_r8_to_r8(&Register::H, &Register::B),
            0x61 => self.ld_from_r8_to_r8(&Register::H, &Register::C),
            0x62 => self.ld_from_r8_to_r8(&Register::H, &Register::D),
            0x63 => self.ld_from_r8_to_r8(&Register::H, &Register::E),
            0x64 => self.ld_from_r8_to_r8(&Register::H, &Register::H),
            0x65 => self.ld_from_r8_to_r8(&Register::H, &Register::L),
            0x66 => self.ld_from_memory_to_r8(&Register::H, self.get_hl()),

            0x68 => self.ld_from_r8_to_r8(&Register::L, &Register::B),
            0x69 => self.ld_from_r8_to_r8(&Register::L, &Register::C),
            0x6a => self.ld_from_r8_to_r8(&Register::L, &Register::D),
            0x6b => self.ld_from_r8_to_r8(&Register::L, &Register::E),
            0x6c => self.ld_from_r8_to_r8(&Register::L, &Register::H),
            0x6d => self.ld_from_r8_to_r8(&Register::L, &Register::L),
            0x6e => self.ld_from_memory_to_r8(&Register::L, self.get_hl()),

            0x70 => self.ld_from_r8_to_m8(self.get_hl(), &Register::B),
            0x71 => self.ld_from_r8_to_m8(self.get_hl(), &Register::C),
            0x72 => self.ld_from_r8_to_m8(self.get_hl(), &Register::D),
            0x73 => self.ld_from_r8_to_m8(self.get_hl(), &Register::E),
            0x74 => self.ld_from_r8_to_m8(self.get_hl(), &Register::H),
            0x75 => self.ld_from_r8_to_m8(self.get_hl(), &Register::L),
            0x36 => self.ld_from_d16_to_m8(self.get_hl()),

            // 3. LD A,n
            0x1a => self.ld_from_memory_to_r8(&Register::A, self.get_de()),
            0x3e => {
                // LD A, #
                let d8 = self.read_byte(self.pc + 1);
                self.a = d8;
                self.pc = self.pc.wrapping_add(2);

                self.t += 8;
                self.m += 2;
            }

            // 5. LD A,(C)
            0xf2 => {
                let address = 0xff00 + self.c as u16;
                let value = self.read_byte(address);
                self.a = value;
                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 6. LD (C),A
            0xe2 => {
                let address = 0xff00 + self.c as u16;
                self.write_byte(address, self.a);
                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 9. LDD A,(HL)
            0x3a => {
                let hl = self.get_hl();
                let value = self.read_byte(hl);
                self.a = value;

                self.set_hl(hl.wrapping_sub(1));
                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 15. LDI A,(HL)
            0x2a => {
                let hl = self.get_hl();
                let value = self.read_byte(hl);
                self.a = value;

                // println!("LDI A,(HL): A: {:x} hl: {:x}", self.a, hl);

                self.set_hl(hl.wrapping_add(1));
                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 18. LDI (HL),A
            0x22 => {
                let hl = self.get_hl();
                self.write_byte(hl, self.a);
                self.set_hl(hl.wrapping_add(1));

                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 20. LDH A,(n)
            0xf0 => {
                // opcode = read(PC++)
                // if opcode == 0xF0:
                // n = read(PC++)
                // A = read(unsigned_16(lsb=n, msb=0xFF))
                let n = self.read_byte(self.pc + 1);
                let address: u16 = 0xff00 | n as u16;
                let value = self.read_byte(address);
                self.write_r8(&Register::A, value);

                self.pc = self.pc.wrapping_add(2);

                self.t += 12;
                self.m += 3;
            }

            // 3.3.3. 8-Bit ALU
            // 1. ADD A,n
            0x80 => self.add_r8(&Register::B),
            0x81 => self.add_r8(&Register::C),
            0x82 => self.add_r8(&Register::D),
            0x83 => self.add_r8(&Register::E),
            0x84 => self.add_r8(&Register::H),
            0x85 => self.add_r8(&Register::L),
            0x86 => self.add_m8(self.get_hl()),
            0x87 => self.add_r8(&Register::A),
            0xc6 => self.add_d8(),

            // 2. ADC A,n
            0x88 => self.adc_r8(&Register::B),
            0x89 => self.adc_r8(&Register::C),
            0x8a => self.adc_r8(&Register::D),
            0x8b => self.adc_r8(&Register::E),
            0x8c => self.adc_r8(&Register::H),
            0x8d => self.adc_r8(&Register::L),
            0x8e => self.adc_m8(self.get_hl()),
            0x8f => self.adc_r8(&Register::A),
            0xce => self.adc_d8(),

            // 3. SUB n
            0x97 => self.sub_r8(&Register::A),
            0x90 => self.sub_r8(&Register::B),
            0x91 => self.sub_r8(&Register::C),
            0x92 => self.sub_r8(&Register::D),
            0x93 => self.sub_r8(&Register::E),
            0x94 => self.sub_r8(&Register::H),
            0x95 => self.sub_r8(&Register::L),
            0x96 => self.sub_m8(self.get_hl()),
            0xd6 => self.sub_d8(),

            // 4. SBC A,n
            0x9f => self.sbc_r8(&Register::A),
            0x98 => self.sbc_r8(&Register::B),
            0x99 => self.sbc_r8(&Register::C),
            0x9a => self.sbc_r8(&Register::D),
            0x9b => self.sbc_r8(&Register::E),
            0x9c => self.sbc_r8(&Register::H),
            0x9d => self.sbc_r8(&Register::L),
            0x9e => self.sbc_m8(self.get_hl()),
            0xde => self.sbc_d8(),

            // 8. CP n
            0xbf => self.cp_r8(&Register::A),
            0xb8 => self.cp_r8(&Register::B),
            0xb9 => self.cp_r8(&Register::C),
            0xba => self.cp_r8(&Register::D),
            0xbb => self.cp_r8(&Register::E),
            0xbc => self.cp_r8(&Register::H),
            0xbd => self.cp_r8(&Register::L),
            0xbe => {
                let a = self.a;
                let address = self.get_hl();
                let value = self.read_byte(address);

                self.set_z_flag_if(a == value);
                self.set_n_flag();
                self.set_h_flag_if(a & 0x0f < value & 0x0f);
                self.set_c_flag_if(a < value);

                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }
            0xfe => self.cp_d8(),

            // 5. AND n
            0xa7 => self.and_r8(&Register::A),
            0xa0 => self.and_r8(&Register::B),
            0xa1 => self.and_r8(&Register::C),
            0xa2 => self.and_r8(&Register::D),
            0xa3 => self.and_r8(&Register::E),
            0xa4 => self.and_r8(&Register::H),
            0xa5 => self.and_r8(&Register::L),
            0xa6 => self.and_m8(self.get_hl()),
            0xe6 => self.and_d8(),

            // 6. OR n
            0xb7 => self.or_r8(&Register::A),
            0xb0 => self.or_r8(&Register::B),
            0xb1 => self.or_r8(&Register::C),
            0xb2 => self.or_r8(&Register::D),
            0xb3 => self.or_r8(&Register::E),
            0xb4 => self.or_r8(&Register::H),
            0xb5 => self.or_r8(&Register::L),
            0xb6 => self.or_m8(self.get_hl()),
            0xf6 => self.or_d8(),

            // 7. XOR n
            0xaf => self.xor_r8(&Register::A),
            0xa8 => self.xor_r8(&Register::B),
            0xa9 => self.xor_r8(&Register::C),
            0xaa => self.xor_r8(&Register::D),
            0xab => self.xor_r8(&Register::E),
            0xac => self.xor_r8(&Register::H),
            0xad => self.xor_r8(&Register::L),
            0xae => self.xor_m8(self.get_hl()),
            0xee => self.xor_d8(),

            // 9. INC n
            0x3c => self.inc_r8(&Register::A),
            0x04 => self.inc_r8(&Register::B),
            0x0c => self.inc_r8(&Register::C),
            0x14 => self.inc_r8(&Register::D),
            0x1c => self.inc_r8(&Register::E),
            0x24 => self.inc_r8(&Register::H),
            0x2c => self.inc_r8(&Register::L),
            0x34 => self.inc_m8(self.get_hl()),

            // 10. DEC n
            0x3d => self.dec_r8(&Register::A),
            0x05 => self.dec_r8(&Register::B),
            0x0d => self.dec_r8(&Register::C),
            0x15 => self.dec_r8(&Register::D),
            0x1d => self.dec_r8(&Register::E),
            0x25 => self.dec_r8(&Register::H),
            0x2d => self.dec_r8(&Register::L),
            0x35 => self.dec_m8(self.get_hl()),

            // 3.3.4. 16-Bit Arithmetic
            // 1. ADD HL,n
            0x09 => self.add_r16(&Register16::HL, &Register16::BC),
            0x19 => self.add_r16(&Register16::HL, &Register16::DE),
            0x29 => self.add_r16(&Register16::HL, &Register16::HL),
            0x39 => self.add_r16(&Register16::HL, &Register16::SP),

            // 2. ADD SP,n
            0xe8 => self.add_r16_d8(&Register16::SP),

            // 3. INC nn
            0x03 => self.inc_r16(&Register16::BC),
            0x13 => self.inc_r16(&Register16::DE),
            0x23 => self.inc_r16(&Register16::HL),
            0x33 => self.inc_r16(&Register16::SP),

            // 4. DEC nn
            0x0b => self.dec_r16(&Register16::BC),
            0x1b => self.dec_r16(&Register16::DE),
            0x2b => self.dec_r16(&Register16::HL),
            0x3b => self.dec_r16(&Register16::SP),

            // 19. LDH (n),A
            0xe0 => {
                let offset = self.read_byte(self.pc + 1);
                let address = 0xff00 + offset as u16;
                self.write_byte(address, self.a);
                self.pc = self.pc.wrapping_add(2);

                self.t += 12;
                self.m += 3;
            }

            // 16-bit Load instructions
            0x01 => {
                // LD BC, d16
                let d16 = self.read_d16();
                self.b = ((d16 & 0xFF00) >> 8) as u8;
                self.c = (d16 & 0x00FF) as u8;
                self.pc += 3;

                self.t += 12;
                self.m += 3;
            }
            0x11 => {
                // LD DE, d16
                let d16 = self.read_d16();
                self.d = ((d16 & 0xFF00) >> 8) as u8;
                self.e = (d16 & 0x00FF) as u8;
                self.pc += 3;

                self.t += 12;
                self.m += 3;
            }
            0x21 => {
                // LD HL, d16
                let d16 = self.read_d16();
                self.h = ((d16 & 0xFF00) >> 8) as u8;
                self.l = (d16 & 0x00FF) as u8;
                self.pc += 3;

                self.t += 12;
                self.m += 3;
            }
            0x31 => {
                // LD SP, d16
                self.sp = self.read_d16();
                self.pc += 3;

                self.t += 12;
                self.m += 3;
            }

            // 2. LD SP,HL
            0xf9 => {
                self.sp = self.get_hl();

                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }

            // 4. LDHL SP,n
            0xf8 => {
                let sp = self.sp;
                let n = self.read_byte(self.pc + 1) as i8;
                let address = self.sp.wrapping_add(n as u16);
                self.set_hl(address);

                self.reset_z_flag();
                self.reset_n_flag();
                // set if carry from bit-3
                self.set_h_flag_if((sp & 0x0f) + (n as u16 & 0x0f) > 0x0f);
                // set if carry from bit-7
                self.set_c_flag_if((sp & 0xff) + (n as u16 & 0xff) > 0xff);

                self.pc = self.pc.wrapping_add(2);

                self.t += 12;
                self.m += 3;
            }

            // 5. LD (nn),SP
            0x08 => {
                // 2020-01-17
                let address = self.read_byte16(self.pc + 1);
                self.write_byte16(address, self.sp);

                self.pc = self.pc.wrapping_add(3);

                self.t += 20;
                self.m += 5;
            }

            // 6. PUSH nn
            0xf5 => self.push(&Register::A, &Register::F),
            0xc5 => self.push(&Register::B, &Register::C),
            0xd5 => self.push(&Register::D, &Register::E),
            0xe5 => self.push(&Register::H, &Register::L),

            // 7. POP nn
            0xf1 => self.pop(&Register::A, &Register::F),
            0xc1 => self.pop(&Register::B, &Register::C),
            0xd1 => self.pop(&Register::D, &Register::E),
            0xe1 => self.pop(&Register::H, &Register::L),

            // 3.3.8. Jumps
            // 1. JP nn
            0xc3 => self.jp(true),
            //2. JP cc,nn
            0xc2 => self.jp(!self.get_z_flag()),
            0xca => self.jp(self.get_z_flag()),
            0xd2 => self.jp(!self.get_c_flag()),
            0xda => self.jp(self.get_c_flag()),

            // 3. JP (HL)
            0xe9 => {
                self.pc = self.get_hl();

                self.t += 4;
                self.m += 1;
            }
            // 4. JR n
            0x18 => self.jr(true),
            // 5. JR cc,n
            0x20 => self.jr(!self.get_z_flag()),
            0x28 => self.jr(self.get_z_flag()),
            0x30 => self.jr(!self.get_c_flag()),
            0x38 => self.jr(self.get_c_flag()),

            // 12. LDD (HL),A
            0x32 => {
                // LDD (HL), A
                // Put A into memory address HL. Decrement HL.
                let mut hl = self.get_hl();
                self.write_byte(hl, self.a);
                hl = hl.wrapping_sub(1);
                self.set_hl(hl);
                self.pc += 1;

                self.t += 8;
                self.m += 2;
            }

            // 3.3.6. Rotates & Shifts
            // 1. RLCA
            0x07 => {
                let previous = self.a;
                self.a = self.a.rotate_left(1);

                self.reset_z_flag();
                self.reset_n_flag();
                self.reset_h_flag();
                self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);
                // self.set_c_flag_if((previous >> 7 & 1) == 1);

                self.pc = self.pc.wrapping_add(1);

                self.t += 4;
                self.m += 1;

                // println!("RLCA A: {:#X} -> {:#X}", previous, self.a);
            }

            // 2. RLA
            0x17 => {
                let previous = self.a;
                let mut value = self.a << 1;
                if self.get_c_flag() {
                    value = value | 0x01
                } else {
                    value = value | 0x00
                }
                self.a = value;

                self.pc = self.pc.wrapping_add(1);

                self.reset_z_flag();
                self.reset_n_flag();
                self.reset_h_flag();
                // self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);
                self.set_c_flag_if((previous >> 7 & 1) == 1);
            }

            // 3. RRCA
            0x0f => {
                let previous = self.a;
                self.a = self.a.rotate_right(1);

                self.pc = self.pc.wrapping_add(1);

                self.reset_z_flag();
                self.reset_n_flag();
                self.reset_h_flag();
                self.set_c_flag_if((previous & 0x01) > 0);
            }

            // 4. RRA
            0x1f => {
                let previous = self.a;
                let mut value = self.a >> 1;
                if self.get_c_flag() {
                    value = value | 0x80
                } else {
                    value = value | 0x00
                }
                self.a = value;

                self.pc = self.pc.wrapping_add(1);

                // document is wrong
                // self.set_z_flag_if(value == 0);
                self.reset_z_flag();
                self.reset_n_flag();
                self.reset_h_flag();
                self.set_c_flag_if((previous & 0x01) > 0);
            }

            // DAA
            0x27 => {
                // ref. https://ehaskins.com/2018-01-30%20Z80%20DAA/
                let mut a = self.a;

                if !self.get_n_flag() {
                    if self.get_c_flag() || a > 0x99 {
                        a = a.wrapping_add(0x60);
                        self.set_c_flag();
                    }
                    if self.get_h_flag() || a & 0x0f > 0x09 {
                        a = a.wrapping_add(0x06);
                    }
                } else {
                    if self.get_c_flag() {
                        a = a.wrapping_sub(0x60);
                    }
                    if self.get_h_flag() {
                        a = a.wrapping_sub(0x06);
                    }
                }

                self.a = a;

                self.set_z_flag_if(a == 0);
                self.reset_h_flag();

                self.pc = self.pc.wrapping_add(1);
            }

            // prefixed
            0xcb => {
                let prefix = self.read_byte(self.pc + 1);
                match prefix {
                    // RLC
                    0x07 => self.rlc_r8(&Register::A),
                    0x00 => self.rlc_r8(&Register::B),
                    0x01 => self.rlc_r8(&Register::C),
                    0x02 => self.rlc_r8(&Register::D),
                    0x03 => self.rlc_r8(&Register::E),
                    0x04 => self.rlc_r8(&Register::H),
                    0x05 => self.rlc_r8(&Register::L),
                    0x06 => self.rlc_m8(self.get_hl()),

                    // RRC
                    0x0f => self.rrc_r8(&Register::A),
                    0x08 => self.rrc_r8(&Register::B),
                    0x09 => self.rrc_r8(&Register::C),
                    0x0a => self.rrc_r8(&Register::D),
                    0x0b => self.rrc_r8(&Register::E),
                    0x0c => self.rrc_r8(&Register::H),
                    0x0d => self.rrc_r8(&Register::L),
                    0x0e => self.rrc_m8(self.get_hl()),

                    // RL
                    0x17 => self.rl_r8(&Register::A),
                    0x10 => self.rl_r8(&Register::B),
                    0x11 => self.rl_r8(&Register::C),
                    0x12 => self.rl_r8(&Register::D),
                    0x13 => self.rl_r8(&Register::E),
                    0x14 => self.rl_r8(&Register::H),
                    0x15 => self.rl_r8(&Register::L),
                    0x16 => self.rl_m8(self.get_hl()),

                    // 8. RR n
                    0x1f => self.rr_r8(&Register::A),
                    0x18 => self.rr_r8(&Register::B),
                    0x19 => self.rr_r8(&Register::C),
                    0x1a => self.rr_r8(&Register::D),
                    0x1b => self.rr_r8(&Register::E),
                    0x1c => self.rr_r8(&Register::H),
                    0x1d => self.rr_r8(&Register::L),
                    0x1e => self.rr_m8(self.get_hl()),

                    // 9. SLA n
                    0x27 => self.sla_r8(&Register::A),
                    0x20 => self.sla_r8(&Register::B),
                    0x21 => self.sla_r8(&Register::C),
                    0x22 => self.sla_r8(&Register::D),
                    0x23 => self.sla_r8(&Register::E),
                    0x24 => self.sla_r8(&Register::H),
                    0x25 => self.sla_r8(&Register::L),
                    0x26 => self.sla_m8(self.get_hl()),

                    // 10. SRA n
                    0x2f => self.sra_r8(&Register::A),
                    0x28 => self.sra_r8(&Register::B),
                    0x29 => self.sra_r8(&Register::C),
                    0x2a => self.sra_r8(&Register::D),
                    0x2b => self.sra_r8(&Register::E),
                    0x2c => self.sra_r8(&Register::H),
                    0x2d => self.sra_r8(&Register::L),
                    0x2e => self.sra_m8(self.get_hl()),

                    // 1. SWAP n
                    0x37 => self.swap_r8(&Register::A),
                    0x30 => self.swap_r8(&Register::B),
                    0x31 => self.swap_r8(&Register::C),
                    0x32 => self.swap_r8(&Register::D),
                    0x33 => self.swap_r8(&Register::E),
                    0x34 => self.swap_r8(&Register::H),
                    0x35 => self.swap_r8(&Register::L),
                    0x36 => self.swap_m8(self.get_hl()),

                    // 11. SRL n
                    0x3f => self.srl_r8(&Register::A),
                    0x38 => self.srl_r8(&Register::B),
                    0x39 => self.srl_r8(&Register::C),
                    0x3a => self.srl_r8(&Register::D),
                    0x3b => self.srl_r8(&Register::E),
                    0x3c => self.srl_r8(&Register::H),
                    0x3d => self.srl_r8(&Register::L),
                    0x3e => self.srl_m8(self.get_hl()),

                    // BIT
                    0x40 => self.bit_r8(0, &Register::B),
                    0x41 => self.bit_r8(0, &Register::C),
                    0x42 => self.bit_r8(0, &Register::D),
                    0x43 => self.bit_r8(0, &Register::E),
                    0x44 => self.bit_r8(0, &Register::H),
                    0x45 => self.bit_r8(0, &Register::L),
                    0x46 => self.bit_m8(0, self.get_hl()),
                    0x47 => self.bit_r8(0, &Register::A),

                    0x48 => self.bit_r8(1, &Register::B),
                    0x49 => self.bit_r8(1, &Register::C),
                    0x4a => self.bit_r8(1, &Register::D),
                    0x4b => self.bit_r8(1, &Register::E),
                    0x4c => self.bit_r8(1, &Register::H),
                    0x4d => self.bit_r8(1, &Register::L),
                    0x4e => self.bit_m8(1, self.get_hl()),
                    0x4f => self.bit_r8(1, &Register::A),

                    0x50 => self.bit_r8(2, &Register::B),
                    0x51 => self.bit_r8(2, &Register::C),
                    0x52 => self.bit_r8(2, &Register::D),
                    0x53 => self.bit_r8(2, &Register::E),
                    0x54 => self.bit_r8(2, &Register::H),
                    0x55 => self.bit_r8(2, &Register::L),
                    0x56 => {
                        let hl = self.get_hl();
                        self.bit_m8(2, hl);
                    }
                    0x57 => self.bit_r8(2, &Register::A),

                    0x58 => self.bit_r8(3, &Register::B),
                    0x59 => self.bit_r8(3, &Register::C),
                    0x5a => self.bit_r8(3, &Register::D),
                    0x5b => self.bit_r8(3, &Register::E),
                    0x5c => self.bit_r8(3, &Register::H),
                    0x5d => self.bit_r8(3, &Register::L),
                    0x5e => self.bit_m8(3, self.get_hl()),
                    0x5f => self.bit_r8(3, &Register::A),

                    0x60 => self.bit_r8(4, &Register::B),
                    0x61 => self.bit_r8(4, &Register::C),
                    0x62 => self.bit_r8(4, &Register::D),
                    0x63 => self.bit_r8(4, &Register::E),
                    0x64 => self.bit_r8(4, &Register::H),
                    0x65 => self.bit_r8(4, &Register::L),
                    0x66 => {
                        let hl = self.get_hl();
                        self.bit_m8(4, hl);
                    }
                    0x67 => self.bit_r8(4, &Register::A),

                    0x68 => self.bit_r8(5, &Register::B),
                    0x69 => self.bit_r8(5, &Register::C),
                    0x6a => self.bit_r8(5, &Register::D),
                    0x6b => self.bit_r8(5, &Register::E),
                    0x6c => self.bit_r8(5, &Register::H),
                    0x6d => self.bit_r8(5, &Register::L),
                    0x6e => {
                        let hl = self.get_hl();
                        self.bit_m8(5, hl);
                    }
                    0x6f => self.bit_r8(5, &Register::A),

                    0x70 => self.bit_r8(6, &Register::B),
                    0x71 => self.bit_r8(6, &Register::C),
                    0x72 => self.bit_r8(6, &Register::D),
                    0x73 => self.bit_r8(6, &Register::E),
                    0x74 => self.bit_r8(6, &Register::H),
                    0x75 => self.bit_r8(6, &Register::L),
                    0x76 => self.bit_m8(6, self.get_hl()),
                    0x77 => self.bit_r8(6, &Register::A),

                    0x78 => self.bit_r8(7, &Register::B),
                    0x79 => self.bit_r8(7, &Register::C),
                    0x7a => self.bit_r8(7, &Register::D),
                    0x7b => self.bit_r8(7, &Register::E),
                    0x7c => self.bit_r8(7, &Register::H),
                    0x7d => self.bit_r8(7, &Register::L),
                    0x7e => {
                        let hl = self.get_hl();
                        self.bit_m8(7, hl);
                    }
                    0x7f => self.bit_r8(7, &Register::A),

                    // 3. RES b,r
                    0x80 => self.res_r8(0, &Register::B),
                    0x81 => self.res_r8(0, &Register::C),
                    0x82 => self.res_r8(0, &Register::D),
                    0x83 => self.res_r8(0, &Register::E),
                    0x84 => self.res_r8(0, &Register::H),
                    0x85 => self.res_r8(0, &Register::L),
                    0x86 => self.res_m8(0, self.get_hl()),
                    0x87 => self.res_r8(0, &Register::A),

                    0x88 => self.res_r8(1, &Register::B),
                    0x89 => self.res_r8(1, &Register::C),
                    0x8a => self.res_r8(1, &Register::D),
                    0x8b => self.res_r8(1, &Register::E),
                    0x8c => self.res_r8(1, &Register::H),
                    0x8d => self.res_r8(1, &Register::L),
                    0x8e => self.res_m8(1, self.get_hl()),
                    0x8f => self.res_r8(1, &Register::A),

                    0x90 => self.res_r8(2, &Register::B),
                    0x91 => self.res_r8(2, &Register::C),
                    0x92 => self.res_r8(2, &Register::D),
                    0x93 => self.res_r8(2, &Register::E),
                    0x94 => self.res_r8(2, &Register::H),
                    0x95 => self.res_r8(2, &Register::L),
                    0x96 => self.res_m8(2, self.get_hl()),
                    0x97 => self.res_r8(2, &Register::A),

                    0x98 => self.res_r8(3, &Register::B),
                    0x99 => self.res_r8(3, &Register::C),
                    0x9a => self.res_r8(3, &Register::D),
                    0x9b => self.res_r8(3, &Register::E),
                    0x9c => self.res_r8(3, &Register::H),
                    0x9d => self.res_r8(3, &Register::L),
                    0x9e => self.res_m8(3, self.get_hl()),
                    0x9f => self.res_r8(3, &Register::A),

                    0xa0 => self.res_r8(4, &Register::B),
                    0xa1 => self.res_r8(4, &Register::C),
                    0xa2 => self.res_r8(4, &Register::D),
                    0xa3 => self.res_r8(4, &Register::E),
                    0xa4 => self.res_r8(4, &Register::H),
                    0xa5 => self.res_r8(4, &Register::L),
                    0xa6 => self.res_m8(4, self.get_hl()),
                    0xa7 => self.res_r8(4, &Register::A),

                    0xa8 => self.res_r8(5, &Register::B),
                    0xa9 => self.res_r8(5, &Register::C),
                    0xaa => self.res_r8(5, &Register::D),
                    0xab => self.res_r8(5, &Register::E),
                    0xac => self.res_r8(5, &Register::H),
                    0xad => self.res_r8(5, &Register::L),
                    0xae => self.res_m8(5, self.get_hl()),
                    0xaf => self.res_r8(5, &Register::A),

                    0xb0 => self.res_r8(6, &Register::B),
                    0xb1 => self.res_r8(6, &Register::C),
                    0xb2 => self.res_r8(6, &Register::D),
                    0xb3 => self.res_r8(6, &Register::E),
                    0xb4 => self.res_r8(6, &Register::H),
                    0xb5 => self.res_r8(6, &Register::L),
                    0xb6 => self.res_m8(6, self.get_hl()),
                    0xb7 => self.res_r8(6, &Register::A),

                    0xb8 => self.res_r8(7, &Register::B),
                    0xb9 => self.res_r8(7, &Register::C),
                    0xba => self.res_r8(7, &Register::D),
                    0xbb => self.res_r8(7, &Register::E),
                    0xbc => self.res_r8(7, &Register::H),
                    0xbd => self.res_r8(7, &Register::L),
                    0xbe => self.res_m8(7, self.get_hl()),
                    0xbf => self.res_r8(7, &Register::A),

                    // 2. SET b,r
                    0xc0 => self.set_r8(0, &Register::B),
                    0xc1 => self.set_r8(0, &Register::C),
                    0xc2 => self.set_r8(0, &Register::D),
                    0xc3 => self.set_r8(0, &Register::E),
                    0xc4 => self.set_r8(0, &Register::H),
                    0xc5 => self.set_r8(0, &Register::L),
                    0xc6 => self.set_m8(0, self.get_hl()),
                    0xc7 => self.set_r8(0, &Register::A),

                    0xc8 => self.set_r8(1, &Register::B),
                    0xc9 => self.set_r8(1, &Register::C),
                    0xca => self.set_r8(1, &Register::D),
                    0xcb => self.set_r8(1, &Register::E),
                    0xcc => self.set_r8(1, &Register::H),
                    0xcd => self.set_r8(1, &Register::L),
                    0xce => self.set_m8(1, self.get_hl()),
                    0xcf => self.set_r8(1, &Register::A),

                    0xd0 => self.set_r8(2, &Register::B),
                    0xd1 => self.set_r8(2, &Register::C),
                    0xd2 => self.set_r8(2, &Register::D),
                    0xd3 => self.set_r8(2, &Register::E),
                    0xd4 => self.set_r8(2, &Register::H),
                    0xd5 => self.set_r8(2, &Register::L),
                    0xd6 => self.set_m8(2, self.get_hl()),
                    0xd7 => self.set_r8(2, &Register::A),

                    0xd8 => self.set_r8(3, &Register::B),
                    0xd9 => self.set_r8(3, &Register::C),
                    0xda => self.set_r8(3, &Register::D),
                    0xdb => self.set_r8(3, &Register::E),
                    0xdc => self.set_r8(3, &Register::H),
                    0xdd => self.set_r8(3, &Register::L),
                    0xde => self.set_m8(3, self.get_hl()),
                    0xdf => self.set_r8(3, &Register::A),

                    0xe0 => self.set_r8(4, &Register::B),
                    0xe1 => self.set_r8(4, &Register::C),
                    0xe2 => self.set_r8(4, &Register::D),
                    0xe3 => self.set_r8(4, &Register::E),
                    0xe4 => self.set_r8(4, &Register::H),
                    0xe5 => self.set_r8(4, &Register::L),
                    0xe6 => self.set_m8(4, self.get_hl()),
                    0xe7 => self.set_r8(4, &Register::A),

                    0xe8 => self.set_r8(5, &Register::B),
                    0xe9 => self.set_r8(5, &Register::C),
                    0xea => self.set_r8(5, &Register::D),
                    0xeb => self.set_r8(5, &Register::E),
                    0xec => self.set_r8(5, &Register::H),
                    0xed => self.set_r8(5, &Register::L),
                    0xee => self.set_m8(5, self.get_hl()),
                    0xef => self.set_r8(5, &Register::A),

                    0xf0 => self.set_r8(6, &Register::B),
                    0xf1 => self.set_r8(6, &Register::C),
                    0xf2 => self.set_r8(6, &Register::D),
                    0xf3 => self.set_r8(6, &Register::E),
                    0xf4 => self.set_r8(6, &Register::H),
                    0xf5 => self.set_r8(6, &Register::L),
                    0xf6 => self.set_m8(6, self.get_hl()),
                    0xf7 => self.set_r8(6, &Register::A),

                    0xf8 => self.set_r8(7, &Register::B),
                    0xf9 => self.set_r8(7, &Register::C),
                    0xfa => self.set_r8(7, &Register::D),
                    0xfb => self.set_r8(7, &Register::E),
                    0xfc => self.set_r8(7, &Register::H),
                    0xfd => self.set_r8(7, &Register::L),
                    0xfe => self.set_m8(7, self.get_hl()),
                    0xff => self.set_r8(7, &Register::A),
                }

                self.pc += 2;
            }

            // 1. CALL nn
            0xcd => self.call(),

            // 2. CALL cc,nn
            0xc4 => self.call_if(!self.get_z_flag()),
            0xcc => self.call_if(self.get_z_flag()),
            0xd4 => self.call_if(!self.get_c_flag()),
            0xdc => self.call_if(self.get_c_flag()),

            // 1. RST n
            0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
                self.sp = self.sp.wrapping_sub(2);
                self.write_byte16(self.sp, self.pc + 1);

                self.pc = 0x0000 + instruction as u16 - 0x00c7;
                // self.pc = self.pc.wrapping_add(1);

                self.t += 16;
                self.m += 4;
            }

            // 1. RET
            0xc9 => self.ret(),

            // 2. RET cc
            0xc0 => {
                if !self.get_z_flag() {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xc8 => {
                if self.get_z_flag() {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xd0 => {
                if !self.get_c_flag() {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }
            0xd8 => {
                if self.get_c_flag() {
                    self.ret();
                } else {
                    self.pc = self.pc.wrapping_add(1);
                }
            }

            // 3. RETI
            0xd9 => {
                self.ret();
                self.ime = true;
            }

            // CPL
            0x2f => {
                // Complement A register. (Flip all bits.)
                self.a = !self.a;

                self.set_n_flag();
                self.set_h_flag();

                self.pc = self.pc.wrapping_add(1);

                self.t += 4;
                self.m += 1;
            }

            // 4. CCF
            0x3f => {
                self.reset_n_flag();
                self.reset_h_flag();
                if self.get_c_flag() {
                    self.reset_c_flag();
                } else {
                    self.set_c_flag();
                }

                self.pc = self.pc.wrapping_add(1);

                self.t += 4;
                self.m += 1;
            }

            // 5. SCF
            0x37 => {
                self.reset_n_flag();
                self.reset_h_flag();
                self.set_c_flag();

                self.pc = self.pc.wrapping_add(1);

                self.t += 4;
                self.m += 1;
            }

            // 7. HALT
            0x76 => {
                self.halt = true;

                self.pc = self.pc.wrapping_add(1)
                // halt until interrupt occurs
            }

            // 8. STOP
            0x10 => {
                self.t += 4;
                self.m += 1;

                self.pc = self.pc.wrapping_add(1);
            }

            // EI
            0xfb => {
                self.ime = true;
                self.pc += 1;

                self.t += 4;
                self.m += 1;
            }

            // DI
            0xf3 => {
                // Disables interrupt handling by setting IME=0
                self.ime = false;
                self.pc += 1;

                self.t += 4;
                self.m += 1;
            }

            0xd3 | 0xdb | 0xdd | 0xe3..=0xe4 | 0xeb..=0xed | 0xf4 | 0xfc..=0xfd => panic!(
                "unrecognized instructions {:#x} on pc {:#x}",
                instruction, self.pc
            ),
        }
    }

    fn read_byte(&mut self, address: u16) -> u8 {
        return self.mmu.read_byte(address);
    }

    fn read_byte16(&mut self, address: u16) -> u16 {
        let low = self.mmu.read_byte(address);
        let high = self.mmu.read_byte(address.wrapping_add(1));

        return ((high as u16) << 8) | low as u16;
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        // match address {
        //     0xc5fb | 0xd600 => {
        //         println!(
        //             "[{:#X}] debug write pc: {:#X} write {:#X} HL: {:#X} DE: {:#X}",
        //             address,
        //             self.pc,
        //             value,
        //             self.get_hl(),
        //             self.get_de()
        //         )
        //     }
        //     _ => {}
        // }
        self.mmu.write_byte(address, value);
    }

    fn write_byte16(&mut self, address: u16, value: u16) {
        // little endian
        self.mmu.write_byte(address, (value & 0x00ff) as u8);
        let next = address.wrapping_add(1);
        self.mmu.write_byte(next, (value >> 8) as u8);
    }

    fn read_d16(&self) -> u16 {
        // The 16-bit immediates are in little endian

        // println!(
        //     "{:#x} {:#x}",
        //     self.mmu.read_byte(self.pc + 1),
        //     self.mmu.read_byte(self.pc + 2),
        // );

        let low = self.mmu.read_byte(self.pc + 1) as u16;
        let high = self.mmu.read_byte(self.pc + 2) as u16;
        let d16: u16 = (high << 8) | low;

        return d16;
    }

    fn read_r8(&mut self, register: &Register) -> u8 {
        match register {
            Register::A => {
                return self.a;
            }
            Register::B => {
                return self.b;
            }
            Register::C => {
                return self.c;
            }
            Register::D => {
                return self.d;
            }
            Register::E => {
                return self.e;
            }
            Register::F => {
                return self.f;
            }
            Register::H => {
                return self.h;
            }
            Register::L => {
                return self.l;
            }
        }
    }

    fn read_r16(&mut self, pair: &Register16) -> u16 {
        match pair {
            Register16::AF => {
                return self.get_af();
            }
            Register16::BC => {
                return self.get_bc();
            }
            Register16::DE => {
                return self.get_de();
            }
            Register16::HL => {
                return self.get_hl();
            }
            Register16::SP => {
                return self.sp;
            }
        }
    }

    fn write_r8(&mut self, register: &Register, value: u8) {
        match register {
            Register::A => {
                self.a = value;
            }
            Register::B => {
                self.b = value;
            }
            Register::C => {
                self.c = value;
            }
            Register::D => {
                self.d = value;
            }
            Register::E => {
                self.e = value;
            }
            Register::F => {
                self.f = value;
            }
            Register::H => {
                self.h = value;
            }
            Register::L => {
                self.l = value;
            }
        }
    }

    fn write_r16(&mut self, pair: &Register16, value: u16) {
        match pair {
            Register16::AF => {
                self.set_hl(value);
            }
            Register16::BC => {
                self.set_bc(value);
            }
            Register16::DE => {
                self.set_de(value);
            }
            Register16::HL => {
                self.set_hl(value);
            }
            Register16::SP => {
                self.sp = value;
            }
        }
    }

    fn and_r8(&mut self, r: &Register) {
        let value = self.read_r8(r);
        self.a &= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.set_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn and_m8(&mut self, address: u16) {
        let value = self.read_byte(address);
        self.a &= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.set_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn and_d8(&mut self) {
        let value = self.read_byte(self.pc + 1);
        self.a &= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.set_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(2);

        self.t += 4;
        self.m += 1;
    }

    fn or_r8(&mut self, r: &Register) {
        let value = self.read_r8(r);
        self.a |= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn or_m8(&mut self, address: u16) {
        let value = self.read_byte(address);
        self.a |= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn or_d8(&mut self) {
        let value = self.read_byte(self.pc + 1);
        self.a |= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(2);

        self.t += 4;
        self.m += 1;
    }

    fn xor_r8(&mut self, r: &Register) {
        let value = self.read_r8(r);
        self.a ^= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn xor_m8(&mut self, address: u16) {
        let value = self.read_byte(address);
        self.a ^= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn xor_d8(&mut self) {
        let value = self.read_byte(self.pc + 1);
        self.a ^= value;

        self.set_z_flag_if(self.a == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        // println!("XOR a {:#X}", value);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn add_r8(&mut self, r: &Register) {
        let previous = self.a;
        let n = self.read_r8(r);

        let (value, carry) = previous.overflowing_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) + (n & 0x0f) > 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn add_m8(&mut self, address: u16) {
        let previous = self.a;
        let n = self.read_byte(address);

        let (value, carry) = previous.overflowing_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) + (n & 0x0f) > 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn add_d8(&mut self) {
        let previous = self.a;
        let n = self.read_byte(self.pc + 1);

        let (value, carry) = previous.overflowing_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) + (n & 0x0f) > 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn adc_r8(&mut self, r: &Register) {
        let previous = self.a;
        let n = self.read_r8(r);
        let c = match self.get_c_flag() {
            true => 1,
            false => 0,
        };

        let value = previous.wrapping_add(n).wrapping_add(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0xf) + (n & 0xf) + c > 0xf);
        let carry = (previous as u16) + (n as u16) + (c as u16) > 0xff;
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn adc_m8(&mut self, address: u16) {
        let previous = self.a;
        let n = self.read_byte(address);
        let c = match self.get_c_flag() {
            true => 1,
            false => 0,
        };

        let value = previous.wrapping_add(n).wrapping_add(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) + (n & 0x0f) + c > 0x0f);
        let carry = (previous as u16) + (n as u16) + (c as u16) > 0xff;
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn adc_d8(&mut self) {
        let previous = self.a;
        let n = self.read_byte(self.pc + 1);
        let c = match self.get_c_flag() {
            true => 1,
            false => 0,
        };

        let value = previous.wrapping_add(n).wrapping_add(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) + (n & 0x0f) + c > 0x0f);
        let carry = (previous as u16) + (n as u16) + (c as u16) > 0xff;
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn sub_r8(&mut self, r: &Register) {
        let a = self.a;
        let n = self.read_r8(r);

        let (value, carry) = a.overflowing_sub(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(a & 0x0f < n & 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn sub_m8(&mut self, address: u16) {
        let previous = self.a;
        let n = self.read_byte(address);

        let (value, carry) = previous.overflowing_sub(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(previous & 0x0f < n & 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn sub_d8(&mut self) {
        let previous = self.a;
        let n = self.read_byte(self.pc + 1);

        let (value, carry) = previous.overflowing_sub(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(previous & 0x0f < n & 0x0f);
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn sbc_r8(&mut self, r: &Register) {
        let a = self.a;
        let n = self.read_r8(r);
        let c = if self.get_c_flag() { 1 } else { 0 };

        let value = a.wrapping_sub(n).wrapping_sub(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        // wrong -> self.set_h_flag_if(previous & 0x0f < (n & 0x0f + c));
        self.set_h_flag_if(a & 0x0f < (n & 0x0f) + c);
        // n + c > u8Max
        self.set_c_flag_if((a as u16) < (n as u16) + (c as u16));

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn sbc_m8(&mut self, address: u16) {
        let previous = self.a;
        let n = self.read_byte(address);
        let c = if self.get_c_flag() { 1 } else { 0 };

        let value = previous.wrapping_sub(n).wrapping_sub(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(previous & 0x0f < (n & 0x0f) + c);
        self.set_c_flag_if((previous as u16) < (n as u16) + (c as u16));

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn sbc_d8(&mut self) {
        let previous = self.a;
        let n = self.read_byte(self.pc + 1);
        let c = if self.get_c_flag() { 1 } else { 0 };

        let value = previous.wrapping_sub(n).wrapping_sub(c);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(previous & 0x0f < (n & 0x0f) + c);
        self.set_c_flag_if((previous as u16) < (n as u16) + (c as u16));

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn cp_r8(&mut self, r: &Register) {
        let a = self.a;
        let value = self.read_r8(r);

        self.set_z_flag_if(a == value);
        self.set_n_flag();
        self.set_h_flag_if((a & 0x0f) < (value & 0x0f));
        self.set_c_flag_if(a < value);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn cp_d8(&mut self) {
        let a = self.a;
        let value = self.read_byte(self.pc + 1);

        self.set_z_flag_if(a == value);
        self.set_n_flag();
        self.set_h_flag_if(a & 0x0f < value & 0x0f);
        self.set_c_flag_if(a < value);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn ld_from_r8_to_r8(&mut self, r1: &Register, r2: &Register) {
        let value = self.read_r8(r2);
        self.write_r8(r1, value);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn ld_from_r8_to_m8(&mut self, address: u16, r: &Register) {
        let value = self.read_r8(r);
        self.write_byte(address, value);

        // println!("lD address({:x}) = {}", address, value);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn ld_from_d16_to_m8(&mut self, address: u16) {
        let value = self.read_byte16(self.pc + 1);
        self.write_byte16(address, value);

        // println!("lD address({:x}) = {}", address, value);

        self.pc = self.pc.wrapping_add(2);

        self.t += 12;
        self.m += 3;
    }

    fn ld_from_r8_to_d16(&mut self, address: u16, r: &Register) {
        let value = self.read_r8(r);
        self.write_byte(address, value);

        self.pc = self.pc.wrapping_add(3);

        self.t += 16;
        self.m += 4;
    }

    fn ld_from_memory_to_r8(&mut self, r1: &Register, address: u16) {
        let value = self.read_byte(address);
        self.write_r8(r1, value);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn ld_from_d16_to_r8(&mut self, r1: &Register) {
        let value = self.read_byte(self.read_d16());
        self.write_r8(r1, value);

        self.pc = self.pc.wrapping_add(3);

        self.t += 8;
        self.m += 2;
    }

    fn res_r8(&mut self, b: u8, r: &Register) {
        // Reset bit b in register r.
        let value = self.read_r8(r);
        self.write_r8(r, value & !(1 << b));

        self.t += 8;
        self.m += 2;
    }

    fn res_m8(&mut self, b: u8, address: u16) {
        let value = self.read_byte(address);
        self.write_byte(address, value & !(1 << b));

        self.t += 16;
        self.m += 4;
    }

    fn set_r8(&mut self, b: u8, r: &Register) {
        // Reset bit b in register r.
        let value = self.read_r8(r);
        self.write_r8(r, value | (1 << b));

        self.t += 8;
        self.m += 2;
    }

    fn set_m8(&mut self, b: u8, address: u16) {
        let value = self.read_byte(address);
        self.write_byte(address, value | (1 << b));

        self.t += 16;
        self.m += 4;
    }

    fn rl_r8(&mut self, r: &Register) {
        // rorate left through Carry flag
        // ref. https://ja.wikipedia.org/wiki/%E3%83%93%E3%83%83%E3%83%88%E6%BC%94%E7%AE%97#/media/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB:Rotate_left_through_carry.svg
        let previous = self.read_r8(r);
        let mut value = self.read_r8(r) << 1;
        if self.get_c_flag() {
            value = value | 0x01;
        } else {
            value = value | 0x00;
        }
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn rl_m8(&mut self, address: u16) {
        // rorate left through Carry flag
        // ref. https://ja.wikipedia.org/wiki/%E3%83%93%E3%83%83%E3%83%88%E6%BC%94%E7%AE%97#/media/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB:Rotate_left_through_carry.svg
        let previous = self.read_byte(address);
        let mut value = self.read_byte(address) << 1;
        if self.get_c_flag() {
            value = value | 0x01;
        } else {
            value = value | 0x00;
        }
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn rlc_r8(&mut self, r: &Register) {
        // Rotate n left. Old bit 7 to Carry flag.
        let previous = self.read_r8(r);
        let value = previous.rotate_left(1);
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn rlc_m8(&mut self, address: u16) {
        // Rotate n left. Old bit 7 to Carry flag.
        let previous = self.read_byte(address);
        let value = previous.rotate_left(1);
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn rrc_r8(&mut self, r: &Register) {
        // Rotate n right. Old bit 0 to Carry flag.
        let previous = self.read_r8(r);
        let value = self.read_r8(r).rotate_right(1);
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn rrc_m8(&mut self, address: u16) {
        // Rotate n right. Old bit 0 to Carry flag.
        let previous = self.read_byte(address);
        let value = self.read_byte(address).rotate_right(1);
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn rr_r8(&mut self, r: &Register) {
        let previous = self.read_r8(r);
        let mut value = self.read_r8(r) >> 1;
        if self.get_c_flag() {
            value = value | 0x80;
        } else {
            value = value | 0x00;
        }
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn rr_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let mut value = self.read_byte(address) >> 1;
        if self.get_c_flag() {
            value = value | 0x80;
        } else {
            value = value | 0x00;
        }
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn sla_r8(&mut self, r: &Register) {
        let previous = self.read_r8(r);
        let value = self.read_r8(r) << 1;
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn sla_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let value = self.read_byte(address) << 1;
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn sra_r8(&mut self, r: &Register) {
        // Shift n right into Carry. MSB doesn't change.
        let previous = self.read_r8(r);
        let value = self.read_r8(r) >> 1;
        self.write_r8(r, value | (previous & 0x80));

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn sra_m8(&mut self, address: u16) {
        // Shift n right into Carry. MSB doesn't change.
        let previous = self.read_byte(address);
        let value = self.read_byte(address) >> 1;
        self.write_byte(address, value | (previous & 0x80));

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn srl_r8(&mut self, r: &Register) {
        // Shift n right into Carry. MSB set to 0.
        let previous = self.read_r8(r);
        let value = self.read_r8(r) >> 1;
        self.write_r8(r, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn srl_m8(&mut self, address: u16) {
        // Shift n right into Carry. MSB set to 0.
        let previous = self.read_byte(address);
        let value = self.read_byte(address) >> 1;
        self.write_byte(address, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0x01) > 0);

        self.t += 8;
        self.m += 2;
    }

    fn swap_r8(&mut self, r: &Register) {
        let previous = self.read_r8(r);
        let value = (previous & 0x0f) << 4 | (previous & 0xf0) >> 4;

        self.write_r8(r, value);
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.t += 8;
        self.m += 2;
    }

    fn swap_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let value = (previous & 0x0f) << 4 | (previous & 0xf0) >> 4;

        self.write_byte(address, value);
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.reset_c_flag();

        self.t += 8;
        self.m += 2;
    }

    fn bit_r8(&mut self, n: u8, r: &Register) {
        let value = self.read_r8(r);
        let mask = 0x0001 << n;
        let bit_test = value & mask;

        self.set_z_flag_if(bit_test == 0);
        self.reset_n_flag();
        self.set_h_flag();

        self.t += 8;
        self.m += 2;
    }

    fn bit_m8(&mut self, n: u8, address: u16) {
        let value = self.read_byte(address);
        let mask = 0x0001 << n;
        let bit_test = value & mask;

        self.set_z_flag_if(bit_test == 0);
        self.reset_n_flag();
        self.set_h_flag();

        self.t += 8;
        self.m += 2;
    }

    fn inc_r8(&mut self, register: &Register) {
        let previous = self.read_r8(register);
        let new_value = self.read_r8(register).wrapping_add(1);
        self.write_r8(register, new_value);

        //update the flags
        self.set_z_flag_if(new_value == 0);
        self.reset_n_flag();
        // Set if carry from bit 3
        self.set_h_flag_if(previous & 0x0f == 0x0f);

        self.pc = self.pc.wrapping_add(1);
    }

    fn inc_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let new_value = self.read_byte(address).wrapping_add(1);
        self.write_byte(address, new_value);

        //update the flags
        self.set_z_flag_if(new_value == 0);
        self.reset_n_flag();
        // Set if carry from bit 3
        self.set_h_flag_if(previous & 0x0f == 0x0f);

        self.pc = self.pc.wrapping_add(1);

        self.t += 12;
        self.m += 3;
    }

    fn add_r16(&mut self, op1: &Register16, op2: &Register16) {
        let value = self.read_r16(op1);
        let n = self.read_r16(op2);
        self.write_r16(op1, value.wrapping_add(n));

        self.reset_n_flag();
        let half_carry = (value & 0x0fff) + (n & 0x0fff) > 0x0fff;
        self.set_h_flag_if(half_carry);
        let carry = (value as u32) + (n as u32) > 0xffff;
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn add_r16_d8(&mut self, r: &Register16) {
        // maybe wrong
        let value = self.read_r16(r);
        let n = self.read_byte(self.pc + 1) as i8;
        let offset = n as u16;
        self.write_r16(r, value.wrapping_add(offset));

        self.reset_z_flag();
        self.reset_n_flag();
        let half_carry = (value & 0x0f) + (offset & 0x0f) > 0x0f;
        self.set_h_flag_if(half_carry);
        let carry = (value & 0xff) + (offset & 0xff) > 0xff;
        self.set_c_flag_if(carry);

        self.pc = self.pc.wrapping_add(2);

        self.t += 16;
        self.m += 4;
    }

    fn inc_r16(&mut self, r16: &Register16) {
        let value = self.read_r16(r16);
        self.write_r16(r16, value.wrapping_add(1));

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn dec_r16(&mut self, r16: &Register16) {
        let value = self.read_r16(r16);
        self.write_r16(r16, value.wrapping_sub(1));

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn dec_r8(&mut self, register: &Register) {
        let previous = self.read_r8(register);
        let new_value = self.read_r8(register).wrapping_sub(1);
        self.write_r8(register, new_value);

        //update the flags
        self.set_z_flag_if(new_value == 0);
        self.set_n_flag();
        // Set if carry from bit 4.
        self.set_h_flag_if(previous & 0x0f == 0);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn dec_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let new_value = self.read_byte(address).wrapping_sub(1);
        self.write_byte(address, new_value);

        //update the flags
        self.set_z_flag_if(new_value == 0);
        self.set_n_flag();
        // Set if carry from bit 4.
        self.set_h_flag_if(previous & 0x0f == 0);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn push(&mut self, msb: &Register, lsb: &Register) {
        self.sp = self.sp.wrapping_sub(1);
        let high = self.read_r8(msb);
        self.write_byte(self.sp, high);

        self.sp = self.sp.wrapping_sub(1);
        let low = self.read_r8(lsb);
        self.write_byte(self.sp, low);

        // println!(
        //     "PUSH {:?}{:?} -> {:#X} {:X}",
        //     msb,
        //     lsb,
        //     self.read_r8(msb),
        //     self.read_r8(lsb)
        // );

        self.pc = self.pc.wrapping_add(1);

        self.t += 16;
        self.m += 4;
    }

    fn pop(&mut self, msb: &Register, lsb: &Register) {
        let mut low = self.read_byte(self.sp);
        match lsb {
            &Register::F => {
                // https://forums.nesdev.com/viewtopic.php?f=20&t=12815
                // the lower 4 bits of the F register is always zero.
                low &= 0xf0;
            }
            _ => {}
        }
        self.write_r8(lsb, low);
        self.sp = self.sp.wrapping_add(1);

        let high = self.read_byte(self.sp);
        self.write_r8(msb, high);
        self.sp = self.sp.wrapping_add(1);

        // println!(
        //     "POP {:?}{:?} <- {:#X} {:X}",
        //     msb,
        //     lsb,
        //     self.read_r8(msb),
        //     self.read_r8(lsb)
        // );

        self.pc = self.pc.wrapping_add(1);

        self.t += 12;
        self.m += 3;
    }

    fn jp(&mut self, condition: bool) {
        let address = self.read_d16();
        if condition {
            self.pc = address;
        } else {
            self.pc = self.pc.wrapping_add(3);
        }

        self.t += 12;
        self.m += 3;
    }

    fn jr(&mut self, condition: bool) {
        // JR cc, n
        // n = one byte ##signed## immediate value

        // opcode = read(PC++)
        // if opcode in [0x20, 0x30, 0x28, 0x38]:
        // e = signed_8(read(PC++))
        // if F.check_condition(cc):
        // PC = PC + e
        let n = self.read_byte(self.pc + 1) as i8;

        self.pc += 2;
        if condition {
            self.pc = self.pc.wrapping_add(n as u16);

            self.t += 12;
            self.m += 3;
        } else {
            self.t += 8;
            self.m += 2;
        }

        // println!("JR to #{:X} offset: #{:?}", self.pc, n);
    }

    fn ret(&mut self) {
        let low = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);
        let high = self.read_byte(self.sp);
        self.sp = self.sp.wrapping_add(1);

        self.pc = (high as u16) << 8 | low as u16;

        self.t += 16;
        self.m += 4;
    }

    fn call(&mut self) {
        // opcode = read(PC++)
        // if opcode == 0xCD:
        // nn = unsigned_16(lsb=read(PC++), msb=read(PC++))
        // write(--SP, msb(PC))
        // write(--SP, lsb(PC))
        // PC = nn
        let next = self.read_d16();
        self.pc = self.pc.wrapping_add(3);

        self.sp = self.sp.wrapping_sub(2);
        self.write_byte16(self.sp, self.pc);

        self.pc = next;

        self.t += 24;
        self.m += 6;
    }

    fn call_if(&mut self, condition: bool) {
        if condition {
            self.call();
        } else {
            self.pc = self.pc.wrapping_add(3);

            self.t += 12;
            self.m += 3;
        }
    }

    fn get_flag(&self, bit_mask: u8) -> bool {
        (self.f & bit_mask) != 0
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
        self.f = self.f | 0b1000_0000;
    }

    fn reset_z_flag(&mut self) {
        self.f = self.f & 0b0111_1111;
    }
    fn set_n_flag(&mut self) {
        self.f = self.f | 0b0100_0000;
    }
    fn reset_n_flag(&mut self) {
        self.f = self.f & 0b1011_1111;
    }
    fn set_h_flag(&mut self) {
        self.f = self.f | 0b0010_0000;
    }
    fn reset_h_flag(&mut self) {
        self.f = self.f & 0b1101_1111;
    }
    fn set_c_flag(&mut self) {
        self.f = self.f | 0b0001_0000;
    }
    fn reset_c_flag(&mut self) {
        self.f = self.f & 0b1110_1111;
    }

    fn get_af(&self) -> u16 {
        return ((self.a as u16) << 8) | self.f as u16;
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }

    fn get_bc(&self) -> u16 {
        return ((self.b as u16) << 8) | self.c as u16;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    fn get_de(&self) -> u16 {
        return ((self.d as u16) << 8) | self.e as u16;
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    fn get_hl(&self) -> u16 {
        return ((self.h as u16) << 8) | self.l as u16;
    }

    fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xFF00) >> 8) as u8;
        self.l = (value & 0x00FF) as u8;
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
