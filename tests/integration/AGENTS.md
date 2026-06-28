# AGENTS.md

## Scope

This directory contains integration and end-to-end tests for MyLAN. They validate multi-crate interactions, database operations, pipeline execution (scan -> enrich -> persist -> export), and cancellation behaviors.

## Local Structure

- `src/lib.rs` — Shared helpers including mock database fixture generation (`fixture_db`), mock observations (`obs`, `obs_with_hint`), and sample network data (`sample_network`).
- `tests/export.rs` — Validates exporting data to JSON and CSV formats.
- `tests/scan_cancel.rs` — Validates scanner cancellation mid-scan and partial results retrieval.
- `tests/scan_pipeline.rs` — Validates the full scan pipeline from discovery to enrichment and database persistence.
- `tests/services_reporting.rs` — Validates service discovery reporting and exporting.

## Local Commands

```bash
# Run all integration tests
cargo test -p mylan-integration-tests
```

## Local Conventions

- **Database Isolation**: Always use temporary files/directories (e.g., via `tempfile`) and the `fixture_db` helper to instantiate databases for tests, ensuring isolation and clean test state.
- **Mock Observations**: Use the `obs` and `obs_with_hint` helper functions to construct mock observations instead of scanning the live host network, keeping tests deterministic.

## Testing

- Integration tests require no root privileges because they use mock network observations and temporary file-based database connections.
- Ensure all tests cleanup their resources automatically.

## Do Not

- Do not commit real hardware MAC addresses or secrets to test files; use placeholder MACs (like `aa:bb:cc:dd:ee:ff`) or dummy credentials to avoid failing the `./scripts/pre-push-safety.sh` check.
- Do not run tests that expect live external network connections (e.g. public DNS/ICMP) as they are prone to flakiness.
