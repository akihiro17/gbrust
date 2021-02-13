// ref. https://hacktix.github.io/GBEDG/timers/#timer-operation

pub struct Timer {
    // $FF04 - Divider Register (DIV)
    div: u16,
    // $FF05 - Timer Counter (TIMA)
    tima: u8,
    // $FF06 - Timer Modulo (TMA)
    tma: u8,
    // $FF07 - Timer Control (TAC)
    // Bit 2 : Timer Enable
    // Bits 1-0 : Clock Select
    tac: u8,
    pub irq: bool,
}

impl Timer {
    pub fn new() -> Timer {
        return Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            irq: false,
        };
    }

    pub fn write_byte(&mut self, address: u16, val: u8) {
        match address {
            0xff04 => {
                // writing to $FF04 resets the whole internal 16-bit DIV counter to 0 instantly
                self.div = 0;
            }
            0xff05 => self.tima = val,
            0xff06 => self.tma = val,
            0xff07 => self.tac = val,
            _ => panic!("unexpected address #{:X}", address),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            // only the upper 8 bits are mapped to memory
            0xff04 => (self.div >> 8) as u8,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => panic!("unexpected address #{:X}", address),
        }
    }

    pub fn step(&mut self, tick: usize) {
        // ref. https://hacktix.github.io/GBEDG/timers/#timer-operation
        let previous = self.div;
        self.div = self.div.wrapping_add(tick as u16);

        // println!("tick: {}", tick);

        // Timer is enabled or not.
        if (self.tac & (0x01 << 2)) > 0 {
            let divider = match self.tac & 0x03 {
                0x00 => 10, // 2^10 = 1024,
                0x01 => 4,
                0x10 => 6,
                0x11 => 8,
                _ => panic!("invalid devider {:X}", self.tac),
            };

            // 0 -> 1
            let t1 = self.div >> divider as u16;
            let t2 = previous >> divider as u16;
            let diff = t1.wrapping_sub(t2);

            if diff > 0 {
                let (value, overflow) = self.tima.overflowing_add(diff as u8);

                if overflow {
                    // when the TIMA register overflows (being incremented when the value is 0xFF),
                    // it is “reloaded” with the value of the TMA register at $FF06
                    // self.tima = self.tma;
                    self.tima = self.tma + (diff as u8 - 1);
                    self.irq = true;
                } else {
                    self.tima = value;
                }
            }
        }
    }
}
