# Contributing to mermaid-cli

## Development Setup

1. Ensure you have Rust installed (MSRV: stable).
2. Clone the repository.
3. Run `cargo build` to verify the build.
4. Run `cargo test` to verify all tests pass.

## Code Quality

- **Formatting**: All code must be formatted with `cargo fmt` before committing.
- **Linting**: Run `cargo clippy` and address all warnings before submitting.
- **Warnings**: The project compiles with zero warnings. Please maintain this.

## Testing

### Unit Tests

```bash
cargo test --lib
```

### Integration Tests

```bash
cargo test --test '*'
```

### All Tests

```bash
cargo test --all
```

## Pull Request Process

1. Ensure all existing tests pass and new tests are added for new features.
2. Run `cargo fmt` and `cargo clippy` before pushing.
3. Update documentation (doc comments and README) as needed.
4. Verify the CI workflow passes on GitHub.

## Project Structure

- `src/lib.rs` — Library entry point, public API functions
- `src/main.rs` — CLI binary entry point
- `src/parser/` — Lexer, Parser, and AST definitions
- `src/renderer/` — Diagram rendering logic
- `src/svg/` — SVG generation utilities
- `src/fixer/` — Syntax error recovery / auto-fix
- `tests/` — Integration tests

## License

By contributing, you agree that your contributions will be licensed under the project's license.
