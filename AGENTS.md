# Agent Instructions — EMU-OPS

Read README.md, CONTRIBUTING.md and FEATURE_UPGRADE_PLAN.md before editing.

- Preserve emulator semantics and document variant/quirk choices.
- Never fabricate ROM files or include copyrighted ROMs without permission.
- Add deterministic tests for every opcode or timing behavior changed.
- Check bounds and invalid instruction behavior explicitly.
- Run cargo fmt, clippy and cargo test when configured; report exact results.
- Work on a branch and open a PR; do not merge or release autonomously.
