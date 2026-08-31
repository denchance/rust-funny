# Contributing

Thanks for considering a contribution to rust-funny! This file explains how to set up the environment, follow the code style, run the tests, and open a pull request.

## Development environment

- A stable Rust toolchain (edition 2021).
- Dependencies are managed automatically by Cargo (`crossterm`, `rand`).

## Getting started

1. Fork the repository and clone it locally.
2. Confirm it builds and passes the tests:

```bash
cargo build
cargo test
```

## Code style

- Run `cargo fmt` before committing (formatting is governed by rustfmt).
- Run `cargo clippy --all-targets --all-features -- -D warnings` and keep it warning-free.
- Follow the existing conventions: unit tests live inline in `src/`; integration tests live in `tests/` and use only the public API.
- Code comments are currently written in Chinese (matching the existing codebase).

## Commit messages

Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```
<type>: <short description>
```

Common types: `feat` (new feature), `fix` (bug fix), `refactor`, `docs`, `test`, `chore`.

## Pull request workflow

1. Create a branch from `main`.
2. Implement the change and add tests.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test` — all must pass.
4. Open the PR and fill in the template (background, changes, verification).

## Adding new features

- New poses/attacks: see the "Maintainer guide" section of the README.
- After changing rendering, use `cargo run -- --dump 80 24` to preview a frame without a TTY.

## Code of conduct

Please follow the [Code of Conduct](CODE_OF_CONDUCT.md).
