# EMU-OPS

**An experimental CHIP-8 emulator prototype with a Rust CPU, memory, graphics core, and desktop window.**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)

## Current Implementation

The runnable application currently includes:

- A CHIP-8 core with CPU registers, stack, timers, memory, and opcode execution.
- 4 KB memory with the standard CHIP-8 font set loaded at startup.
- A 64 × 32 monochrome display buffer.
- Keyboard input mapping for the CHIP-8 hexadecimal keypad.
- A `minifb` desktop window for displaying the emulator output.

## Quick Start

Install the stable Rust toolchain, then clone and build the repository:

```bash
git clone https://github.com/Gitcoder12/EMU-OPS.git
cd EMU-OPS
cargo check --workspace
cargo run --release
```

A graphical desktop session is required by the `minifb` window.

## ROM Handling

The emulator looks for a CHIP-8 ROM at:

```text
roms/pong.ch8
```

No copyrighted ROMs are included. If the file is missing, the current launcher creates a zero-filled placeholder file so the application can start; that placeholder is not a playable game. To run an actual program, place a CHIP-8 ROM that you are legally entitled to use at the expected path.

## Keyboard Mapping

| CHIP-8 key | PC key |
|---|---|
| 0 | X |
| 1–3 | 1–3 |
| 4–6 | Q, W, E |
| 7–9 | A, S, D |
| A–B | Z, C |
| C–F | 4, R, F, V |

Press **Esc** to close the window.

## Project Structure

```text
EMU-OPS/
├── Cargo.toml
├── src/
│   ├── main.rs             # Desktop emulator loop and keyboard mapping
│   └── features.rs         # Experimental recovery-feature draft (not wired into runtime)
└── cores/
    └── chip8/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── cpu.rs
            ├── memory.rs
            ├── graphics.rs
            └── features.rs # Experimental module draft; not exported by the core
```

## Experimental Work

The repository also contains draft save-state, rollback, and runtime-reliability code. Those modules are not currently exported by the CHIP-8 core or connected to the running emulator loop. The current build does **not** provide AI tuning, automated crash recovery, persistent game profiles, or a telemetry dashboard.

## Current Limitations

- CHIP-8 compatibility is incomplete; this is a development prototype, not a validated emulator.
- No game ROM is bundled.
- The placeholder ROM created by the launcher contains only zero bytes.
- Save-state and self-recovery experiments are not integrated into the executable.
- The desktop frontend requires a graphical session.

## License

MIT. See [LICENSE](LICENSE).

---

## Project handbook

- [Activity log](ACTIVITY_LOG.md) · [Feature upgrade plan](FEATURE_UPGRADE_PLAN.md)
- [Company and team fit](COMPANY_FIT.md) · [README explanation draft](README_ADDENDUM_DRAFT.md)
- [README quality checklist](README_QUALITY_CHECKLIST.md)
- [Contributing](CONTRIBUTING.md) · [Agent instructions](AGENTS.md) · [Agent skills](SKILLS.md)

## Understanding the project

### In plain language

A CHIP-8 emulator: a small virtual computer that loads a game program and runs its instructions on a simple screen and keypad.

### Technical and research view

The emulator loads a CHIP-8 program (ROM), interprets its instructions, updates virtual memory/registers/timers and draws pixels into a small window. The keypad maps physical keys to the virtual CHIP-8 keys.

For future work, see [FEATURE_UPGRADE_PLAN.md](FEATURE_UPGRADE_PLAN.md), [ACTIVITY_LOG.md](ACTIVITY_LOG.md), [COMPANY_FIT.md](COMPANY_FIT.md) and [CONTRIBUTING.md](CONTRIBUTING.md).
