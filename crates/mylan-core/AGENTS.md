# AGENTS.md

## Scope

This crate contains the pure domain models, enums, observations, and identity-merging algorithms of MyLAN. It serves as the foundation for all other crates in the workspace.

## Local Structure

- `src/models.rs` — Domain structures for network elements (Device, Interface, Network, Service, Scan, etc.).
- `src/observation.rs` — Observations representing facts collected from discovery techniques, and the `aggregate` function to merge them.
- `src/identity.rs` — Merging strategy and precedence confidence mapping (MAC/IP relationships).
- `src/mac.rs` — Helper for MAC address parsing, formatting, and validation.
- `src/enums.rs` — Core domain enumerations (Protocol, DeviceType, ServiceState, ScanStatus, etc.).
- `src/enrich.rs` — Concrete enricher interface definition for data enrichment.

## Local Commands

```bash
# Run unit tests for mylan-core only
cargo test -p mylan-core
```

## Local Conventions

- **Pure Domain (P3)**: Under no circumstances may code in this crate perform any I/O, read/write files, open network sockets, or contain OS/hardware-specific dependencies.
- **Serialization**: Ensure models are serializable (with `serde`) where required for persistence or export formats.
- **MacAddr Constraints**: Use `MacAddr` abstractions rather than raw strings for physical address operations.

## Testing

- Write comprehensive unit tests in inline modules (`mod tests`) for all parsing, validation, and observation merging logic.
- All testing must be purely deterministic.

## Do Not

- Do not add any external dependency that introduces file system, network, or OS-level capabilities.
- Do not bypass verification or confidence arithmetic when merging device observations.
