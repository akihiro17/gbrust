use std::fmt;

use crate::mmu::MMU;

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
    ime: bool,
    debug_counter: u32,
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
            debug_counter: 0,
        };
    }

    pub fn step(&mut self) -> usize {
        self.t = 0;
        self.fetch_and_execute();
        self.mmu.step(self.t);

        // self.mmu.ppu.debug = true;
        // check for interrupts
        if self.ime {
            // V-Blank interrupt
            let irq = (self.mmu.interrupt_flag & 0x01) == 0x01;
            let ie = (self.mmu.interrupt_enable & 0x01) == 0x01;

            if irq && ie {
                // call isr(Interrupt Serivce Routine)
                // disable further interrupts
                self.ime = false;
                // reset V-Blank interrupt reqeust flag
                self.mmu.interrupt_flag &= 0xfe;

                self.sp = self.sp.wrapping_sub(2);
                self.write_byte16(self.sp, self.pc);
                // V-Blank handler
                self.pc = 0x0040;

                self.t += 12;
                self.m += 3;
            }
        }

        return self.t;
    }

    pub fn fetch_and_execute(&mut self) {
        // println!("byte on {:#X}: {:#X}", self.pc, self.mmu.read_byte(self.pc));

        let instruction = self.mmu.read_byte(self.pc);
        match instruction {
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
            0x7f => {
                // LD A, A
                self.ld_from_r8_to_r8(&Register::A, &Register::A);
            }
            0x47 => {
                // LD B, A
                self.ld_from_r8_to_r8(&Register::B, &Register::A);
            }
            0x4f => {
                // LD C, A
                self.ld_from_r8_to_r8(&Register::C, &Register::A);
            }
            0x57 => {
                // LD D, A
                self.ld_from_r8_to_r8(&Register::D, &Register::A);
            }
            0x5f => {
                // LD E, A
                self.ld_from_r8_to_r8(&Register::E, &Register::A);
            }
            0x67 => {
                // LD H, A
                self.ld_from_r8_to_r8(&Register::H, &Register::A);
            }
            0x6f => {
                // LD L, A
                self.ld_from_r8_to_r8(&Register::L, &Register::A);
            }
            0x02 => {
                // LD (BC), A
                let bc = self.get_bc();
                self.ld_from_r8_to_m8(bc, &Register::A);
            }
            0x12 => {
                // LD (DE), A
                let de = self.get_de();
                self.ld_from_r8_to_m8(de, &Register::A);
            }
            0x77 => {
                let hl = self.get_hl();
                self.ld_from_r8_to_m8(hl, &Register::A);
            }
            0xea => {
                // LD (nn), A
                let address = self.read_byte16(self.pc + 1);
                self.ld_from_r8_to_d16(address, &Register::A);
            }

            // 2. LD r1,r2
            0x78 => {
                self.ld_from_r8_to_r8(&Register::A, &Register::B);
            }
            0x79 => {
                self.ld_from_r8_to_r8(&Register::A, &Register::C);
            }
            0x7a => {
                self.ld_from_r8_to_r8(&Register::A, &Register::D);
            }
            0x7b => {
                self.ld_from_r8_to_r8(&Register::A, &Register::E);
            }
            0x7c => {
                self.ld_from_r8_to_r8(&Register::A, &Register::H);
            }
            0x7d => {
                self.ld_from_r8_to_r8(&Register::A, &Register::L);
            }
            0x7e => {
                let address = self.get_hl();
                self.ld_from_memory_to_r8(&Register::A, address);
            }
            0x40 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::B);
            }
            0x41 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::C);
            }
            0x42 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::D);
            }
            0x43 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::E);
            }
            0x44 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::H);
            }
            0x45 => {
                self.ld_from_r8_to_r8(&Register::B, &Register::L);
            }
            0x46 => {
                let address = self.get_hl();
                self.ld_from_memory_to_r8(&Register::B, address);
            }

            // 3. LD A,n
            0x1a => {
                // LD A, (DE)
                let de = self.get_de();
                let d8 = self.read_byte(de);
                self.a = d8;
                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }
            0x3e => {
                // LD A, #
                let d8 = self.read_byte(self.pc + 1);
                self.a = d8;
                self.pc = self.pc.wrapping_add(2);

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

            // 15. LDI A,(HL)
            0x2a => {
                let hl = self.get_hl();
                let value = self.read_byte(hl);
                self.a = value;

                println!("LDI A,(HL): A: {:x} hl: {:x}", self.a, hl);

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
            0x87 => {
                self.add_r8(&Register::A);
            }
            0x80 => {
                self.add_r8(&Register::B);
            }
            0x81 => {
                self.add_r8(&Register::C);
            }
            0x82 => {
                self.add_r8(&Register::D);
            }
            0x83 => {
                self.add_r8(&Register::E);
            }
            0x84 => {
                self.add_r8(&Register::H);
            }
            0x85 => {
                self.add_r8(&Register::L);
            }
            0x86 => {
                let hl = self.get_hl();
                self.add_m8(hl);
            }
            0xc6 => {
                self.add_d8();
            }

            // 3. SUB n
            0x97 => {
                self.sub_r8(&Register::A);
            }
            0x90 => {
                self.sub_r8(&Register::B);
            }
            0x91 => {
                self.sub_r8(&Register::C);
            }
            0x92 => {
                self.sub_r8(&Register::D);
            }
            0x93 => {
                self.sub_r8(&Register::E);
            }
            0x94 => {
                self.sub_r8(&Register::H);
            }
            0x95 => {
                self.sub_r8(&Register::L);
            }

            // 8. CP n
            0xbf => {
                self.cp_r8(&Register::A);
            }
            0xb8 => {
                self.cp_r8(&Register::B);
            }
            0xb9 => {
                self.cp_r8(&Register::C);
            }
            0xba => {
                self.cp_r8(&Register::D);
            }
            0xbb => {
                self.cp_r8(&Register::E);
            }
            0xbc => {
                self.cp_r8(&Register::H);
            }
            0xbd => {
                self.cp_r8(&Register::L);
            }
            0xbe => {
                let a = self.a;
                let address = self.get_hl();
                let value = self.read_byte(address);

                self.set_z_flag_if(a == value);
                self.set_n_flag();
                self.set_h_flag_if(a & 0x0f < value & 0x0);
                self.set_c_flag_if(a < value);

                self.pc = self.pc.wrapping_add(1);

                self.t += 8;
                self.m += 2;
            }
            0xfe => {
                self.cp_d8();
            }

            // 5. AND n
            0xa7 => self.and_r8(&Register::A),
            0xa0 => self.and_r8(&Register::B),
            0xa1 => self.and_r8(&Register::C),
            0xa2 => self.and_r8(&Register::D),
            0xa3 => self.and_r8(&Register::E),
            0xa4 => self.and_r8(&Register::H),
            0xa5 => self.and_r8(&Register::L),
            0xa6 => {
                let hl = self.get_hl();
                self.and_m8(hl);
            }
            0xe6 => self.and_d8(),

            // 7. XOR n
            0xaf => {
                self.xor_r8(&Register::A);
            }
            0xa8 => {
                self.xor_r8(&Register::B);
            }
            0xa9 => {
                self.xor_r8(&Register::C);
            }
            0xaa => {
                self.xor_r8(&Register::D);
            }
            0xab => {
                self.xor_r8(&Register::E);
            }
            0xac => {
                self.xor_r8(&Register::H);
            }
            0xad => {
                self.xor_r8(&Register::L);
            }
            0xae => {
                let address = self.get_hl();
                self.xor_m8(address);
            }
            0xee => {
                self.xor_d8();
            }

            // 9. INC n
            0x3c => {
                // INC A
                self.inc_r8(&Register::A);
            }
            0x04 => {
                // INC B
                self.inc_r8(&Register::B);
            }
            0x0c => {
                // INC C
                self.inc_r8(&Register::C);
            }
            0x14 => {
                // INC D
                self.inc_r8(&Register::D);
            }
            0x1c => {
                // INC E
                self.inc_r8(&Register::E);
            }
            0x24 => {
                // INC H
                self.inc_r8(&Register::H);
            }
            0x2c => {
                // INC L
                self.inc_r8(&Register::L);
            }
            0x34 => {
                // INC (HL)
                let address = self.get_hl();
                self.inc_m8(address);
            }

            // 10. DEC n
            0x3d => {
                // DEC A
                self.dec_r8(&Register::A);
            }
            0x05 => {
                // DEC B
                self.dec_r8(&Register::B);
            }
            0x0d => {
                // DEC C
                self.dec_r8(&Register::C);
            }
            0x15 => {
                // DEC D
                self.dec_r8(&Register::D);
            }
            0x1d => {
                // DEC E
                self.dec_r8(&Register::E);
            }
            0x25 => {
                // DEC H
                self.dec_r8(&Register::H);
            }
            0x2d => {
                // DEC L
                self.dec_r8(&Register::L);
            }
            0x35 => {
                // DEC (HL)
                let address = self.get_hl();
                self.dec_m8(address);
            }

            // 3.3.4. 16-Bit Arithmetic
            // 3. INC nn
            0x03 => {
                self.inc_r16(&Register16::BC);
            }
            0x13 => {
                self.inc_r16(&Register16::DE);
            }
            0x23 => {
                self.inc_r16(&Register16::HL);
            }
            0x33 => {
                self.inc_r16(&Register16::SP);
            }

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

            // 6. PUSH nn
            0xf5 => {
                self.push(&Register::A, &Register::F);
            }
            0xc5 => {
                self.push(&Register::B, &Register::C);
            }
            0xd5 => {
                self.push(&Register::D, &Register::E);
            }
            0xe5 => {
                self.push(&Register::H, &Register::L);
            }

            // 7. POP nn
            0xf1 => {
                // POP AF
                self.pop(&Register::A, &Register::F);
            }
            0xc1 => {
                // POP BC
                self.pop(&Register::B, &Register::C);
            }
            0xd1 => {
                // POP DE
                self.pop(&Register::D, &Register::E);
            }
            0xe1 => {
                // POP HL
                self.pop(&Register::H, &Register::L);
            }

            // 3.3.8. Jumps
            // 1. JP nn
            0xc3 => {
                let address = self.read_d16();
                self.pc = address;

                self.t += 12;
                self.m += 3;
            }
            // 4. JR n
            0x18 => {
                self.jr(true);
            }

            // 5. JR cc,n
            0x20 => {
                self.jr(!self.get_z_flag());
            }
            0x28 => {
                self.jr(self.get_z_flag());
            }
            0x30 => {
                self.jr(!self.get_c_flag());
            }
            0x38 => {
                self.jr(self.get_c_flag());
            }

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

                self.set_z_flag_if(value == 0);
                self.reset_n_flag();
                // self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);
                self.set_c_flag_if((previous >> 7 & 1) == 1);
            }

            // prefixed
            0xcb => {
                let prefix = self.read_byte(self.pc + 1);
                match prefix {
                    // RL
                    0x17 => {
                        self.rl(&Register::A);
                    }
                    0x10 => {
                        self.rl(&Register::B);
                    }
                    0x11 => {
                        self.rl(&Register::C);
                    }
                    0x12 => {
                        self.rl(&Register::D);
                    }
                    0x13 => {
                        self.rl(&Register::E);
                    }
                    0x14 => {
                        self.rl(&Register::H);
                    }
                    0x15 => {
                        self.rl(&Register::L);
                    }

                    // BIT
                    0x7c => {
                        // BIT 7, H
                        let bit_test = self.h & 0b1000_0000;
                        if bit_test == 0b1000_0000 {
                            self.reset_z_flag();
                        } else {
                            // set Z flag
                            self.set_z_flag();
                        }

                        // reset N flag
                        self.reset_n_flag();
                        // set H flag
                        self.set_h_flag();

                        self.t += 8;
                        self.m += 2;
                    }
                    _ => {
                        panic!("unrecognized prefix {:#}", prefix);
                    }
                }

                self.pc += 2;
            }

            // 1. CALL nn
            0xcd => {
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

            // 1. RET
            0xc9 => {
                let low = self.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);
                let high = self.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);

                self.pc = (high as u16) << 8 | low as u16;

                self.t += 16;
                self.m += 4;
            }

            // 3. RETI
            0xd9 => {
                let low = self.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);

                let high = self.read_byte(self.sp);
                self.sp = self.sp.wrapping_add(1);

                self.pc = (high as u16) << 8 | low as u16;
                self.ime = true;

                self.t += 16;
                self.m += 4;
            }

            // 7. HALT
            0x76 => {
                // self.pc = self.pc.wrapping_add(1);

                self.t += 4;
                self.m += 1;
            }

            // EI
            0xfb => {
                // Disables interrupt handling by setting IME=0
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

            _ => panic!(
                "unrecognized instructions {:#x} on pc {:#x}",
                instruction, self.pc
            ),
        }

        // println!("CPU STATE after {:?}", self);
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
        self.mmu.write_byte(address, value);
    }

    fn write_byte16(&mut self, address: u16, value: u16) {
        // little endian
        self.mmu.write_byte(address, (value & 0x00ff) as u8);
        let next = address.wrapping_add(1);
        self.mmu.write_byte(next, (value >> 8) as u8);
    }

    fn read_d16(&mut self) -> u16 {
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

        self.pc = self.pc.wrapping_add(2);

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

        self.pc = self.pc.wrapping_add(2);

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

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn add_r8(&mut self, r: &Register) {
        let previous = self.a;
        let n = self.read_r8(r);

        let value = previous.wrapping_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) == 0x0f);
        self.set_c_flag_if(previous == 0xff);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn add_m8(&mut self, address: u16) {
        let previous = self.a;
        let n = self.read_byte(address);

        let value = previous.wrapping_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) == 0x0f);
        self.set_c_flag_if(previous == 0xff);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn add_d8(&mut self) {
        let previous = self.a;
        let n = self.read_byte(self.pc + 1);

        let value = previous.wrapping_add(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.set_h_flag_if((previous & 0x0f) == 0x0f);
        self.set_c_flag_if(previous == 0xff);

        self.pc = self.pc.wrapping_add(2);

        self.t += 8;
        self.m += 2;
    }

    fn sub_r8(&mut self, r: &Register) {
        let a = self.a;
        let n = self.read_r8(r);

        let value = a.wrapping_sub(n);
        self.a = value;
        self.set_z_flag_if(value == 0);
        self.set_n_flag();
        self.set_h_flag_if(a & 0x0f < n & 0x0f);
        self.set_c_flag_if(a < n);

        self.pc = self.pc.wrapping_add(1);

        self.t += 4;
        self.m += 1;
    }

    fn cp_r8(&mut self, r: &Register) {
        let a = self.a;
        let value = self.read_r8(r);

        self.set_z_flag_if(a == value);
        self.set_n_flag();
        self.set_h_flag_if(a & 0x0f < value & 0x0);
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
        self.set_h_flag_if(a & 0x0f < value & 0x0);
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

        println!("lD address({:x}) = {}", address, value);

        self.pc = self.pc.wrapping_add(1);

        self.t += 8;
        self.m += 2;
    }

    fn ld_from_r8_to_d16(&mut self, address: u16, r2: &Register) {
        let value = self.read_r8(r2);
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

    fn rl(&mut self, register: &Register) {
        // rorate left through Carry flag
        // ref. https://ja.wikipedia.org/wiki/%E3%83%93%E3%83%83%E3%83%88%E6%BC%94%E7%AE%97#/media/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB:Rotate_left_through_carry.svg
        let previous = self.read_r8(register);
        let mut value = self.read_r8(register) << 1;
        if self.get_c_flag() {
            value = value | 0x01;
        } else {
            value = value | 0x00;
        }
        self.write_r8(register, value);

        self.set_z_flag_if(value == 0);
        self.reset_n_flag();
        self.reset_h_flag();
        self.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

        self.t += 8;
        self.m += 2;
    }

    fn inc_r8(&mut self, register: &Register) {
        let previous = self.read_r8(register);
        let new_value = self.read_r8(register).wrapping_add(1);
        self.write_r8(register, new_value);

        //update the flags
        if new_value == 0 {
            self.set_z_flag();
        } else {
            self.reset_z_flag();
        }
        self.reset_n_flag();
        // Set if carry from bit 4.
        if previous == 0x10 {
            self.set_h_flag();
        } else {
            self.reset_h_flag();
        }

        self.pc = self.pc.wrapping_add(1);
    }

    fn inc_m8(&mut self, address: u16) {
        let previous = self.read_byte(address);
        let new_value = self.read_byte(address).wrapping_add(1);
        self.write_byte(address, new_value);

        //update the flags
        self.set_z_flag_if(new_value == 0);
        self.reset_n_flag();
        // Set if carry from bit 4.
        self.set_h_flag_if(previous == 0x10);

        self.pc = self.pc.wrapping_add(1);

        self.t += 12;
        self.m += 3;
    }

    fn inc_r16(&mut self, r16: &Register16) {
        let value = self.read_r16(r16);
        self.write_r16(r16, value.wrapping_add(1));

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
        self.reset_n_flag();
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

        self.pc = self.pc.wrapping_add(1);

        self.t += 16;
        self.m += 4;
    }

    fn pop(&mut self, msb: &Register, lsb: &Register) {
        let low = self.read_byte(self.sp);
        self.write_r8(lsb, low);
        self.sp = self.sp.wrapping_add(1);

        let high = self.read_byte(self.sp);
        self.write_r8(msb, high);
        self.sp = self.sp.wrapping_add(1);

        self.pc = self.pc.wrapping_add(1);

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

    fn get_af(&mut self) -> u16 {
        return ((self.a as u16) << 8) | self.f as u16;
    }

    fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xFF00) >> 8) as u8;
        self.f = (value & 0x00FF) as u8;
    }

    fn get_bc(&mut self) -> u16 {
        return ((self.b as u16) << 8) | self.c as u16;
    }

    fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xFF00) >> 8) as u8;
        self.c = (value & 0x00FF) as u8;
    }

    fn get_de(&mut self) -> u16 {
        return ((self.d as u16) << 8) | self.e as u16;
    }

    fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xFF00) >> 8) as u8;
        self.e = (value & 0x00FF) as u8;
    }

    fn get_hl(&mut self) -> u16 {
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
