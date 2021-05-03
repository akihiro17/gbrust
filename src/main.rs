mod catridge;
mod cpu;
mod mmu;
mod ppu;
mod timer;

use std::env;
use std::thread;
use std::time;

extern crate minifb;
use minifb::{Window, WindowOptions};

extern crate getopts;
use getopts::Options;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("b", "bootrom-file", "set the bootrom file path", "");
    opts.reqopt("f", "rom file", "set the rom file apth", "");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    let rom_file = matches.opt_str("f").unwrap();

    let mut cpu = match matches.opt_present("b") {
        true => {
            let bootrom_file = matches.opt_str("b").unwrap();
            cpu::Cpu::new_with_boot_rom(&bootrom_file, &rom_file)
        }
        false => cpu::Cpu::new(&rom_file),
    };

    let mut window = Window::new("GameBoy Emulator", 320, 288, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // thread::sleep(time::Duration::from_secs(10));

    loop {
        let now = time::Instant::now();

        // 1 frame

        // https://mgba-emu.github.io/gbdoc/
        // > One frame takes 70224 cycles
        // Timing is divided into 154 lines, ~~ Each line takes 456 cycles.
        let mut elapsed_tick: u32 = 0;
        while elapsed_tick < 456 * 154 {
            elapsed_tick += cpu.step() as u32;
        }

        let buffer = &cpu.mmu.ppu.buffer;

        window
            .update_with_buffer(buffer, ppu::WIDTH, ppu::HEIGHT)
            .unwrap();

        let wait = time::Duration::from_micros(1000000 / 60);
        let elapsed = now.elapsed();

        if wait > elapsed {
            thread::sleep(wait - elapsed);
        }
    }
}
