pub struct Register {
    // The value of the register
    value: u16,

    mask: u16,
}

impl Register {
    pub fn new(value: u16, mask: u16) -> Self {
        Register { value, mask }
    }

    pub fn high(&self) -> u8 {
        (self.value >> 8) as u8
    }

    pub fn low(&self) -> u8 {
        (self.value & 0x00ff) as u8
    }

    pub fn value(&self) -> u16 {
        self.value
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
