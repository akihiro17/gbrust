mod cpu;
mod mmu;
mod ppu;

use std::thread;
use std::time;

extern crate minifb;
use minifb::{Window, WindowOptions};

fn main() {
    println!("Hello, world!");

    let mut window = Window::new(
        "Test - ESC to exit",
        ppu::WIDTH,
        ppu::HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/bg_scroll_x_y.gb");

    loop {
        let now = time::Instant::now();

        let mut elapsed_tick: u32 = 0;
        while elapsed_tick < 456 * 154 {
            elapsed_tick += cpu.step() as u32;
        }

        let mut elapsed = now.elapsed();
        println!("after cpu step() elapsed {:?}", elapsed);

        let buffer = &cpu.mmu.ppu.buffer;
        elapsed = now.elapsed();
        println!("after get buffer elapsed {:?}", elapsed);

        if elapsed_tick >= 456 * 154 {
            elapsed_tick = 0;
            window
                .update_with_buffer(buffer, ppu::WIDTH, ppu::HEIGHT)
                .unwrap();
        }

        let wait = time::Duration::from_micros(1000000 / 60);
        elapsed = now.elapsed();

        println!("after update_with_buffer elapsed {:?}", elapsed);

        if wait > elapsed {
            println!("{:?} sleep", wait - elapsed);
            thread::sleep(wait - elapsed);
        }
    }
}
