# AGENTS.md

## Project

Single-crate Rust binary (no lib target) for Rummikub game logic. All modules are declared with `mod` in `src/main.rs`; tests live in dedicated `src/<module>/tests.rs` files, each declared with `#[cfg(test)] mod tests;`. Edition 2024 — requires a recent toolchain (1.85+).

## Gotchas

- **Modules are silently ignored unless declared.** Every module must be declared in `src/main.rs` or it will never compile.
- **French naming is a project-wide rule.** Types, fields, locals, and comments use `Tuile`, `Tuiles`, etc., and comments are in French. `Tuile.Type` is capitalized because `type` is a reserved keyword. Do not rename to English. `#![allow(non_snake_case)]` at the top of `src/main.rs` silences the resulting warnings — do not remove it.
- Domain invariants exercised by tests: jokers in runs resolve to values in 1..=13 (no duplicate values, gaps must not exceed joker count); the initial meld needs ≥ 30 points from melds built only from the player's unique tile IDs.

## Commands

- `cargo build` / `cargo test`
- Single test: `cargo test <name>` (e.g. `cargo test test_valid_runs_with_jokers`)
- `cargo clippy` and `cargo fmt` components are installed — run both before finalizing changes.
