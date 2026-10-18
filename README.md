# Study Period

<img src="./assets/study.png" width="50" height="50" alt="Study Period" />

This is my personal workspace for studying programming and computer science. It collects the exercises, small tools, and experiments I build while reading books and working through their examples.

For example, when a book introduces a text replacement tool, I implement it as I read, try it out, and keep the result here. Some projects follow a book closely; others are personal experiments alongside my studies.

These are learning projects at different stages of completion. They may contain unfinished features, rough edges, and notes from the learning process.

## Projects

| Directory | What I am working on |
| --- | --- |
| `rust-study/programming-rust` | A command-line greatest common divisor calculator. |
| `rust-study/actix-gcd` | A web-based greatest common divisor calculator using Actix Web. |
| `rust-study/mandelbrot` | A Mandelbrot image renderer with parallel processing. |
| `rust-study/quickreplace` | A command-line tool for replacing text using regular expressions. |
| `rust-study/lox-rs` | A Lox interpreter built while following *Crafting Interpreters*. |
| `rust-study/ahtapot-rs` | A personal CLI project for bulk image renaming, resizing, and format conversion. |
| `rust-study/bird-lang` | Early experiments with building a programming language. |

`math-study` and `eng-study` are placeholders for future study material.

## Running an exercise

Each Rust project is a separate Cargo package. Open its directory and run Cargo there; this repository does not have a shared Cargo workspace.

For example:

```bash
cd rust-study/programming-rust
cargo run -- 14 28 42
```

To try the text replacement exercise from the repository root:

```bash
cd rust-study/quickreplace
cargo run -- world Rust test.txt /tmp/quickreplace-output.txt
```

Rust and Cargo are required. Check each project's source or README for its arguments and any additional setup. Build output directories (`target/`) are excluded from version control.

## Reading context

- `lox-rs` follows *Crafting Interpreters*.
- The GCD, Mandelbrot, and text replacement exercises are grouped under `rust-study` alongside my other Rust work.

Book examples and my own experiments live together here. This repository records my progress rather than a set of finished applications.
