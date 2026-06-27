# AGENTS.md

## Scope

This directory contains the primary command-line binary (`mylan`) and frontend orchestration logic. It parses command-line arguments, dispatches subcommands, and coordinates the scanning pipeline from discovery to enrichment and database persistence.

## Local Structure

- `src/main.rs` — Entry point, tracing initialization, and error reporting.
- `src/cli.rs` — Argument parser definitions using `clap` (subcommands, options, profiles).
- `src/pipeline.rs` — Coordinates the multi-crate execution pipeline (discovery -> fingerprinting -> database upsert).
- `src/ctx.rs` — App context wrapper containing database connections, cancellation tokens, and configuration.
- `src/commands/` — Implementation handlers for subcommands (`scan`, `ports`, `devices`, `device`, `export`, etc.).

## Local Commands

```bash
# Run tests for the cli binary only
cargo test -p mylan-cli

# Build and run the local mylan binary
cargo run --bin mylan -- --help
```

## Local Conventions

- **Application Error Handling**: Use `anyhow` for top-level application logic and subcommand errors.
- **Output Presentation**: Use the `comfy-table` crate or structured JSON for CLI output to keep formatting consistent and user-friendly.

## Testing

- Subcommand and CLI logic can be tested using temporary SQLite files to avoid polluting persistent host configurations.
- Verify pipeline synchronization across cancellation and timeout scenarios.

## Do Not

- Do not bypass the orchestrating pipeline to perform direct, unrecorded database writes or network probes.
- Do not mix presentation formatting styles inside domain crates; keep table rendering and stdout/stderr output restricted to the CLI binary package.
