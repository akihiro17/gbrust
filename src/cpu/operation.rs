use crate::cpu::Cpu;

// do operations
// update flags
pub fn inc(cpu: &mut Cpu, original: u8) -> u8 {
    let new_value = original.wrapping_add(1);

    //update the flags
    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    // Set if carry from bit 3
    cpu.set_h_flag_if(original & 0x0f == 0x0f);

    new_value
}

pub fn dec(cpu: &mut Cpu, original: u8) -> u8 {
    let new_value = original.wrapping_sub(1);

    //update the flags
    cpu.set_z_flag_if(new_value == 0);
    cpu.set_n_flag();
    // Set if carry from bit 4.
    cpu.set_h_flag_if(original & 0x0f == 0);

    new_value
}

pub fn add8(cpu: &mut Cpu, value: u8) {
    let original = cpu.af.high();

    let (new_value, carry) = original.overflowing_add(value);
    cpu.af.set_high(new_value);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag_if((original & 0x0f) + (value & 0x0f) > 0x0f);
    cpu.set_c_flag_if(carry);
}

pub fn adc(cpu: &mut Cpu, op2: u8) {
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

pub fn sub(cpu: &mut Cpu, op2: u8) {
    let op1 = cpu.af.high();

    let (value, carry) = op1.overflowing_sub(op2);
    cpu.af.set_high(value);

    cpu.set_z_flag_if(value == 0);
    cpu.set_n_flag();
    cpu.set_h_flag_if(op1 & 0x0f < op2 & 0x0f);
    cpu.set_c_flag_if(carry);
}

pub fn sbc(cpu: &mut Cpu, op2: u8) {
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

pub fn cp(cpu: &mut Cpu, op1: u8) {
    let a = cpu.af.high();

    cpu.set_z_flag_if(a == op1);
    cpu.set_n_flag();
    cpu.set_h_flag_if((a & 0x0f) < (op1 & 0x0f));
    cpu.set_c_flag_if(a < op1);
}

pub fn or(cpu: &mut Cpu, op1: u8) {
    cpu.af.set_high(cpu.af.high() | op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();
}

pub fn and(cpu: &mut Cpu, op1: u8) {
    cpu.af.set_high(cpu.af.high() & op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag();
    cpu.reset_c_flag();
}

pub fn xor(cpu: &mut Cpu, op1: u8) {
    cpu.af.set_high(cpu.af.high() ^ op1);

    cpu.set_z_flag_if(cpu.af.high() == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();
}

pub fn add16(cpu: &mut Cpu, op1: u16, op2: u16) -> u16 {
    let new_value = op1.wrapping_add(op2);

    cpu.reset_n_flag();
    let half_carry = (op1 & 0x0fff) + (op2 & 0x0fff) > 0x0fff;
    cpu.set_h_flag_if(half_carry);
    let carry = (op1 as u32) + (op2 as u32) > 0xffff;
    cpu.set_c_flag_if(carry);

    new_value
}

pub fn jp(cpu: &mut Cpu, condition: bool) {
    let address = cpu.pop_pc16();
    if condition {
        cpu.pc = address;
    }
}

pub fn jr(cpu: &mut Cpu, condition: bool) {
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

pub fn rlc(cpu: &mut Cpu, op1: u8) -> u8 {
    // Rotate n left. Old bit 7 to Carry flag.
    let new_value = op1.rotate_left(1);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    new_value
}

pub fn rrc(cpu: &mut Cpu, op1: u8) -> u8 {
    // Rotate n right. Old bit 0 to Carry flag.
    let new_value = op1.rotate_right(1);

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    new_value
}

pub fn rr(cpu: &mut Cpu, op1: u8) -> u8 {
    let mut new_value = op1 >> 1;
    if cpu.get_c_flag() {
        new_value |= 0x80;
    } else {
        new_value |= 0x00;
    }

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    new_value
}

pub fn rl(cpu: &mut Cpu, op1: u8) -> u8 {
    // rorate left through Carry flag
    // ref. https://ja.wikipedia.org/wiki/%E3%83%93%E3%83%83%E3%83%88%E6%BC%94%E7%AE%97#/media/%E3%83%95%E3%82%A1%E3%82%A4%E3%83%AB:Rotate_left_through_carry.svg
    let mut new_value = op1 << 1;
    if cpu.get_c_flag() {
        new_value |= 0x01;
    } else {
        new_value |= 0x00;
    }

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    new_value
}

pub fn sla(cpu: &mut Cpu, op1: u8) -> u8 {
    let new_value = op1 << 1;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0b1000_0000) == 0b1000_0000);

    new_value
}

pub fn sra(cpu: &mut Cpu, op1: u8) -> u8 {
    // Shift n right into Carry. MSB doesn't change.
    let mut new_value = op1 >> 1;
    new_value |= op1 & 0x80;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    new_value
}

pub fn srl(cpu: &mut Cpu, op1: u8) -> u8 {
    // Shift n right into Carry. MSB set to 0.
    let new_value = op1 >> 1;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.set_c_flag_if((op1 & 0x01) > 0);

    new_value
}

pub fn swap(cpu: &mut Cpu, op1: u8) -> u8 {
    let new_value = (op1 & 0x0f) << 4 | (op1 & 0xf0) >> 4;

    cpu.set_z_flag_if(new_value == 0);
    cpu.reset_n_flag();
    cpu.reset_h_flag();
    cpu.reset_c_flag();

    new_value
}

pub fn bit(cpu: &mut Cpu, n: u8, value: u8) {
    let mask = 0x0001 << n;
    let bit_test = value & mask;

    cpu.set_z_flag_if(bit_test == 0);
    cpu.reset_n_flag();
    cpu.set_h_flag();
}

pub fn ret(cpu: &mut Cpu) {
    cpu.pc = cpu.read_byte16(cpu.sp);
    cpu.sp = cpu.sp.wrapping_add(2);
}

pub fn call(cpu: &mut Cpu) {
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

pub fn call_if(cpu: &mut Cpu, condition: bool) {
    if condition {
        call(cpu);
    } else {
        cpu.pc = cpu.pc.wrapping_add(2);
    }
}
