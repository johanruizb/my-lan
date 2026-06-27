# AGENTS.md

## Scope

This crate implements the device fingerprinting layer for MyLAN. It enriches the device observations collected in the discovery phase with hostname (via reverse DNS), vendor (via OUI prefix lookup), and device type/confidence (via a YAML rules engine).

## Local Structure

- `src/fingerprint.rs` — Integrates all fingerprinting components and provides the concrete `Enricher` implementation.
- `src/rules.rs` — Core YAML rules engine representing `any`/`all` matchers against device observation parameters.
- `src/oui.rs` — OUI prefix mapping database loader and query interface.
- `src/reverse.rs` — Best-effort reverse DNS hostname lookup utilities.

## Local Commands

```bash
# Run unit tests for mylan-fingerprint only
cargo test -p mylan-fingerprint
```

## Local Conventions

- **Passive Inference Only (P2)**: Fingerprinting must strictly operate on observations already collected during discovery. Do not perform active probes, network requests, or banner grabbing inside this crate.
- **Rule Persistence**: Any new identification rule should be defined as a YAML file under `signatures/device-rules/` rather than hardcoded in Rust.

## Testing

- Unit tests should mock OUI rows or rule maps to isolate matching logic.
- Avoid actual network/DNS operations in tests, relying on deterministic inputs.

## Do Not

- Do not perform any active network probes or connections (such as connect sweeps or TCP banners).
- Do not manually edit the auto-loaded `signatures/oui/oui.csv` prefix lookup database.
