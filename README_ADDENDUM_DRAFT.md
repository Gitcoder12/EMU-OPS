# EMU-OPS — README Explanation Draft

## In plain language

A CHIP-8 emulator: a small virtual computer that loads a game program and runs its instructions on a simple screen and keypad.

## What happens when it runs

The emulator loads a CHIP-8 program (ROM), interprets its instructions, updates virtual memory/registers/timers and draws pixels into a small window. The keypad maps physical keys to the virtual CHIP-8 keys.

## Technical explanation

Rust-based experimental emulator with CPU registers, stack, timers, memory, CHIP-8 font, 64×32 display buffer and a minifb desktop window. ROMs must be supplied by the user; a missing ROM should produce an explicit error instead of generating fake content. Emulator correctness depends on opcode semantics and timing behavior.

Document opcode dispatch, memory layout, font placement, stack behavior, timers, input mapping and rendering loop. Include the exact tested Rust toolchain and build commands.

## Limitations

Different CHIP-8 variants can interpret some behaviors differently. State which variant rules are implemented and which test ROMs validate them.

## Contributing

See FEATURE_UPGRADE_PLAN.md, CONTRIBUTING.md and ACTIVITY_LOG.md.
