mod catridge;
mod cpu;
mod mmu;
mod ppu;
mod timer;

use std::thread;
use std::time;

extern crate minifb;
use minifb::{Window, WindowOptions};

fn main() {
    let mut window = Window::new("Test - ESC to exit", 320, 288, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    thread::sleep(time::Duration::from_secs(10));

    // let rom_name = "roms/hello-2.gb";
    let rom_name = "roms/bg_scroll_x_y.gb";
    let rom_name = "roms/cpu_instrs.gb";
    let rom_name = "roms/Tetris.gb";
    // let rom_name = "roms/HungryBirds.gb";
    // Limit to max ~60 fps update rate
    // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/Tetris.gb");

    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/cpu_instrs.gb");
    // let mut cpu = cpu::CPU::new("", "roms/cpu_instrs.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/02.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/01.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/11.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/10.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/03.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/09.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/07.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/05.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/04.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/ld.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/08.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/sprite.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/window.gb");
    // let mut cpu = cpu::CPU::new("roms/DMG_ROM.bin", "roms/bg_scroll_x_y.gb");
    let mut cpu = cpu::CPU::new_with_boot_rom("roms/DMG_ROM.bin", rom_name);

    loop {
        let now = time::Instant::now();

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
