use minifb::{Window, WindowOptions, Key};
use std::fs;
use std::time::{Instant, Duration};
use chip8_core::{Cpu, Memory, Graphics};

fn load_rom(path: &str) -> Result<Vec<u8>, std::io::Error> {
    fs::read(path)
}

fn main() {
    let rom_file = "./roms/pong.ch8";
    let rom_data = match load_rom(rom_file) {
        Ok(data) => data,
        Err(error) => {
            eprintln!(
                "Unable to load ROM at {rom_file}: {error}. Add a CHIP-8 ROM you are legally entitled to use."
            );
            return;
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
            if cpu.execute(&mut memory.ram, &mut graphics.display, &keypad) {
                graphics.needs_render = true;
            }
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


#[cfg(test)]
mod tests {
    use super::load_rom;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn missing_rom_returns_an_error_without_creating_a_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("emu-ops-missing-{nonce}.ch8"));

        assert!(load_rom(path.to_str().expect("temporary path should be UTF-8")).is_err());
        assert!(!path.exists());
    }
}
