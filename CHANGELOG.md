# Changelog

Changes to this study workspace, newest first.

## 2026-11-24

### Added

- Learning sources at the end of every `rust-study/learn` source file: a `Learn more` comment lists, topic by topic, two or more sources from different sites. Beginner-friendly ones are marked `Easy:` and come last. It covers 194 topics in 48 files; every link was opened and its page and anchor checked.

### Changed

- Renamed the files and directories of `rust-study/book` to English: `temel`, `orta`, and `ileri` are now `basics`, `intermediate`, and `advanced`, `*-derinlesme.md` is `*-deep-dive.md`, `dogrula.py` is `verify.py`, `YAZAR-KILAVUZU.md` is `WRITING-GUIDE.md`, and `tema/kitap.css` is `styles/book.css`. The text of the book stays Turkish.
- Renamed the functions, variables, types, modules, and tests in `rust-study/learn` from Turkish to simple English (A1–B1). Module files follow: `derinlesme.rs` is now `deep_dive.rs`, and in `advanced-concurrency-async` the files are `threads.rs`, `runtime.rs`, and `client.rs`. Behavior is the same; printed text stays Turkish, only names inside it changed. The `project` crates already used English names, and the book keeps its Turkish examples.
- Renamed the nine `ileri-*` packages and their directories in `rust-study/learn/advanced-concepts` to `advanced-*`: `ileri-deref-metot` is now `advanced-deref-methods`, `ileri-eszamanlilik-async` is now `advanced-concurrency-async`, and the other seven only change their prefix (`cargo run -p ileri-future` becomes `cargo run -p advanced-future`). Older entries below keep the names used at the time.
- Formatted the `learn` crates with rustfmt.

### Removed

- `rust-study/reports` and `rust-study/research_notes`, the accuracy audit of the notes; the files remain in commit `22a3cca`.
- `rust-study/README.md`; the book (`rust-study/book`) holds the explanations and the order of the topics.

## 2026-11-23

### Added

- `rust-study/learn/*/src/deep_dive.rs` (then named `derinlesme.rs`): deepening modules for eleven Rust Book chapters (ownership, structs, enums, error handling, generics/traits/lifetimes, closures/iterators, smart pointers, concurrency, async/await, trait objects, patterns) with realistic scenarios, common compiler errors, exercises, and unit tests. The existing `main.rs` files only gained the two lines that wire the module in.
- `rust-study/learn/advanced-concepts/ileri-lifetime-hrtb-gat`: advanced lifetimes, higher-ranked trait bounds, and generic associated types.
- `rust-study/learn/advanced-concepts/ileri-eszamanlilik-async`: advanced concurrency (RwLock, Condvar, lock ordering, atomics) and a dependency-free mini async runtime with a simulated API client.
- `rust-study/README.md`: an index of the deepening modules and the two new advanced crates.
- A plain-language layer in every `learn` source file: a mini glossary at the top and a `SADE ANLATIM` box (everyday analogy plus a mini example) above each topic. Only comment lines were added; code, program output, and tests are unchanged.
- `rust-study/book`: the notes rewritten as a 43-page Turkish mdBook, with a script (`dogrula.py`) that compiles every example and checks outputs and links.

### Changed

- Moved the explanations out of `rust-study/learn` into the book: `.rs` comments and the `ACIKLAMA.md` files are removed, and each source file now only names its book chapter. Code, tests, and program output are unchanged; the original notes are in commit `22a3cca`. The doc comments in `cargo-and-crates/src/lib.rs` stay because they are doc-tests.
- Moved the nine `ileri-*` crates from `rust-study/learn/` into `rust-study/learn/advanced-concepts/`. Package names are unchanged, so `cargo run -p ileri-future` still works; the workspace manifest now lists them explicitly.
- Correction notes (`↳ DÜZELTME NOTU`) under 17 technically inaccurate comments in the original chapter files, each verified against rustc 1.99 or the std source; the original comments are kept.

## 2026-10-18

### Added

- Rust exercises: command-line GCD, an Actix Web GCD calculator, a Mandelbrot renderer, and a regular-expression text replacement tool.
- A README project index and instructions for running individual exercises.
- A repository-wide ignore rule for Rust build output directories.

### Changed

- Clarified that this repository is a personal study workspace for book exercises and related experiments.
- Translated Turkish notes and output text into English.

### Removed

- The `c-study`, `cpp-study`, `lua-study`, `docker-study`, `git-study`, `linux-study`, `math-study`, and `eng-study` directories.

## 2026-10-14

### Added

- Repository layout, one folder per topic: `c-study`, `cpp-study`, `rust-study`, `lua-study`, `docker-study`, `git-study`, `linux-study`, `math-study`, `eng-study`.
- `rust-study/ahtapot-rs`: Ahtapot, a CLI for bulk image renaming, resizing and format conversion.
- `rust-study/lox-rs`: a Lox interpreter in Rust following *Crafting Interpreters* (scanner, parser, AST).
- `rust-study/bird-lang`: the first steps of Bird, a programming language (Rust 2024 edition).
- `c-study/chip8-c`: a CHIP-8 emulator in C11 with raylib 6.0 (a window so far).
- `cpp-study/swarm-cpp`: a swarm simulation in C++20 with SFML 3.1 (window and event loop so far).
