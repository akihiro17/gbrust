// use crate::cpu::operations::Operations;
use crate::cpu::{Register, Register16, CPU};

// do operations
// update flags
fn inc(cpu: &mut CPU, original: u8) -> u8 {
    let new_value = original.wrapping_add(1);

    //update the flags
    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    // Set if carry from bit 3
    cpu.set_h_flag_if(original & 0x0f == 0x0f);

    return new_value;
}

fn dec(cpu: &mut CPU, original: u8) -> u8 {
    let new_value = original.wrapping_sub(1);

    //update the flags
    cpu.set_z_flag_if(new_value == 0);
    cpu.set_n_flag();
    // Set if carry from bit 4.
    cpu.set_h_flag_if(original & 0x0f == 0);

    return new_value;
}

fn add8(cpu: &mut CPU, value: u8) {
    let original = cpu.af.high();

    let (new_value, carry) = original.overflowing_add(value);
    cpu.af.set_high(new_value);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag_if((original & 0x0f) + (value & 0x0f) > 0x0f);
    cpu.set_c_flag_if(carry);
}

fn adc8(cpu: &mut CPU, op2: u8) {
    let op1 = cpu.af.high();

    let c = match cpu.get_c_flag() {
        true => 1,
        false => 0,
    };

    let new_value = op1.wrapping_add(op2).wrapping_add(c);
    cpu.af.set_high(new_value);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag_if((op1 & 0xf) + (op2 & 0xf) + c > 0xf);
    let carry = (op1 as u16) + (op2 as u16) + (c as u16) > 0xff;
    cpu.set_c_flag_if(carry);
}

fn sub8(cpu: &mut CPU, op2: u8) {
    let op1 = cpu.af.high();

    let (value, carry) = op1.overflowing_sub(op2);
    cpu.af.set_high(value);

    cpu.set_z_flag_if(value == 0);
    cpu.set_n_flag();
    cpu.set_h_flag_if(op1 & 0x0f < op2 & 0x0f);
    cpu.set_c_flag_if(carry);
}

fn sbc8(cpu: &mut CPU, op2: u8) {
    let op1 = cpu.af.high();
    let c = if cpu.get_c_flag() { 1 } else { 0 };

    let value = op1.wrapping_sub(op2).wrapping_sub(c);
    cpu.af.set_high(value);

    cpu.set_z_flag_if(value == 0);
    cpu.set_n_flag();
    cpu.set_h_flag_if(op1 & 0x0f < (op2 & 0x0f) + c);
    // n + c > u8Max
    cpu.set_c_flag_if((op1 as u16) < (op2 as u16) + (c as u16));
}

fn cp8(cpu: &mut CPU, op1: u8) {
    let a = cpu.af.high();

    cpu.set_z_flag_if(a == op1);
    cpu.set_n_flag();
    cpu.set_h_flag_if((a & 0x0f) < (op1 & 0x0f));
    cpu.set_c_flag_if(a < op1);
}

