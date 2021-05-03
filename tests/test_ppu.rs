use gbrust::cpu;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[test]
fn test_ppu() {
    struct Test<'a> {
        rom_name: &'a str,
        frames: u64,
        expected: u64,
    };

    let tests: [Test; 3] = [
        Test {
            rom_name: "roms/picture.gb",
            frames: 3,
            expected: 0xE604AB94ADD6271D,
        },
        Test {
            rom_name: "roms/window.gb",
            frames: 3,
            expected: 0x185A16599A703C94,
        },
        Test {
            rom_name: "roms/sprite.gb",
            frames: 3,
            expected: 0x86604EEABD2DEEBB,
        },
    ];

    for t in tests.iter() {
        println!("{}", t.rom_name);
        let mut cpu = cpu::CPU::new(t.rom_name);

        let steps: u64 = 456 * 154 * t.frames;
        for _ in 1..=steps {
            cpu.step();
        }

        let mut hasher = DefaultHasher::new();
        cpu.mmu.ppu.buffer.hash(&mut hasher);
        assert_eq!(hasher.finish(), t.expected);
    }
}
