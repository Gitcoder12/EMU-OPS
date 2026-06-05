# Contributing to EMU-OPS
- Core emulator logic is in `cores/chip8/src/`
- Adding a new core: create `cores/your_system/` mirroring chip8 structure
- Run checks: `cargo check && cargo test`
- Self-healing AI hooks go in `src/features.rs`
