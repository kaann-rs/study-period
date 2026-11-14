# lox-rs

`lox-rs` is a learning project that implements a Lox interpreter in Rust by
following [Crafting Interpreters](https://craftinginterpreters.com/).

The goal is not to provide a complete language for production use, but to
learn the scanner, token, parser, AST, and interpreter stages in small,
understandable steps. Version numbers follow [SemVer](https://semver.org/),
and the project is still in the `0.x` development stage.

## Getting started

Rust and Cargo must be installed.

```bash
cargo run
cargo run -- main.lox
```

Checks:

```bash
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings -A dead_code -A clippy::struct_field_names
```

## Versioning

New versions are published with a `vMAJOR.MINOR.PATCH` tag. For example:

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

When a tag is pushed, Gitea Actions runs the quality checks and creates a
GitHub-compatible release.

## Project structure

- `src/scanner.rs`: Splits source text into tokens.
- `src/parser.rs`: Builds an expression tree from tokens.
- `src/expr.rs`: Defines AST nodes.
- `src/lib.rs`: Provides the entry points for running Lox.
- `src/main.rs`: Provides the CLI entry point.

This repository is a learning exercise; its API and behavior may change as the
implementation progresses through the book.
