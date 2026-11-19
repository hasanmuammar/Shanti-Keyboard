# shanti-keysyms

Single source of truth for the key event **keysym → riti virtual-key**
mapping used by the IBus engine. Replaces the 112-entry table that used to
live in the engine's `keycode.h` header.

## Layout

- `data/keysyms.tsv` — committed source of truth: `keysym_value`, `keysym_name`, `VC_*` name.
- `src/data.rs` — generated from the TSV, sorted array of `(keysym_value, VC_* name)`.
- `src/lib.rs` — Rust lookup (`pub fn lookup`) + C ABI `shanti_keysyms_lookup`
  (`include/shanti-keysyms.h`), the C++ engine links this; unknown keysyms map
  to `VC_UNKNOWN` (0x0046, the engine's historical fallback).
- `generator/keysyms_extract.py` — regeneration script (needs IBus dev headers
  to probe real enum values and verify that keysym values are unique).

## Why names, not numbers

The table stores riti virtual-key *names*; numeric values are resolved at
build time from the actual riti submodule header (`include_str!` of
`src/engine/riti/include/riti.h`), so riti bumps never require a data
regeneration for virtual-key renumbering — `cargo test` fails if a name is
not defined or the TSV and generated table drift apart.

## Regenerating

```bash
crates/shanti-keysyms/generator/keysyms_extract.py --diff
cargo test  # in this crate; asserts TSV ↔ data.rs ↔ riti.h consistency
```

Then rebuild the IBus engine (`bash tools/build.sh --backend ibus --package`)
and re-verify typing behaviour before committing.
