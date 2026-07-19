# Contributing

Thank you for your interest in contributing to this project. Issues, bug
reports, and pull requests are welcome.

## Reporting issues

When opening an issue, please include:

- The exact version (from `Cargo.toml` `[workspace.package] version` or the
  container image tag you ran).
- A minimal reproduction — commands, configuration, and observed vs expected
  output.
- Relevant log lines (truncated to the interesting portion).

## Submitting changes

1. Fork the repository and create a feature branch from `main`.
2. Make your change. Keep commits focused; prefer several small commits over
  one large commit.
3. Run the full local check before pushing:
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check
   ```
4. Push your branch and open a pull request against `main` with a clear
  description of the change and the reasoning behind it.

## Coding style

- Rust 2024 edition (or the edition declared in `Cargo.toml`).
- `cargo fmt` formatting is authoritative.
- Lints are enforced via `[workspace.lints.rust]` and `[workspace.lints.clippy]`
  — see `Cargo.toml`.
- Prefer `?` over `unwrap()` outside test code.

## Commit messages

- Imperative mood ("Add feature", not "Added feature").
- First line is a short summary, ideally under 72 characters.
- Leave the body blank for trivial changes; otherwise explain *why*, not
  *what*.

## License

By submitting a contribution, you agree that your contribution will be
licensed under the same license as the project (see `LICENSE`).
