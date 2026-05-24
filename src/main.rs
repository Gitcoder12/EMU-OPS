use minifb::{Window, WindowOptions, Key};
use std::fs;
use std::time::{Instant, Duration};
use chip8_core::{Cpu, Memory, Graphics};

fn main() {
    let rom_path = "./roms/";
    let rom_file = format!("{}pong.ch8", rom_path);
    
    let rom_data = match fs::read(&rom_file) {
        Ok(data) => data,
        Err(_) => {
            println!("No ROM found. Creating dummy roms/pong.ch8 – please add a real Chip-8 ROM");
            fs::create_dir_all(rom_path).unwrap();
            fs::write(&rom_file, &[0; 4096]).unwrap();
            vec![0; 4096]
        }
    };

    let mut cpu = Cpu::new();
    let mut memory = Memory::new();
    let mut graphics = Graphics::new();
    memory.load_rom(&rom_data);

    let mut window = Window::new(
        "EMU-OPS – Chip-8",
        640,
        320,
        WindowOptions::default(),
    ).unwrap();
    window.limit_update_rate(Some(Duration::from_micros(16666)));

    let frame_duration = Duration::from_nanos(1_000_000_000 / 60);
    let mut last_frame = Instant::now();

    // Keypad mapping (Chip-8 hex keys 0x0..0xF)
    let mut keypad = [false; 16];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Update keypad state from keyboard
        // Map PC keyboard to Chip-8 keys (typical layout)
        let key_map = [
            (Key::X, 0x0), (Key::Key1, 0x1), (Key::Key2, 0x2), (Key::Key3, 0x3),
            (Key::Q, 0x4), (Key::W, 0x5), (Key::E, 0x6), (Key::A, 0x7),
            (Key::S, 0x8), (Key::D, 0x9), (Key::Z, 0xA), (Key::C, 0xB),
            (Key::Key4, 0xC), (Key::R, 0xD), (Key::F, 0xE), (Key::V, 0xF),
        ];
        for (key, chip_key) in key_map.iter() {
            keypad[*chip_key] = window.is_key_down(*key);
        }

        // Emulate 10 instructions per frame
        for _ in 0..10 {
            cpu.fetch(&memory.ram);
            cpu.execute(&mut memory.ram, &mut graphics.display, &keypad);
            cpu.tick();
        }

        if graphics.needs_render {
            let mut buffer = vec![0; 640 * 320];
            for y in 0..32 {
                for x in 0..64 {
                    let color = if graphics.display[y][x] { 0xFFFFFF } else { 0x000000 };
                    for sy in 0..10 {
                        for sx in 0..10 {
                            let idx = ((y * 10 + sy) * 640) + (x * 10 + sx);
                            buffer[idx] = color;
                        }
                    }
                }
            }
            window.update_with_buffer(&buffer, 640, 320).unwrap();
            graphics.needs_render = false;
        }

        let now = Instant::now();
        if now.duration_since(last_frame) < frame_duration {
            std::thread::sleep(frame_duration - now.duration_since(last_frame));
        }
        last_frame = Instant::now();
    }
}
