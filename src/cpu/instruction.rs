use crate::cpu::opcode;
use crate::cpu::operation;
use crate::cpu::CPU;

pub fn execute(opecode: &opcode::Opcode, cpu: &mut CPU) {
    match opecode.code() {
        // nop
        0x0 => {}
        // ref. http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
        // 8-bit loads
        // 1. LD nn,n
        0x06 => {
            // LD B, n
            let n = cpu.pop_pc();
            cpu.bc.set_high(n);
        }
        0x0e => {
            // LD C, n
            let n = cpu.pop_pc();
            cpu.bc.set_low(n);
        }
        0x16 => {
            // LD D, n
            let n = cpu.pop_pc();
            cpu.de.set_high(n);
        }
        0x1e => {
            // LD E, n
            let n = cpu.pop_pc();
            cpu.de.set_low(n);
        }
        0x26 => {
            // LD h, n
            let n = cpu.pop_pc();
            cpu.hl.set_high(n);
        }
        0x2e => {
            // LD L, n
            let n = cpu.pop_pc();
            cpu.hl.set_low(n);
        }

        // 4. LD n,A
        // LD A, A
        0x7f => cpu.af.set_high(cpu.af.high()),
        0x47 => cpu.bc.set_high(cpu.af.high()),
        0x4f => cpu.bc.set_low(cpu.af.high()),
        0x57 => cpu.de.set_high(cpu.af.high()),
        0x5f => cpu.de.set_low(cpu.af.high()),
        0x67 => cpu.hl.set_high(cpu.af.high()),
        0x6f => cpu.hl.set_low(cpu.af.high()),
        0x02 => cpu.write_byte(cpu.bc.value(), cpu.af.high()),
        0x12 => cpu.write_byte(cpu.de.value(), cpu.af.high()),
        0x77 => cpu.write_byte(cpu.hl.value(), cpu.af.high()),
        0xea => {
            // LD (nn), A
            let address = cpu.pop_pc16();
            cpu.write_byte(address, cpu.af.high());
        }

        // 2. LD r1,r2
        0x78 => cpu.af.set_high(cpu.bc.high()),
        0x79 => cpu.af.set_high(cpu.bc.low()),
        0x7a => cpu.af.set_high(cpu.de.high()),
        0x7b => cpu.af.set_high(cpu.de.low()),
        0x7c => cpu.af.set_high(cpu.hl.high()),
        0x7d => cpu.af.set_high(cpu.hl.low()),
        0x7e => cpu.af.set_high(cpu.read_byte(cpu.hl.value())),
        0x0a => cpu.af.set_high(cpu.read_byte(cpu.bc.value())),
        0xfa => {
            let address = cpu.pop_pc16();
            cpu.af.set_high(cpu.read_byte(address));
        }

        0x40 => cpu.bc.set_high(cpu.bc.high()),
        0x41 => cpu.bc.set_high(cpu.bc.low()),
        0x42 => cpu.bc.set_high(cpu.de.high()),
        0x43 => cpu.bc.set_high(cpu.de.low()),
        0x44 => cpu.bc.set_high(cpu.hl.high()),
        0x45 => cpu.bc.set_high(cpu.hl.low()),
        0x46 => cpu.bc.set_high(cpu.read_byte(cpu.hl.value())),

        0x48 => cpu.bc.set_low(cpu.bc.high()),
        0x49 => cpu.bc.set_low(cpu.bc.low()),
        0x4a => cpu.bc.set_low(cpu.de.high()),
        0x4b => cpu.bc.set_low(cpu.de.low()),
        0x4c => cpu.bc.set_low(cpu.hl.high()),
        0x4d => cpu.bc.set_low(cpu.hl.low()),
        0x4e => cpu.bc.set_low(cpu.read_byte(cpu.hl.value())),

        0x50 => cpu.de.set_high(cpu.bc.high()),
        0x51 => cpu.de.set_high(cpu.bc.low()),
        0x52 => cpu.de.set_high(cpu.de.high()),
        0x53 => cpu.de.set_high(cpu.de.low()),
        0x54 => cpu.de.set_high(cpu.hl.high()),
        0x55 => cpu.de.set_high(cpu.hl.low()),
        0x56 => cpu.de.set_high(cpu.read_byte(cpu.hl.value())),

        0x58 => cpu.de.set_low(cpu.bc.high()),
        0x59 => cpu.de.set_low(cpu.bc.low()),
        0x5a => cpu.de.set_low(cpu.de.high()),
        0x5b => cpu.de.set_low(cpu.de.low()),
        0x5c => cpu.de.set_low(cpu.hl.high()),
        0x5d => cpu.de.set_low(cpu.hl.low()),
        0x5e => cpu.de.set_low(cpu.read_byte(cpu.hl.value())),

        0x60 => cpu.hl.set_high(cpu.bc.high()),
        0x61 => cpu.hl.set_high(cpu.bc.low()),
        0x62 => cpu.hl.set_high(cpu.de.high()),
        0x63 => cpu.hl.set_high(cpu.de.low()),
        0x64 => cpu.hl.set_high(cpu.hl.high()),
        0x65 => cpu.hl.set_high(cpu.hl.low()),
        0x66 => cpu.hl.set_high(cpu.read_byte(cpu.hl.value())),

        0x68 => cpu.hl.set_low(cpu.bc.high()),
        0x69 => cpu.hl.set_low(cpu.bc.low()),
        0x6a => cpu.hl.set_low(cpu.de.high()),
        0x6b => cpu.hl.set_low(cpu.de.low()),
        0x6c => cpu.hl.set_low(cpu.hl.high()),
        0x6d => cpu.hl.set_low(cpu.hl.low()),
        0x6e => cpu.hl.set_low(cpu.read_byte(cpu.hl.value())),

        0x70 => cpu.write_byte(cpu.hl.value(), cpu.bc.high()),
        0x71 => cpu.write_byte(cpu.hl.value(), cpu.bc.low()),
        0x72 => cpu.write_byte(cpu.hl.value(), cpu.de.high()),
        0x73 => cpu.write_byte(cpu.hl.value(), cpu.de.low()),
        0x74 => cpu.write_byte(cpu.hl.value(), cpu.hl.high()),
        0x75 => cpu.write_byte(cpu.hl.value(), cpu.hl.low()),
        0x36 => {
            let value = cpu.pop_pc();
            cpu.write_byte(cpu.hl.value(), value);
        }

        // 3. LD A,n
        0x1a => cpu.af.set_high(cpu.read_byte(cpu.de.value())),
        0x3e => {
            // LD A, #
            let d8 = cpu.pop_pc();
            cpu.af.set_high(d8);
        }

        // 5. LD A,(C)
        0xf2 => {
            let address = 0xff00 + cpu.bc.low() as u16;
            let value = cpu.read_byte(address);
            cpu.af.set_high(value);
        }

        // 6. LD (C),A
        0xe2 => {
            let address = 0xff00 + cpu.bc.low() as u16;
            cpu.write_byte(address, cpu.af.high());
        }

        // 9. LDD A,(HL)
        0x3a => {
            let hl = cpu.hl.value();
            let value = cpu.read_byte(hl);
            cpu.af.set_high(value);

            cpu.hl.set(hl.wrapping_sub(1));
        }

        // 15. LDI A,(HL)
        0x2a => {
            cpu.af.set_high(cpu.read_byte(cpu.hl.value()));
            // println!("LDI A,(HL): A: {:x} hl: {:x}", cpu.a, hl);
            cpu.hl.set(cpu.hl.value().wrapping_add(1));
        }

        // 18. LDI (HL),A
        0x22 => {
            cpu.write_byte(cpu.hl.value(), cpu.af.high());
            cpu.hl.set(cpu.hl.value().wrapping_add(1));
        }

        // 20. LDH A,(n)
        0xf0 => {
            // opcode = read(PC++)
            // if opcode == 0xF0:
            // n = read(PC++)
            // A = read(unsigned_16(lsb=n, msb=0xFF))
            let n = cpu.pop_pc();
            let address: u16 = 0xff00 | n as u16;
            let value = cpu.read_byte(address);
            cpu.af.set_high(value);
        }

        // 3.3.3. 8-Bit ALU
        // 1. ADD A,n
        0x80 => operation::add8(cpu, cpu.bc.high()),
        0x81 => operation::add8(cpu, cpu.bc.low()),
        0x82 => operation::add8(cpu, cpu.de.high()),
        0x83 => operation::add8(cpu, cpu.de.low()),
        0x84 => operation::add8(cpu, cpu.hl.high()),
        0x85 => operation::add8(cpu, cpu.hl.low()),
        0x86 => operation::add8(cpu, cpu.read_byte(cpu.hl.value())),
        0x87 => operation::add8(cpu, cpu.af.high()),
        0xc6 => {
            let value = cpu.pop_pc();
            operation::add8(cpu, value);
        }

        // 2. ADC A,n
        0x88 => operation::adc(cpu, cpu.bc.high()),
        0x89 => operation::adc(cpu, cpu.bc.low()),
        0x8a => operation::adc(cpu, cpu.de.high()),
        0x8b => operation::adc(cpu, cpu.de.low()),
        0x8c => operation::adc(cpu, cpu.hl.high()),
        0x8d => operation::adc(cpu, cpu.hl.low()),
        0x8e => operation::adc(cpu, cpu.read_byte(cpu.hl.value())),
        0x8f => operation::adc(cpu, cpu.af.high()),
        0xce => {
            let value = cpu.pop_pc();
            operation::adc(cpu, value)
        }

        // 3. SUB n
        0x97 => operation::sub(cpu, cpu.af.high()),
        0x90 => operation::sub(cpu, cpu.bc.high()),
        0x91 => operation::sub(cpu, cpu.bc.low()),
        0x92 => operation::sub(cpu, cpu.de.high()),
        0x93 => operation::sub(cpu, cpu.de.low()),
        0x94 => operation::sub(cpu, cpu.hl.high()),
        0x95 => operation::sub(cpu, cpu.hl.low()),
        0x96 => operation::sub(cpu, cpu.read_byte(cpu.hl.value())),
        0xd6 => {
            let value = cpu.pop_pc();
            operation::sub(cpu, value)
        }

        // 4. SBC A,n
        0x9f => operation::sbc(cpu, cpu.af.high()),
        0x98 => operation::sbc(cpu, cpu.bc.high()),
        0x99 => operation::sbc(cpu, cpu.bc.low()),
        0x9a => operation::sbc(cpu, cpu.de.high()),
        0x9b => operation::sbc(cpu, cpu.de.low()),
        0x9c => operation::sbc(cpu, cpu.hl.high()),
        0x9d => operation::sbc(cpu, cpu.hl.low()),
        0x9e => operation::sbc(cpu, cpu.read_byte(cpu.hl.value())),
        0xde => {
            let value = cpu.pop_pc();
            operation::sbc(cpu, value)
        }

        // 8. CP n
        0xbf => operation::cp(cpu, cpu.af.high()),
        0xb8 => operation::cp(cpu, cpu.bc.high()),
        0xb9 => operation::cp(cpu, cpu.bc.low()),
        0xba => operation::cp(cpu, cpu.de.high()),
        0xbb => operation::cp(cpu, cpu.de.low()),
        0xbc => operation::cp(cpu, cpu.hl.high()),
        0xbd => operation::cp(cpu, cpu.hl.low()),
        0xbe => operation::cp(cpu, cpu.read_byte(cpu.hl.value())),
        0xfe => {
            let value = cpu.pop_pc();
            operation::cp(cpu, value)
        }

        // 5. AND n
        0xa7 => operation::and(cpu, cpu.af.high()),
        0xa0 => operation::and(cpu, cpu.bc.high()),
        0xa1 => operation::and(cpu, cpu.bc.low()),
        0xa2 => operation::and(cpu, cpu.de.high()),
        0xa3 => operation::and(cpu, cpu.de.low()),
        0xa4 => operation::and(cpu, cpu.hl.high()),
        0xa5 => operation::and(cpu, cpu.hl.low()),
        0xa6 => operation::and(cpu, cpu.read_byte(cpu.hl.value())),
        0xe6 => {
            let value = cpu.pop_pc();
            operation::and(cpu, value)
        }

        // 6. OR n
        0xb7 => operation::or(cpu, cpu.af.high()),
        0xb0 => operation::or(cpu, cpu.bc.high()),
        0xb1 => operation::or(cpu, cpu.bc.low()),
        0xb2 => operation::or(cpu, cpu.de.high()),
        0xb3 => operation::or(cpu, cpu.de.low()),
        0xb4 => operation::or(cpu, cpu.hl.high()),
        0xb5 => operation::or(cpu, cpu.hl.low()),
        0xb6 => operation::or(cpu, cpu.read_byte(cpu.hl.value())),
        0xf6 => {
            let value = cpu.pop_pc();
            operation::or(cpu, value)
        }

        // 7. XOR n
        0xaf => operation::xor(cpu, cpu.af.high()),
        0xa8 => operation::xor(cpu, cpu.bc.high()),
        0xa9 => operation::xor(cpu, cpu.bc.low()),
        0xaa => operation::xor(cpu, cpu.de.high()),
        0xab => operation::xor(cpu, cpu.de.low()),
        0xac => operation::xor(cpu, cpu.hl.high()),
        0xad => operation::xor(cpu, cpu.hl.low()),
        0xae => operation::xor(cpu, cpu.read_byte(cpu.hl.value())),
        0xee => {
            let value = cpu.pop_pc();
            operation::xor(cpu, value)
        }

        // 9. INC n
        0x3c => {
            let value = operation::inc(cpu, cpu.af.high());
            cpu.af.set_high(value);
        }
        0x04 => {
            let value = operation::inc(cpu, cpu.bc.high());
            cpu.bc.set_high(value);
        }
        0x0c => {
            let value = operation::inc(cpu, cpu.bc.low());
            cpu.bc.set_low(value);
        }
        0x14 => {
            let value = operation::inc(cpu, cpu.de.high());
            cpu.de.set_high(value);
        }
        0x1c => {
            let value = operation::inc(cpu, cpu.de.low());
            cpu.de.set_low(value);
        }
        0x24 => {
            let value = operation::inc(cpu, cpu.hl.high());
            cpu.hl.set_high(value);
        }
        0x2c => {
            let value = operation::inc(cpu, cpu.hl.low());
            cpu.hl.set_low(value);
        }
        0x34 => {
            let value = operation::inc(cpu, cpu.read_byte(cpu.hl.value()));
            cpu.write_byte(cpu.hl.value(), value);
        }

        // 10. DEC n
        0x3d => {
            let value = operation::dec(cpu, cpu.af.high());
            cpu.af.set_high(value);
        }
        0x05 => {
            let value = operation::dec(cpu, cpu.bc.high());
            cpu.bc.set_high(value);
        }
        0x0d => {
            let value = operation::dec(cpu, cpu.bc.low());
            cpu.bc.set_low(value);
        }
        0x15 => {
            let value = operation::dec(cpu, cpu.de.high());
            cpu.de.set_high(value);
        }
        0x1d => {
            let value = operation::dec(cpu, cpu.de.low());
            cpu.de.set_low(value);
        }
        0x25 => {
            let value = operation::dec(cpu, cpu.hl.high());
            cpu.hl.set_high(value);
        }
        0x2d => {
            let value = operation::dec(cpu, cpu.hl.low());
            cpu.hl.set_low(value);
        }
        0x35 => {
            let value = operation::dec(cpu, cpu.read_byte(cpu.hl.value()));
            cpu.write_byte(cpu.hl.value(), value);
        }

        // 3.3.4. 16-Bit Arithmetic
        // 1. ADD HL,n
        0x09 => {
            let value = operation::add16(cpu, cpu.hl.value(), cpu.bc.value());
            cpu.hl.set(value);
        }
        0x19 => {
            let value = operation::add16(cpu, cpu.hl.value(), cpu.de.value());
            cpu.hl.set(value);
        }
        0x29 => {
            let value = operation::add16(cpu, cpu.hl.value(), cpu.hl.value());
            cpu.hl.set(value);
        }
        0x39 => {
            let value = operation::add16(cpu, cpu.hl.value(), cpu.sp);
            cpu.hl.set(value);
        }

        // 2. ADD SP,n
        0xe8 => {
            let value = cpu.sp;
            let n = cpu.pop_pc() as i8;
            let offset = n as u16;
            cpu.sp = value.wrapping_add(offset);

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            let half_carry = (value & 0x0f) + (offset & 0x0f) > 0x0f;
            cpu.set_h_flag_if(half_carry);
            let carry = (value & 0xff) + (offset & 0xff) > 0xff;
            cpu.set_c_flag_if(carry);
        }

        // 3. INC nn
        0x03 => cpu.bc.set(cpu.bc.value().wrapping_add(1)),
        0x13 => cpu.de.set(cpu.de.value().wrapping_add(1)),
        0x23 => cpu.hl.set(cpu.hl.value().wrapping_add(1)),
        0x33 => cpu.sp = cpu.sp.wrapping_add(1),

        // 4. DEC nn
        0x0b => cpu.bc.set(cpu.bc.value().wrapping_sub(1)),
        0x1b => cpu.de.set(cpu.de.value().wrapping_sub(1)),
        0x2b => cpu.hl.set(cpu.hl.value().wrapping_sub(1)),
        0x3b => cpu.sp = cpu.sp.wrapping_sub(1),

        // 19. LDH (n),A
        0xe0 => {
            let offset = cpu.pop_pc();
            let address = 0xff00 + offset as u16;
            cpu.write_byte(address, cpu.af.high());
        }

        // 16-bit Load instructions
        0x01 => {
            // LD BC, d16
            let d16 = cpu.pop_pc16();
            cpu.bc.set(d16);
        }
        0x11 => {
            // LD DE, d16
            let d16 = cpu.pop_pc16();
            cpu.de.set(d16);
        }
        0x21 => {
            // LD HL, d16
            let d16 = cpu.pop_pc16();
            cpu.hl.set(d16);
        }
        // LD SP, d16
        0x31 => cpu.sp = cpu.pop_pc16(),

        // 2. LD SP,HL
        0xf9 => cpu.sp = cpu.hl.value(),

        // 4. LDHL SP,n
        0xf8 => {
            let sp = cpu.sp;
            let n = cpu.pop_pc() as i8;
            let address = cpu.sp.wrapping_add(n as u16);
            cpu.hl.set(address);

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            // set if carry from bit-3
            cpu.set_h_flag_if((sp & 0x0f) + (n as u16 & 0x0f) > 0x0f);
            // set if carry from bit-7
            cpu.set_c_flag_if((sp & 0xff) + (n as u16 & 0xff) > 0xff);
        }

        // 5. LD (nn),SP
        0x08 => {
            let address = cpu.pop_pc16();
            cpu.write_byte16(address, cpu.sp);
        }

        // 6. PUSH nn
        0xf5 => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.af.value());
        }
        0xc5 => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.bc.value());
        }
        0xd5 => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.de.value());
        }
        0xe5 => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.hl.value());
        }

        // 7. POP nn
        0xf1 => {
            cpu.af.set(cpu.read_byte16(cpu.sp));
            cpu.sp = cpu.sp.wrapping_add(2);
        }
        0xc1 => {
            cpu.bc.set(cpu.read_byte16(cpu.sp));
            cpu.sp = cpu.sp.wrapping_add(2);
        }
        0xd1 => {
            cpu.de.set(cpu.read_byte16(cpu.sp));
            cpu.sp = cpu.sp.wrapping_add(2);
        }
        0xe1 => {
            cpu.hl.set(cpu.read_byte16(cpu.sp));
            cpu.sp = cpu.sp.wrapping_add(2);
        }

        // 3.3.8. Jumps
        // 1. JP nn
        0xc3 => operation::jp(cpu, true),
        //2. JP cc,nn
        0xc2 => operation::jp(cpu, !cpu.get_z_flag()),
        0xca => operation::jp(cpu, cpu.get_z_flag()),
        0xd2 => operation::jp(cpu, !cpu.get_c_flag()),
        0xda => operation::jp(cpu, cpu.get_c_flag()),

        // 3. JP (HL)
        0xe9 => cpu.pc = cpu.hl.value(),

        // 4. JR n
        0x18 => operation::jr(cpu, true),
        // 5. JR cc,n
        0x20 => operation::jr(cpu, !cpu.get_z_flag()),
        0x28 => operation::jr(cpu, cpu.get_z_flag()),
        0x30 => operation::jr(cpu, !cpu.get_c_flag()),
        0x38 => operation::jr(cpu, cpu.get_c_flag()),

        // 12. LDD (HL),A
        0x32 => {
            // LDD (HL), A
            // Put A into memory address HL. Decrement HL.
            cpu.write_byte(cpu.hl.value(), cpu.af.high());
            cpu.hl.set(cpu.hl.value().wrapping_sub(1));
        }

        // 3.3.6. Rotates & Shifts
        // 1. RLCA
        0x07 => {
            let previous = cpu.af.high();
            cpu.af.set_high(cpu.af.high().rotate_left(1));

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            cpu.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);

            // println!("RLCA A: {:#X} -> {:#X}", previous, cpu.a);
        }

        // 2. RLA
        0x17 => {
            let previous = cpu.af.high();
            let mut value = cpu.af.high() << 1;
            if cpu.get_c_flag() {
                value |= 0x01
            } else {
                value |= 0x00
            }
            cpu.af.set_high(value);

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            // cpu.set_c_flag_if((previous & 0b1000_0000) == 0b1000_0000);
            cpu.set_c_flag_if((previous >> 7 & 1) == 1);
        }

        // 3. RRCA
        0x0f => {
            let previous = cpu.af.high();
            cpu.af.set_high(cpu.af.high().rotate_right(1));

            // cpu.pc = cpu.pc.wrapping_add(1);

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            cpu.set_c_flag_if((previous & 0x01) > 0);
        }

        // 4. RRA
        0x1f => {
            let previous = cpu.af.high();
            let mut value = cpu.af.high() >> 1;
            if cpu.get_c_flag() {
                value |= 0x80
            } else {
                value |= 0x00
            }
            cpu.af.set_high(value);

            cpu.reset_z_flag();
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            cpu.set_c_flag_if((previous & 0x01) > 0);
        }

        // DAA
        0x27 => {
            // ref. https://ehaskins.com/2018-01-30%20Z80%20DAA/
            let mut a = cpu.af.high();

            if !cpu.get_n_flag() {
                if cpu.get_c_flag() || a > 0x99 {
                    a = a.wrapping_add(0x60);
                    cpu.set_c_flag();
                }
                if cpu.get_h_flag() || a & 0x0f > 0x09 {
                    a = a.wrapping_add(0x06);
                }
            } else {
                if cpu.get_c_flag() {
                    a = a.wrapping_sub(0x60);
                }
                if cpu.get_h_flag() {
                    a = a.wrapping_sub(0x06);
                }
            }

            cpu.af.set_high(a);

            cpu.set_z_flag_if(a == 0);
            cpu.reset_h_flag();
        }

        // prefixed
        0xcb => {
            let prefix = cpu.pop_pc();
            match prefix {
                // RLC
                0x07 => {
                    let value = operation::rlc(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x00 => {
                    let value = operation::rlc(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x01 => {
                    let value = operation::rlc(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x02 => {
                    let value = operation::rlc(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x03 => {
                    let value = operation::rlc(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x04 => {
                    let value = operation::rlc(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x05 => {
                    let value = operation::rlc(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x06 => {
                    let value = operation::rlc(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // RRC
                0x0f => {
                    let value = operation::rrc(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x08 => {
                    let value = operation::rrc(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x09 => {
                    let value = operation::rrc(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x0a => {
                    let value = operation::rrc(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x0b => {
                    let value = operation::rrc(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x0c => {
                    let value = operation::rrc(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x0d => {
                    let value = operation::rrc(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x0e => {
                    let value = operation::rrc(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // RL
                0x17 => {
                    let value = operation::rl(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x10 => {
                    let value = operation::rl(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x11 => {
                    let value = operation::rl(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x12 => {
                    let value = operation::rl(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x13 => {
                    let value = operation::rl(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x14 => {
                    let value = operation::rl(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x15 => {
                    let value = operation::rl(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x16 => {
                    let value = operation::rl(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 8. RR n
                0x1f => {
                    let value = operation::rr(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x18 => {
                    let value = operation::rr(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x19 => {
                    let value = operation::rr(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x1a => {
                    let value = operation::rr(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x1b => {
                    let value = operation::rr(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x1c => {
                    let value = operation::rr(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x1d => {
                    let value = operation::rr(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x1e => {
                    let value = operation::rr(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 9. SLA n
                0x27 => {
                    let value = operation::sla(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x20 => {
                    let value = operation::sla(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x21 => {
                    let value = operation::sla(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x22 => {
                    let value = operation::sla(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x23 => {
                    let value = operation::sla(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x24 => {
                    let value = operation::sla(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x25 => {
                    let value = operation::sla(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x26 => {
                    let value = operation::sla(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 10. SRA n
                0x2f => {
                    let value = operation::sra(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x28 => {
                    let value = operation::sra(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x29 => {
                    let value = operation::sra(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x2a => {
                    let value = operation::sra(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x2b => {
                    let value = operation::sra(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x2c => {
                    let value = operation::sra(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x2d => {
                    let value = operation::sra(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x2e => {
                    let value = operation::sra(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 1. SWAP n
                0x37 => {
                    let value = operation::swap(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x30 => {
                    let value = operation::swap(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x31 => {
                    let value = operation::swap(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x32 => {
                    let value = operation::swap(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x33 => {
                    let value = operation::swap(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x34 => {
                    let value = operation::swap(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x35 => {
                    let value = operation::swap(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x36 => {
                    let value = operation::swap(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 11. SRL n
                0x3f => {
                    let value = operation::srl(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x38 => {
                    let value = operation::srl(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x39 => {
                    let value = operation::srl(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x3a => {
                    let value = operation::srl(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x3b => {
                    let value = operation::srl(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x3c => {
                    let value = operation::srl(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x3d => {
                    let value = operation::srl(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x3e => {
                    let value = operation::srl(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // BIT
                0x40 => operation::bit(cpu, 0, cpu.bc.high()),
                0x41 => operation::bit(cpu, 0, cpu.bc.low()),
                0x42 => operation::bit(cpu, 0, cpu.de.high()),
                0x43 => operation::bit(cpu, 0, cpu.de.low()),
                0x44 => operation::bit(cpu, 0, cpu.hl.high()),
                0x45 => operation::bit(cpu, 0, cpu.hl.low()),
                0x46 => operation::bit(cpu, 0, cpu.read_byte(cpu.hl.value())),
                0x47 => operation::bit(cpu, 0, cpu.af.high()),

                0x48 => operation::bit(cpu, 1, cpu.bc.high()),
                0x49 => operation::bit(cpu, 1, cpu.bc.low()),
                0x4a => operation::bit(cpu, 1, cpu.de.high()),
                0x4b => operation::bit(cpu, 1, cpu.de.low()),
                0x4c => operation::bit(cpu, 1, cpu.hl.high()),
                0x4d => operation::bit(cpu, 1, cpu.hl.low()),
                0x4e => operation::bit(cpu, 1, cpu.read_byte(cpu.hl.value())),
                0x4f => operation::bit(cpu, 1, cpu.af.high()),

                0x50 => operation::bit(cpu, 2, cpu.bc.high()),
                0x51 => operation::bit(cpu, 2, cpu.bc.low()),
                0x52 => operation::bit(cpu, 2, cpu.de.high()),
                0x53 => operation::bit(cpu, 2, cpu.de.low()),
                0x54 => operation::bit(cpu, 2, cpu.hl.high()),
                0x55 => operation::bit(cpu, 2, cpu.hl.low()),
                0x56 => operation::bit(cpu, 2, cpu.read_byte(cpu.hl.value())),
                0x57 => operation::bit(cpu, 2, cpu.af.high()),

                0x58 => operation::bit(cpu, 3, cpu.bc.high()),
                0x59 => operation::bit(cpu, 3, cpu.bc.low()),
                0x5a => operation::bit(cpu, 3, cpu.de.high()),
                0x5b => operation::bit(cpu, 3, cpu.de.low()),
                0x5c => operation::bit(cpu, 3, cpu.hl.high()),
                0x5d => operation::bit(cpu, 3, cpu.hl.low()),
                0x5e => operation::bit(cpu, 3, cpu.read_byte(cpu.hl.value())),
                0x5f => operation::bit(cpu, 3, cpu.af.high()),

                0x60 => operation::bit(cpu, 4, cpu.bc.high()),
                0x61 => operation::bit(cpu, 4, cpu.bc.low()),
                0x62 => operation::bit(cpu, 4, cpu.de.high()),
                0x63 => operation::bit(cpu, 4, cpu.de.low()),
                0x64 => operation::bit(cpu, 4, cpu.hl.high()),
                0x65 => operation::bit(cpu, 4, cpu.hl.low()),
                0x66 => operation::bit(cpu, 4, cpu.read_byte(cpu.hl.value())),
                0x67 => operation::bit(cpu, 4, cpu.af.high()),

                0x68 => operation::bit(cpu, 5, cpu.bc.high()),
                0x69 => operation::bit(cpu, 5, cpu.bc.low()),
                0x6a => operation::bit(cpu, 5, cpu.de.high()),
                0x6b => operation::bit(cpu, 5, cpu.de.low()),
                0x6c => operation::bit(cpu, 5, cpu.hl.high()),
                0x6d => operation::bit(cpu, 5, cpu.hl.low()),
                0x6e => operation::bit(cpu, 5, cpu.read_byte(cpu.hl.value())),
                0x6f => operation::bit(cpu, 5, cpu.af.high()),

                0x70 => operation::bit(cpu, 6, cpu.bc.high()),
                0x71 => operation::bit(cpu, 6, cpu.bc.low()),
                0x72 => operation::bit(cpu, 6, cpu.de.high()),
                0x73 => operation::bit(cpu, 6, cpu.de.low()),
                0x74 => operation::bit(cpu, 6, cpu.hl.high()),
                0x75 => operation::bit(cpu, 6, cpu.hl.low()),
                0x76 => operation::bit(cpu, 6, cpu.read_byte(cpu.hl.value())),
                0x77 => operation::bit(cpu, 6, cpu.af.high()),

                0x78 => operation::bit(cpu, 7, cpu.bc.high()),
                0x79 => operation::bit(cpu, 7, cpu.bc.low()),
                0x7a => operation::bit(cpu, 7, cpu.de.high()),
                0x7b => operation::bit(cpu, 7, cpu.de.low()),
                0x7c => operation::bit(cpu, 7, cpu.hl.high()),
                0x7d => operation::bit(cpu, 7, cpu.hl.low()),
                0x7e => operation::bit(cpu, 7, cpu.read_byte(cpu.hl.value())),
                0x7f => operation::bit(cpu, 7, cpu.af.high()),

                // 3. RES b,r
                0x80 => cpu.bc.set_high(cpu.bc.high() & !(1 << 0)),
                0x81 => cpu.bc.set_low(cpu.bc.low() & !(1 << 0)),
                0x82 => cpu.de.set_high(cpu.de.high() & !(1 << 0)),
                0x83 => cpu.de.set_low(cpu.de.low() & !(1 << 0)),
                0x84 => cpu.hl.set_high(cpu.hl.high() & !(1 << 0)),
                0x85 => cpu.hl.set_low(cpu.hl.low() & !(1 << 0)),
                0x86 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 0)),
                0x87 => cpu.af.set_high(cpu.af.high() & !(1 << 0)),

                0x88 => cpu.bc.set_high(cpu.bc.high() & !(1 << 1)),
                0x89 => cpu.bc.set_low(cpu.bc.low() & !(1 << 1)),
                0x8a => cpu.de.set_high(cpu.de.high() & !(1 << 1)),
                0x8b => cpu.de.set_low(cpu.de.low() & !(1 << 1)),
                0x8c => cpu.hl.set_high(cpu.hl.high() & !(1 << 1)),
                0x8d => cpu.hl.set_low(cpu.hl.low() & !(1 << 1)),
                0x8e => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 1)),
                0x8f => cpu.af.set_high(cpu.af.high() & !(1 << 1)),

                0x90 => cpu.bc.set_high(cpu.bc.high() & !(1 << 2)),
                0x91 => cpu.bc.set_low(cpu.bc.low() & !(1 << 2)),
                0x92 => cpu.de.set_high(cpu.de.high() & !(1 << 2)),
                0x93 => cpu.de.set_low(cpu.de.low() & !(1 << 2)),
                0x94 => cpu.hl.set_high(cpu.hl.high() & !(1 << 2)),
                0x95 => cpu.hl.set_low(cpu.hl.low() & !(1 << 2)),
                0x96 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 2)),
                0x97 => cpu.af.set_high(cpu.af.high() & !(1 << 2)),

                0x98 => cpu.bc.set_high(cpu.bc.high() & !(1 << 3)),
                0x99 => cpu.bc.set_low(cpu.bc.low() & !(1 << 3)),
                0x9a => cpu.de.set_high(cpu.de.high() & !(1 << 3)),
                0x9b => cpu.de.set_low(cpu.de.low() & !(1 << 3)),
                0x9c => cpu.hl.set_high(cpu.hl.high() & !(1 << 3)),
                0x9d => cpu.hl.set_low(cpu.hl.low() & !(1 << 3)),
                0x9e => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 3)),
                0x9f => cpu.af.set_high(cpu.af.high() & !(1 << 3)),

                0xa0 => cpu.bc.set_high(cpu.bc.high() & !(1 << 4)),
                0xa1 => cpu.bc.set_low(cpu.bc.low() & !(1 << 4)),
                0xa2 => cpu.de.set_high(cpu.de.high() & !(1 << 4)),
                0xa3 => cpu.de.set_low(cpu.de.low() & !(1 << 4)),
                0xa4 => cpu.hl.set_high(cpu.hl.high() & !(1 << 4)),
                0xa5 => cpu.hl.set_low(cpu.hl.low() & !(1 << 4)),
                0xa6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 4)),
                0xa7 => cpu.af.set_high(cpu.af.high() & !(1 << 4)),

                0xa8 => cpu.bc.set_high(cpu.bc.high() & !(1 << 5)),
                0xa9 => cpu.bc.set_low(cpu.bc.low() & !(1 << 5)),
                0xaa => cpu.de.set_high(cpu.de.high() & !(1 << 5)),
                0xab => cpu.de.set_low(cpu.de.low() & !(1 << 5)),
                0xac => cpu.hl.set_high(cpu.hl.high() & !(1 << 5)),
                0xad => cpu.hl.set_low(cpu.hl.low() & !(1 << 5)),
                0xae => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 5)),
                0xaf => cpu.af.set_high(cpu.af.high() & !(1 << 5)),

                0xb0 => cpu.bc.set_high(cpu.bc.high() & !(1 << 6)),
                0xb1 => cpu.bc.set_low(cpu.bc.low() & !(1 << 6)),
                0xb2 => cpu.de.set_high(cpu.de.high() & !(1 << 6)),
                0xb3 => cpu.de.set_low(cpu.de.low() & !(1 << 6)),
                0xb4 => cpu.hl.set_high(cpu.hl.high() & !(1 << 6)),
                0xb5 => cpu.hl.set_low(cpu.hl.low() & !(1 << 6)),
                0xb6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 6)),
                0xb7 => cpu.af.set_high(cpu.af.high() & !(1 << 6)),

                0xb8 => cpu.bc.set_high(cpu.bc.high() & !(1 << 7)),
                0xb9 => cpu.bc.set_low(cpu.bc.low() & !(1 << 7)),
                0xba => cpu.de.set_high(cpu.de.high() & !(1 << 7)),
                0xbb => cpu.de.set_low(cpu.de.low() & !(1 << 7)),
                0xbc => cpu.hl.set_high(cpu.hl.high() & !(1 << 7)),
                0xbd => cpu.hl.set_low(cpu.hl.low() & !(1 << 7)),
                0xbe => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) & !(1 << 7)),
                0xbf => cpu.af.set_high(cpu.af.high() & !(1 << 7)),

                // 2. SET b,r
                0xc0 => cpu.bc.set_high(cpu.bc.high() | (1 << 0)),
                0xc1 => cpu.bc.set_low(cpu.bc.low() | (1 << 0)),
                0xc2 => cpu.de.set_high(cpu.de.high() | (1 << 0)),
                0xc3 => cpu.de.set_low(cpu.de.low() | (1 << 0)),
                0xc4 => cpu.hl.set_high(cpu.hl.high() | (1 << 0)),
                0xc5 => cpu.hl.set_low(cpu.hl.low() | (1 << 0)),
                0xc6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 0)),
                0xc7 => cpu.af.set_high(cpu.af.high() | (1 << 0)),

                0xc8 => cpu.bc.set_high(cpu.bc.high() | (1 << 1)),
                0xc9 => cpu.bc.set_low(cpu.bc.low() | (1 << 1)),
                0xca => cpu.de.set_high(cpu.de.high() | (1 << 1)),
                0xcb => cpu.de.set_low(cpu.de.low() | (1 << 1)),
                0xcc => cpu.hl.set_high(cpu.hl.high() | (1 << 1)),
                0xcd => cpu.hl.set_low(cpu.hl.low() | (1 << 1)),
                0xce => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 1)),
                0xcf => cpu.af.set_high(cpu.af.high() | (1 << 1)),

                0xd0 => cpu.bc.set_high(cpu.bc.high() | (1 << 2)),
                0xd1 => cpu.bc.set_low(cpu.bc.low() | (1 << 2)),
                0xd2 => cpu.de.set_high(cpu.de.high() | (1 << 2)),
                0xd3 => cpu.de.set_low(cpu.de.low() | (1 << 2)),
                0xd4 => cpu.hl.set_high(cpu.hl.high() | (1 << 2)),
                0xd5 => cpu.hl.set_low(cpu.hl.low() | (1 << 2)),
                0xd6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 2)),
                0xd7 => cpu.af.set_high(cpu.af.high() | (1 << 2)),

                0xd8 => cpu.bc.set_high(cpu.bc.high() | (1 << 3)),
                0xd9 => cpu.bc.set_low(cpu.bc.low() | (1 << 3)),
                0xda => cpu.de.set_high(cpu.de.high() | (1 << 3)),
                0xdb => cpu.de.set_low(cpu.de.low() | (1 << 3)),
                0xdc => cpu.hl.set_high(cpu.hl.high() | (1 << 3)),
                0xdd => cpu.hl.set_low(cpu.hl.low() | (1 << 3)),
                0xde => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 3)),
                0xdf => cpu.af.set_high(cpu.af.high() | (1 << 3)),

                0xe0 => cpu.bc.set_high(cpu.bc.high() | (1 << 4)),
                0xe1 => cpu.bc.set_low(cpu.bc.low() | (1 << 4)),
                0xe2 => cpu.de.set_high(cpu.de.high() | (1 << 4)),
                0xe3 => cpu.de.set_low(cpu.de.low() | (1 << 4)),
                0xe4 => cpu.hl.set_high(cpu.hl.high() | (1 << 4)),
                0xe5 => cpu.hl.set_low(cpu.hl.low() | (1 << 4)),
                0xe6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 4)),
                0xe7 => cpu.af.set_high(cpu.af.high() | (1 << 4)),

                0xe8 => cpu.bc.set_high(cpu.bc.high() | (1 << 5)),
                0xe9 => cpu.bc.set_low(cpu.bc.low() | (1 << 5)),
                0xea => cpu.de.set_high(cpu.de.high() | (1 << 5)),
                0xeb => cpu.de.set_low(cpu.de.low() | (1 << 5)),
                0xec => cpu.hl.set_high(cpu.hl.high() | (1 << 5)),
                0xed => cpu.hl.set_low(cpu.hl.low() | (1 << 5)),
                0xee => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 5)),
                0xef => cpu.af.set_high(cpu.af.high() | (1 << 5)),

                0xf0 => cpu.bc.set_high(cpu.bc.high() | (1 << 6)),
                0xf1 => cpu.bc.set_low(cpu.bc.low() | (1 << 6)),
                0xf2 => cpu.de.set_high(cpu.de.high() | (1 << 6)),
                0xf3 => cpu.de.set_low(cpu.de.low() | (1 << 6)),
                0xf4 => cpu.hl.set_high(cpu.hl.high() | (1 << 6)),
                0xf5 => cpu.hl.set_low(cpu.hl.low() | (1 << 6)),
                0xf6 => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 6)),
                0xf7 => cpu.af.set_high(cpu.af.high() | (1 << 6)),

                0xf8 => cpu.bc.set_high(cpu.bc.high() | (1 << 7)),
                0xf9 => cpu.bc.set_low(cpu.bc.low() | (1 << 7)),
                0xfa => cpu.de.set_high(cpu.de.high() | (1 << 7)),
                0xfb => cpu.de.set_low(cpu.de.low() | (1 << 7)),
                0xfc => cpu.hl.set_high(cpu.hl.high() | (1 << 7)),
                0xfd => cpu.hl.set_low(cpu.hl.low() | (1 << 7)),
                0xfe => cpu.write_byte(cpu.hl.value(), cpu.read_byte(cpu.hl.value()) | (1 << 7)),
                0xff => cpu.af.set_high(cpu.af.high() | (1 << 7)),
            }
        }

        // 1. CALL nn
        0xcd => operation::call(cpu),

        // 2. CALL cc,nn
        0xc4 => operation::call_if(cpu, !cpu.get_z_flag()),
        0xcc => operation::call_if(cpu, cpu.get_z_flag()),
        0xd4 => operation::call_if(cpu, !cpu.get_c_flag()),
        0xdc => operation::call_if(cpu, cpu.get_c_flag()),

        // 1. RST n
        0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.pc);

            // cpu.pc = 0x0000 + opecode.code() as u16 - 0x00c7;
            cpu.pc = opecode.code() as u16 - 0x00c7;
        }

        // 1. RET
        0xc9 => operation::ret(cpu),

        // 2. RET cc
        0xc0 => {
            if !cpu.get_z_flag() {
                operation::ret(cpu);
            }
        }
        0xc8 => {
            if cpu.get_z_flag() {
                operation::ret(cpu);
            }
        }
        0xd0 => {
            if !cpu.get_c_flag() {
                operation::ret(cpu);
            }
        }
        0xd8 => {
            if cpu.get_c_flag() {
                operation::ret(cpu);
            }
        }

        // 3. RETI
        0xd9 => {
            operation::ret(cpu);
            cpu.ime = true;
        }

        // CPL
        0x2f => {
            // Complement A register. (Flip all bits.)
            cpu.af.set_high(!cpu.af.high());

            cpu.set_n_flag();
            cpu.set_h_flag();
        }

        // 4. CCF
        0x3f => {
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            if cpu.get_c_flag() {
                cpu.reset_c_flag();
            } else {
                cpu.set_c_flag();
            }
        }

        // 5. SCF
        0x37 => {
            cpu.reset_n_flag();
            cpu.reset_h_flag();
            cpu.set_c_flag();
        }

        // 7. HALT
        0x76 => {
            cpu.halt = true;
        }

        // 8. STOP
        0x10 => {}

        // EI
        0xfb => {
            cpu.ime = true;
        }

        // DI
        0xf3 => {
            // Disables interrupt handling by setting IME=0
            cpu.ime = false;
        }

        0xd3 | 0xdb | 0xdd | 0xe3..=0xe4 | 0xeb..=0xed | 0xf4 | 0xfc..=0xfd => panic!(
            "unrecognized instructions {:#x} on pc {:#x}",
            opecode.code(),
            cpu.pc
        ),
    }
}
