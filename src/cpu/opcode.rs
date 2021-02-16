use crate::cpu::clock;

pub struct Opcode {
    code: u8,
}

impl Opcode {
    pub fn new(code: u8) -> Self {
        Opcode { code: code }
    }

    pub fn code(&self) -> u8 {
        self.code
    }

    pub fn clock(&self) -> u8 {
        clock::clock(self.code())
    }
}
