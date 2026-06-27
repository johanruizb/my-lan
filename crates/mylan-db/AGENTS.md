# AGENTS.md

## Scope

This directory contains the SQLite local database persistence layer for MyLAN. It manages the connection lifecycle, embedded SQL schema migrations, and database repositories for networks, devices, addresses, services, and scans.

## Local Structure

- `src/connection.rs` — Manages connection opening, XDG data directory resolution, and initialization settings.
- `src/migrations.rs` — Embbeded SQL migrations governed by `PRAGMA user_version` (idempotent setup).
- `src/codec.rs` — Custom codecs to parse/format types (e.g., `MacAddr`, `IpAddr`, enums) for SQLite storage.
- `src/*_repo.rs` — Repositories containing SQL queries for CRUD/upsert actions on domain models.

## Local Commands

```bash
# Run tests for the mylan-db crate only
cargo test -p mylan-db
```

## Local Conventions

- **Foreign Keys**: Always enable referential integrity by running `PRAGMA foreign_keys = ON;` on connection setup.
- **SQL Parameter Binding**: Use positional or named bindings for all queries; never concatenate strings to form queries.
- **Schema Migrations**: Schema alterations must be appended to the `MIGRATIONS` array in `migrations.rs`, incrementing `user_version`.

## Testing

- Unit tests must either use SQLite in-memory databases (`Connection::open_in_memory()`) or write to temporary file locations using the `tempfile` crate.
- Verify migration idempotency and ensure data constraint validation checks are in place.

## Do Not

- Do not bypass referential integrity checks.
- Do not run blocking database operations directly in performance-critical asynchronous CLI paths without utilizing standard connections or blocking thread spawns when needed.
- Do not hardcode custom migration logic outside of `migrations.rs`.