fn or8(cpu: &mut CPU, op1: u8) {
    cpu.af.set_high(cpu.af.high() | op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();
}

fn and8(cpu: &mut CPU, op1: u8) {
    cpu.af.set_high(cpu.af.high() & op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag();
    cpu.reset_c_flag();
}

fn xor8(cpu: &mut CPU, op1: u8) {
    cpu.af.set_high(cpu.af.high() ^ op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();
}

fn add16(cpu: &mut CPU, op1: u16, op2: u16) -> u16 {
    let new_value = op1.wrapping_add(op2);

    cpu.reset_n_flag();
    let half_carry = (op1 & 0x0fff) + (op2 & 0x0fff) > 0x0fff;
    cpu.set_h_flag_if(half_carry);
    let carry = (op1 as u32) + (op2 as u32) > 0xffff;
    cpu.set_c_flag_if(carry);

    return new_value;
}

fn jp(cpu: &mut CPU, condition: bool) {
    let address = cpu.pop_pc16();
    if condition {
        cpu.pc = address;
    }
}

fn jr(cpu: &mut CPU, condition: bool) {
    // n = one byte ##signed## immediate value

    // opcode = read(PC++)
    // if opcode in [0x20, 0x30, 0x28, 0x38]:
    // e = signed_8(read(PC++))
    // if F.check_condition(cc):
    // PC = PC + e
    let n = cpu.pop_pc() as i8;

    if condition {
        cpu.pc = cpu.pc.wrapping_add(n as u16);
    }
    // println!("JR to #{:X} offset: #{:?}", self.pc, n);
}

fn rlc(cpu: &mut CPU, op1: u8) -> u8 {
    // Rotate n left. Old bit 7 to Carry flag.
    let new_value = op1.rotate_left(1);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    return new_value;
}

fn rrc(cpu: &mut CPU, op1: u8) -> u8 {
    // Rotate n right. Old bit 0 to Carry flag.
    let new_value = op1.rotate_right(1);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    return new_value;
}

fn rr(cpu: &mut CPU, op1: u8) -> u8 {
    let mut new_value = op1 >> 1;
    if cpu.get_c_flag() {
        new_value = new_value | 0x80;
    } else {
        new_value = new_value | 0x00;
    }

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    return new_value;
}

fn rl(cpu: &mut CPU, op1: u8) -> u8 {
    // rorate left through Carry flag
    // ref. https://ja.wikipedia.org/wiki/%E3%83%93%E3%83%83%E3%83%88%E6%BC%94%E7%AE%97#/media/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB:Rotate_left_through_carry.svg
    let mut new_value = op1 << 1;
    if cpu.get_c_flag() {
        new_value = new_value | 0x01;
    } else {
        new_value = new_value | 0x00;
    }

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    return new_value;
}

fn sla(cpu: &mut CPU, op1: u8) -> u8 {
    let new_value = op1 << 1;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    return new_value;
}

fn sra(cpu: &mut CPU, op1: u8) -> u8 {
    // Shift n right into Carry. MSB doesn't change.
    let mut new_value = op1 >> 1;
    new_value = new_value | (op1 & 0x80);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    return new_value;
}

fn srl(cpu: &mut CPU, op1: u8) -> u8 {
    // Shift n right into Carry. MSB set to 0.
    let new_value = op1 >> 1;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    return new_value;
}

fn swap(cpu: &mut CPU, op1: u8) -> u8 {
    let new_value = (op1 & 0x0f) << 4 | (op1 & 0xf0) >> 4;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();

    return new_value;
}

fn bit(cpu: &mut CPU, n: u8, value: u8) {
    let mask = 0x0001 << n;
    let bit_test = value & mask;

    cpu.set_z_flag_if(bit_test == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag();
}

fn ret(cpu: &mut CPU) {
    cpu.pc = cpu.read_byte16(cpu.sp);
    cpu.sp = cpu.sp.wrapping_add(2);
}

fn call(cpu: &mut CPU) {
    // opcode = read(PC++)
    // if opcode == 0xCD:
    // nn = unsigned_16(lsb=read(PC++), msb=read(PC++))
    // write(--SP, msb(PC))
    // write(--SP, lsb(PC))
    // PC = nn
    let next = cpu.pop_pc16();

    cpu.sp = cpu.sp.wrapping_sub(2);
    cpu.write_byte16(cpu.sp, cpu.pc);

    cpu.pc = next;
}

fn call_if(cpu: &mut CPU, condition: bool) {
    if condition {
        call(cpu);
    } else {
        cpu.pc = cpu.pc.wrapping_add(2);
    }
}

pub fn execute(opecode: u8, cpu: &mut CPU) {
    match opecode {
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

        0x47 => cpu.ld_from_r8_to_r8(&Register::B, &Register::A),
        0x4f => cpu.ld_from_r8_to_r8(&Register::C, &Register::A),
        0x57 => cpu.ld_from_r8_to_r8(&Register::D, &Register::A),
        0x5f => cpu.ld_from_r8_to_r8(&Register::E, &Register::A),
        0x67 => cpu.ld_from_r8_to_r8(&Register::H, &Register::A),
        0x6f => cpu.ld_from_r8_to_r8(&Register::L, &Register::A),
        0x02 => cpu.ld_from_r8_to_m8(cpu.get_bc(), &Register::A),
        0x12 => cpu.ld_from_r8_to_m8(cpu.get_de(), &Register::A),
        0x77 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::A),
        0xea => {
            // LD (nn), A
            let address = cpu.pop_pc16();
            cpu.ld_from_r8_to_d16(address, &Register::A);
        }

        // 2. LD r1,r2
        0x78 => cpu.ld_from_r8_to_r8(&Register::A, &Register::B),
        0x79 => cpu.ld_from_r8_to_r8(&Register::A, &Register::C),
        0x7a => cpu.ld_from_r8_to_r8(&Register::A, &Register::D),
        0x7b => cpu.ld_from_r8_to_r8(&Register::A, &Register::E),
        0x7c => cpu.ld_from_r8_to_r8(&Register::A, &Register::H),
        0x7d => cpu.ld_from_r8_to_r8(&Register::A, &Register::L),
        0x7e => cpu.ld_from_memory_to_r8(&Register::A, cpu.get_hl()),
        0x0a => cpu.ld_from_memory_to_r8(&Register::A, cpu.get_bc()),
        0xfa => cpu.ld_from_d16_to_r8(&Register::A),

        0x40 => cpu.ld_from_r8_to_r8(&Register::B, &Register::B),
        0x41 => cpu.ld_from_r8_to_r8(&Register::B, &Register::C),
        0x42 => cpu.ld_from_r8_to_r8(&Register::B, &Register::D),
        0x43 => cpu.ld_from_r8_to_r8(&Register::B, &Register::E),
        0x44 => cpu.ld_from_r8_to_r8(&Register::B, &Register::H),
        0x45 => cpu.ld_from_r8_to_r8(&Register::B, &Register::L),
        0x46 => cpu.ld_from_memory_to_r8(&Register::B, cpu.get_hl()),

        0x48 => cpu.ld_from_r8_to_r8(&Register::C, &Register::B),
        0x49 => cpu.ld_from_r8_to_r8(&Register::C, &Register::C),
        0x4a => cpu.ld_from_r8_to_r8(&Register::C, &Register::D),
        0x4b => cpu.ld_from_r8_to_r8(&Register::C, &Register::E),
        0x4c => cpu.ld_from_r8_to_r8(&Register::C, &Register::H),
        0x4d => cpu.ld_from_r8_to_r8(&Register::C, &Register::L),
        0x4e => cpu.ld_from_memory_to_r8(&Register::C, cpu.get_hl()),

        0x50 => cpu.ld_from_r8_to_r8(&Register::D, &Register::B),
        0x51 => cpu.ld_from_r8_to_r8(&Register::D, &Register::C),
        0x52 => cpu.ld_from_r8_to_r8(&Register::D, &Register::D),
        0x53 => cpu.ld_from_r8_to_r8(&Register::D, &Register::E),
        0x54 => cpu.ld_from_r8_to_r8(&Register::D, &Register::H),
        0x55 => cpu.ld_from_r8_to_r8(&Register::D, &Register::L),
        0x56 => cpu.ld_from_memory_to_r8(&Register::D, cpu.get_hl()),

        0x58 => cpu.ld_from_r8_to_r8(&Register::E, &Register::B),
        0x59 => cpu.ld_from_r8_to_r8(&Register::E, &Register::C),
        0x5a => cpu.ld_from_r8_to_r8(&Register::E, &Register::D),
        0x5b => cpu.ld_from_r8_to_r8(&Register::E, &Register::E),
        0x5c => cpu.ld_from_r8_to_r8(&Register::E, &Register::H),
        0x5d => cpu.ld_from_r8_to_r8(&Register::E, &Register::L),
        0x5e => cpu.ld_from_memory_to_r8(&Register::E, cpu.get_hl()),

        0x60 => cpu.ld_from_r8_to_r8(&Register::H, &Register::B),
        0x61 => cpu.ld_from_r8_to_r8(&Register::H, &Register::C),
        0x62 => cpu.ld_from_r8_to_r8(&Register::H, &Register::D),
        0x63 => cpu.ld_from_r8_to_r8(&Register::H, &Register::E),
        0x64 => cpu.ld_from_r8_to_r8(&Register::H, &Register::H),
        0x65 => cpu.ld_from_r8_to_r8(&Register::H, &Register::L),
        0x66 => cpu.ld_from_memory_to_r8(&Register::H, cpu.get_hl()),

        0x68 => cpu.ld_from_r8_to_r8(&Register::L, &Register::B),
        0x69 => cpu.ld_from_r8_to_r8(&Register::L, &Register::C),
        0x6a => cpu.ld_from_r8_to_r8(&Register::L, &Register::D),
        0x6b => cpu.ld_from_r8_to_r8(&Register::L, &Register::E),
        0x6c => cpu.ld_from_r8_to_r8(&Register::L, &Register::H),
        0x6d => cpu.ld_from_r8_to_r8(&Register::L, &Register::L),
        0x6e => cpu.ld_from_memory_to_r8(&Register::L, cpu.get_hl()),

        0x70 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::B),
        0x71 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::C),
        0x72 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::D),
        0x73 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::E),
        0x74 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::H),
        0x75 => cpu.ld_from_r8_to_m8(cpu.get_hl(), &Register::L),
        0x36 => cpu.ld_from_d16_to_m8(cpu.get_hl()),

        // 3. LD A,n
        0x1a => cpu.ld_from_memory_to_r8(&Register::A, cpu.get_de()),
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
            let hl = cpu.get_hl();
            let value = cpu.read_byte(hl);
            cpu.af.set_high(value);

            cpu.set_hl(hl.wrapping_sub(1));
        }

        // 15. LDI A,(HL)
        0x2a => {
            let hl = cpu.get_hl();
            let value = cpu.read_byte(hl);
            cpu.af.set_high(value);

            // println!("LDI A,(HL): A: {:x} hl: {:x}", cpu.a, hl);

            cpu.set_hl(hl.wrapping_add(1));
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
            cpu.write_r8(&Register::A, value);
        }

        // 3.3.3. 8-Bit ALU
        // 1. ADD A,n
        0x80 => add8(cpu, cpu.bc.high()),
        0x81 => add8(cpu, cpu.bc.low()),
        0x82 => add8(cpu, cpu.de.high()),
        0x83 => add8(cpu, cpu.de.low()),
        0x84 => add8(cpu, cpu.hl.high()),
        0x85 => add8(cpu, cpu.hl.low()),
        0x86 => add8(cpu, cpu.read_byte(cpu.hl.value())),
        0x87 => add8(cpu, cpu.af.high()),
        0xc6 => {
            let value = cpu.pop_pc();
            add8(cpu, value);
        }

        // 2. ADC A,n
        0x88 => adc8(cpu, cpu.bc.high()),
        0x89 => adc8(cpu, cpu.bc.low()),
        0x8a => adc8(cpu, cpu.de.high()),
        0x8b => adc8(cpu, cpu.de.low()),
        0x8c => adc8(cpu, cpu.hl.high()),
        0x8d => adc8(cpu, cpu.hl.low()),
        0x8e => adc8(cpu, cpu.read_byte(cpu.hl.value())),
        0x8f => adc8(cpu, cpu.af.high()),
        0xce => {
            let value = cpu.pop_pc();
            adc8(cpu, value)
        }

        // 3. SUB n
        0x97 => sub8(cpu, cpu.af.high()),
        0x90 => sub8(cpu, cpu.bc.high()),
        0x91 => sub8(cpu, cpu.bc.low()),
        0x92 => sub8(cpu, cpu.de.high()),
        0x93 => sub8(cpu, cpu.de.low()),
        0x94 => sub8(cpu, cpu.hl.high()),
        0x95 => sub8(cpu, cpu.hl.low()),
        0x96 => sub8(cpu, cpu.read_byte(cpu.hl.value())),
        0xd6 => {
            let value = cpu.pop_pc();
            sub8(cpu, value)
        }

        // 4. SBC A,n
        0x9f => sbc8(cpu, cpu.af.high()),
        0x98 => sbc8(cpu, cpu.bc.high()),
        0x99 => sbc8(cpu, cpu.bc.low()),
        0x9a => sbc8(cpu, cpu.de.high()),
        0x9b => sbc8(cpu, cpu.de.low()),
        0x9c => sbc8(cpu, cpu.hl.high()),
        0x9d => sbc8(cpu, cpu.hl.low()),
        0x9e => sbc8(cpu, cpu.read_byte(cpu.hl.value())),
        0xde => {
            let value = cpu.pop_pc();
            sbc8(cpu, value)
        }

        // 8. CP n
        0xbf => cp8(cpu, cpu.af.high()),
        0xb8 => cp8(cpu, cpu.bc.high()),
        0xb9 => cp8(cpu, cpu.bc.low()),
        0xba => cp8(cpu, cpu.de.high()),
        0xbb => cp8(cpu, cpu.de.low()),
        0xbc => cp8(cpu, cpu.hl.high()),
        0xbd => cp8(cpu, cpu.hl.low()),
        0xbe => cp8(cpu, cpu.read_byte(cpu.hl.value())),
        0xfe => {
            let value = cpu.pop_pc();
            cp8(cpu, value)
        }

        // 5. AND n
        0xa7 => and8(cpu, cpu.af.high()),
        0xa0 => and8(cpu, cpu.bc.high()),
        0xa1 => and8(cpu, cpu.bc.low()),
        0xa2 => and8(cpu, cpu.de.high()),
        0xa3 => and8(cpu, cpu.de.low()),
        0xa4 => and8(cpu, cpu.hl.high()),
        0xa5 => and8(cpu, cpu.hl.low()),
        0xa6 => and8(cpu, cpu.read_byte(cpu.hl.value())),
        0xe6 => {
            let value = cpu.pop_pc();
            and8(cpu, value)
        }

        // 6. OR n
        0xb7 => or8(cpu, cpu.af.high()),
        0xb0 => or8(cpu, cpu.bc.high()),
        0xb1 => or8(cpu, cpu.bc.low()),
        0xb2 => or8(cpu, cpu.de.high()),
        0xb3 => or8(cpu, cpu.de.low()),
        0xb4 => or8(cpu, cpu.hl.high()),
        0xb5 => or8(cpu, cpu.hl.low()),
        0xb6 => or8(cpu, cpu.read_byte(cpu.hl.value())),
        0xf6 => {
            let value = cpu.pop_pc();
            or8(cpu, value)
        }

        // 7. XOR n
        0xaf => xor8(cpu, cpu.af.high()),
        0xa8 => xor8(cpu, cpu.bc.high()),
        0xa9 => xor8(cpu, cpu.bc.low()),
        0xaa => xor8(cpu, cpu.de.high()),
        0xab => xor8(cpu, cpu.de.low()),
        0xac => xor8(cpu, cpu.hl.high()),
        0xad => xor8(cpu, cpu.hl.low()),
        0xae => xor8(cpu, cpu.read_byte(cpu.hl.value())),
        0xee => {
            let value = cpu.pop_pc();
            xor8(cpu, value)
        }

        // 9. INC n
        0x3c => {
            let value = inc(cpu, cpu.af.high());
            cpu.af.set_high(value);
        }
        0x04 => {
            let value = inc(cpu, cpu.bc.high());
            cpu.bc.set_high(value);
        }
        0x0c => {
            let value = inc(cpu, cpu.bc.low());
            cpu.bc.set_low(value);
        }
        0x14 => {
            let value = inc(cpu, cpu.de.high());
            cpu.de.set_high(value);
        }
        0x1c => {
            let value = inc(cpu, cpu.de.low());
            cpu.de.set_low(value);
        }
        0x24 => {
            let value = inc(cpu, cpu.hl.high());
            cpu.hl.set_high(value);
        }
        0x2c => {
            let value = inc(cpu, cpu.hl.low());
            cpu.hl.set_low(value);
        }
        0x34 => {
            let value = inc(cpu, cpu.read_byte(cpu.hl.value()));
            cpu.write_byte(cpu.hl.value(), value);
        }

        // 10. DEC n
        0x3d => {
            let value = dec(cpu, cpu.af.high());
            cpu.af.set_high(value);
        }
        0x05 => {
            let value = dec(cpu, cpu.bc.high());
            cpu.bc.set_high(value);
        }
        0x0d => {
            let value = dec(cpu, cpu.bc.low());
            cpu.bc.set_low(value);
        }
        0x15 => {
            let value = dec(cpu, cpu.de.high());
            cpu.de.set_high(value);
        }
        0x1d => {
            let value = dec(cpu, cpu.de.low());
            cpu.de.set_low(value);
        }
        0x25 => {
            let value = dec(cpu, cpu.hl.high());
            cpu.hl.set_high(value);
        }
        0x2d => {
            let value = dec(cpu, cpu.hl.low());
            cpu.hl.set_low(value);
        }
        0x35 => {
            let value = dec(cpu, cpu.read_byte(cpu.hl.value()));
            cpu.write_byte(cpu.hl.value(), value);
        }

        // 3.3.4. 16-Bit Arithmetic
        // 1. ADD HL,n
        0x09 => {
            let value = add16(cpu, cpu.hl.value(), cpu.bc.value());
            cpu.hl.set(value);
        }
        0x19 => {
            let value = add16(cpu, cpu.hl.value(), cpu.de.value());
            cpu.hl.set(value);
        }
        0x29 => {
            let value = add16(cpu, cpu.hl.value(), cpu.hl.value());
            cpu.hl.set(value);
        }
        0x39 => {
            let value = add16(cpu, cpu.hl.value(), cpu.sp);
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
        0x03 => cpu.bc.set(cpu.bc.value.wrapping_add(1)),
        0x13 => cpu.de.set(cpu.de.value.wrapping_add(1)),
        0x23 => cpu.hl.set(cpu.hl.value.wrapping_add(1)),
        0x33 => cpu.sp = cpu.sp.wrapping_add(1),

        // 4. DEC nn
        0x0b => cpu.bc.set(cpu.bc.value.wrapping_sub(1)),
        0x1b => cpu.de.set(cpu.de.value.wrapping_sub(1)),
        0x2b => cpu.hl.set(cpu.hl.value.wrapping_sub(1)),
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
        0xf9 => cpu.sp = cpu.get_hl(),

        // 4. LDHL SP,n
        0xf8 => {
            let sp = cpu.sp;
            let n = cpu.pop_pc() as i8;
            let address = cpu.sp.wrapping_add(n as u16);
            cpu.set_hl(address);

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
        0xc3 => jp(cpu, true),
        //2. JP cc,nn
        0xc2 => jp(cpu, !cpu.get_z_flag()),
        0xca => jp(cpu, cpu.get_z_flag()),
        0xd2 => jp(cpu, !cpu.get_c_flag()),
        0xda => jp(cpu, cpu.get_c_flag()),

        // 3. JP (HL)
        0xe9 => cpu.pc = cpu.hl.value(),

        // 4. JR n
        0x18 => jr(cpu, true),
        // 5. JR cc,n
        0x20 => jr(cpu, !cpu.get_z_flag()),
        0x28 => jr(cpu, cpu.get_z_flag()),
        0x30 => jr(cpu, !cpu.get_c_flag()),
        0x38 => jr(cpu, cpu.get_c_flag()),

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
                value = value | 0x01
            } else {
                value = value | 0x00
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
                value = value | 0x80
            } else {
                value = value | 0x00
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
                    let value = rlc(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x00 => {
                    let value = rlc(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x01 => {
                    let value = rlc(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x02 => {
                    let value = rlc(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x03 => {
                    let value = rlc(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x04 => {
                    let value = rlc(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x05 => {
                    let value = rlc(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x06 => {
                    let value = rlc(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // RRC
                0x0f => {
                    let value = rrc(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x08 => {
                    let value = rrc(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x09 => {
                    let value = rrc(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x0a => {
                    let value = rrc(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x0b => {
                    let value = rrc(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x0c => {
                    let value = rrc(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x0d => {
                    let value = rrc(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x0e => {
                    let value = rrc(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // RL
                0x17 => {
                    let value = rl(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x10 => {
                    let value = rl(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x11 => {
                    let value = rl(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x12 => {
                    let value = rl(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x13 => {
                    let value = rl(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x14 => {
                    let value = rl(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x15 => {
                    let value = rl(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x16 => {
                    let value = rl(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 8. RR n
                0x1f => {
                    let value = rr(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x18 => {
                    let value = rr(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x19 => {
                    let value = rr(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x1a => {
                    let value = rr(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x1b => {
                    let value = rr(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x1c => {
                    let value = rr(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x1d => {
                    let value = rr(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x1e => {
                    let value = rr(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 9. SLA n
                0x27 => {
                    let value = sla(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x20 => {
                    let value = sla(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x21 => {
                    let value = sla(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x22 => {
                    let value = sla(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x23 => {
                    let value = sla(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x24 => {
                    let value = sla(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x25 => {
                    let value = sla(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x26 => {
                    let value = sla(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 10. SRA n
                0x2f => {
                    let value = sra(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x28 => {
                    let value = sra(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x29 => {
                    let value = sra(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x2a => {
                    let value = sra(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x2b => {
                    let value = sra(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x2c => {
                    let value = sra(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x2d => {
                    let value = sra(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x2e => {
                    let value = sra(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 1. SWAP n
                0x37 => {
                    let value = swap(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x30 => {
                    let value = swap(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x31 => {
                    let value = swap(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x32 => {
                    let value = swap(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x33 => {
                    let value = swap(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x34 => {
                    let value = swap(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x35 => {
                    let value = swap(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x36 => {
                    let value = swap(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // 11. SRL n
                0x3f => {
                    let value = srl(cpu, cpu.af.high());
                    cpu.af.set_high(value);
                }
                0x38 => {
                    let value = srl(cpu, cpu.bc.high());
                    cpu.bc.set_high(value);
                }
                0x39 => {
                    let value = srl(cpu, cpu.bc.low());
                    cpu.bc.set_low(value);
                }
                0x3a => {
                    let value = srl(cpu, cpu.de.high());
                    cpu.de.set_high(value);
                }
                0x3b => {
                    let value = srl(cpu, cpu.de.low());
                    cpu.de.set_low(value);
                }
                0x3c => {
                    let value = srl(cpu, cpu.hl.high());
                    cpu.hl.set_high(value);
                }
                0x3d => {
                    let value = srl(cpu, cpu.hl.low());
                    cpu.hl.set_low(value);
                }
                0x3e => {
                    let value = srl(cpu, cpu.read_byte(cpu.hl.value()));
                    cpu.write_byte(cpu.hl.value(), value);
                }

                // BIT
                0x40 => bit(cpu, 0, cpu.bc.high()),
                0x41 => bit(cpu, 0, cpu.bc.low()),
                0x42 => bit(cpu, 0, cpu.de.high()),
                0x43 => bit(cpu, 0, cpu.de.low()),
                0x44 => bit(cpu, 0, cpu.hl.high()),
                0x45 => bit(cpu, 0, cpu.hl.low()),
                0x46 => bit(cpu, 0, cpu.read_byte(cpu.hl.value())),
                0x47 => bit(cpu, 0, cpu.af.high()),

                0x48 => bit(cpu, 1, cpu.bc.high()),
                0x49 => bit(cpu, 1, cpu.bc.low()),
                0x4a => bit(cpu, 1, cpu.de.high()),
                0x4b => bit(cpu, 1, cpu.de.low()),
                0x4c => bit(cpu, 1, cpu.hl.high()),
                0x4d => bit(cpu, 1, cpu.hl.low()),
                0x4e => bit(cpu, 1, cpu.read_byte(cpu.hl.value())),
                0x4f => bit(cpu, 1, cpu.af.high()),

                0x50 => bit(cpu, 2, cpu.bc.high()),
                0x51 => bit(cpu, 2, cpu.bc.low()),
                0x52 => bit(cpu, 2, cpu.de.high()),
                0x53 => bit(cpu, 2, cpu.de.low()),
                0x54 => bit(cpu, 2, cpu.hl.high()),
                0x55 => bit(cpu, 2, cpu.hl.low()),
                0x56 => bit(cpu, 2, cpu.read_byte(cpu.hl.value())),
                0x57 => bit(cpu, 2, cpu.af.high()),

                0x58 => bit(cpu, 3, cpu.bc.high()),
                0x59 => bit(cpu, 3, cpu.bc.low()),
                0x5a => bit(cpu, 3, cpu.de.high()),
                0x5b => bit(cpu, 3, cpu.de.low()),
                0x5c => bit(cpu, 3, cpu.hl.high()),
                0x5d => bit(cpu, 3, cpu.hl.low()),
                0x5e => bit(cpu, 3, cpu.read_byte(cpu.hl.value())),
                0x5f => bit(cpu, 3, cpu.af.high()),

                0x60 => bit(cpu, 4, cpu.bc.high()),
                0x61 => bit(cpu, 4, cpu.bc.low()),
                0x62 => bit(cpu, 4, cpu.de.high()),
                0x63 => bit(cpu, 4, cpu.de.low()),
                0x64 => bit(cpu, 4, cpu.hl.high()),
                0x65 => bit(cpu, 4, cpu.hl.low()),
                0x66 => bit(cpu, 4, cpu.read_byte(cpu.hl.value())),
                0x67 => bit(cpu, 4, cpu.af.high()),

                0x68 => bit(cpu, 5, cpu.bc.high()),
                0x69 => bit(cpu, 5, cpu.bc.low()),
                0x6a => bit(cpu, 5, cpu.de.high()),
                0x6b => bit(cpu, 5, cpu.de.low()),
                0x6c => bit(cpu, 5, cpu.hl.high()),
                0x6d => bit(cpu, 5, cpu.hl.low()),
                0x6e => bit(cpu, 5, cpu.read_byte(cpu.hl.value())),
                0x6f => bit(cpu, 5, cpu.af.high()),

                0x70 => bit(cpu, 6, cpu.bc.high()),
                0x71 => bit(cpu, 6, cpu.bc.low()),
                0x72 => bit(cpu, 6, cpu.de.high()),
                0x73 => bit(cpu, 6, cpu.de.low()),
                0x74 => bit(cpu, 6, cpu.hl.high()),
                0x75 => bit(cpu, 6, cpu.hl.low()),
                0x76 => bit(cpu, 6, cpu.read_byte(cpu.hl.value())),
                0x77 => bit(cpu, 6, cpu.af.high()),

                0x78 => bit(cpu, 7, cpu.bc.high()),
                0x79 => bit(cpu, 7, cpu.bc.low()),
                0x7a => bit(cpu, 7, cpu.de.high()),
                0x7b => bit(cpu, 7, cpu.de.low()),
                0x7c => bit(cpu, 7, cpu.hl.high()),
                0x7d => bit(cpu, 7, cpu.hl.low()),
                0x7e => bit(cpu, 7, cpu.read_byte(cpu.hl.value())),
                0x7f => bit(cpu, 7, cpu.af.high()),

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
        0xcd => call(cpu),

        // 2. CALL cc,nn
        0xc4 => call_if(cpu, !cpu.get_z_flag()),
        0xcc => call_if(cpu, cpu.get_z_flag()),
        0xd4 => call_if(cpu, !cpu.get_c_flag()),
        0xdc => call_if(cpu, cpu.get_c_flag()),

        // 1. RST n
        0xc7 | 0xcf | 0xd7 | 0xdf | 0xe7 | 0xef | 0xf7 | 0xff => {
            cpu.sp = cpu.sp.wrapping_sub(2);
            cpu.write_byte16(cpu.sp, cpu.pc);

            cpu.pc = 0x0000 + opecode as u16 - 0x00c7;
        }

        // 1. RET
        0xc9 => ret(cpu),

        // 2. RET cc
        0xc0 => {
            if !cpu.get_z_flag() {
                ret(cpu);
            }
        }
        0xc8 => {
            if cpu.get_z_flag() {
                ret(cpu);
            }
        }
        0xd0 => {
            if !cpu.get_c_flag() {
                ret(cpu);
            }
        }
        0xd8 => {
            if cpu.get_c_flag() {
                ret(cpu);
            }
        }

        // 3. RETI
        0xd9 => {
            ret(cpu);
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
            opecode, cpu.pc
        ),
    }
}
