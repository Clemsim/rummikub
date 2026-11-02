# AGENTS.md

## Project

## Project

Cargo workspace. `crates/rummikub-core` holds the game logic: package `rummikub-core`, lib target `rummikub_core` (`src/lib.rs`, no I/O), plus a thin demo binary (bin target `rummikub`). `crates/rummikub-ia` holds the AI players (bots, search, learning) and only uses the core's public API — the AIs play through the same legal moves as humans. Modules are declared in each crate's `src/lib.rs`; unit tests live in dedicated `src/<module>/tests.rs` files, each declared with `#[cfg(test)] mod tests;`, plus integration tests in each crate's `tests/`. Edition 2024 — requires a recent toolchain (1.85+).

## Gotchas

- **Modules are silently ignored unless declared.** Every module must be declared in its crate's `src/lib.rs` or it will never compile.
- **French naming is a project-wide rule.** Modules, types, fields, methods, and locals use French: `Tuile`, `Couleur::Rouge`, `piocher`, `ouvrir_tour`, etc., and comments are in French. `Tuile.Type` is capitalized because `type` is a reserved keyword. Test function names stay English (Rust convention, used with `cargo test <name>`). Do not rename to English. `#![allow(non_snake_case)]` at the top of `src/lib.rs` silences the resulting warnings — do not remove it.
- Domain invariants exercised by tests: jokers in runs resolve to values in 1..=13 (no duplicate values, gaps must not exceed joker count); the initial meld needs ≥ 30 points from melds built only from the player's unique tile IDs. The legal-move enumeration (`coups_possibles`) is deterministic (BTreeMap ordering, same hand -> same list in the same order) and treats exact duplicates as interchangeable (one canonical copy per (couleur, valeur)) — the AIs rely on that for reproducible simulations.

## Commands

- `cargo build` / `cargo test` (at the workspace root: covers every crate)
- Demo binary: `cargo run -p rummikub-core`
- Single test: `cargo test -p rummikub-core <name>` (e.g. `cargo test -p rummikub-core test_valid_runs_with_jokers`)
- `cargo clippy` and `cargo fmt` components are installed — run both before finalizing changes.
- Bench: `cargo run -p rummikub-ia [graines] [simulations] [epsilon]` — ISMCTS vs the heuristic bot over fixed seeds; results are appended to `banc/resultats.csv`.
- Epsilon sweep + curve: `cargo run -p rummikub-ia graphique [graines] [simulations]` — writes `banc/balayage-epsilon.csv` and `banc/graphique-epsilon.png`. Run benchmarks and sweeps with `--release` (5-7x faster than debug).
- Profiling: `[profile.release] debug = true` is set at the workspace root for `cargo flamegraph`; `perf.data` and `flamegraph.svg` are gitignored.
- Terminal chart fallback if ever needed: the `textplots` crate draws an instant ASCII curve (plotters stays the main choice for PNG/SVG).
- Architecture decisions live in `docs/adr/` — read them before touching the
  move enumeration (determinism, scope), and record significant decisions there.
