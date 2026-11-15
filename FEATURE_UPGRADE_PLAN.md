# EMU-OPS — Feature Upgrade Plan

## Objective

A CHIP-8 emulator: a small virtual computer that loads a game program and runs its instructions on a simple screen and keypad.

## P0 — emulator correctness

- [ ] Build a deterministic opcode test suite covering registers, stack, memory boundaries, timers and invalid instructions.
- [ ] Ensure absent or malformed ROMs fail clearly without creating placeholder files.

## P1 — debugging and usability

- [ ] Document keyboard mapping and emulator timing behavior.
- [ ] Add configurable instruction/timer stepping for debugging and reproducible tests.

## P2 — compatibility and performance

- [ ] Compare opcode behavior against a recognised CHIP-8 test ROM suite with provenance/licensing.
- [ ] Profile rendering and CPU loop without changing semantics.

## Collaborator work packages

1. Opcode semantics and deterministic unit tests.
2. Memory/register/stack boundary handling.
3. Timing/debug instrumentation.
4. Compatibility test ROM evaluation with documented provenance.

## Definition of done

- [ ] cargo test covers opcode behavior and invalid inputs.
- [ ] Missing ROM does not create synthetic game content.
- [ ] README states supported platform and controls.
- [ ] Test ROMs and data are legally distributable.
