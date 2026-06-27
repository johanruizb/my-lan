# AGENTS.md

## Scope

This crate handles the port scanning and service detection phase of MyLAN. It performs async TCP connect scans, limited UDP sweeps, and grabs service banners or headers (HTTP, SSH, etc.) to map active ports to service models.

## Local Structure

- `src/lib.rs` — Coordinates scanning routines, supports scan targets, and propagates progress reports.
- `src/ports.rs` — Catalog of common ports sorted by rank, mapping port numbers to service names.
- `src/probes.rs` — Service-specific probes for HTTP titles, SSH banners, and FTP greetings.
- `src/banner.rs` — Generic TCP banner grabbing with connection timeouts and short reads.
- `src/udp.rs` — Basic UDP sweeps targeting common LAN services (DNS, DHCP, SSDP).
- `src/profile.rs` — Port ranges and configurations matching scan profiles (quick, deep, normal, iot, router).

## Local Commands

```bash
# Run unit tests for mylan-scanner only
cargo test -p mylan-scanner
```

## Local Conventions

- **Intrusion Policy (P2)**: Port scans must remain non-intrusive. Rely strictly on standard TCP connections and socket exchanges without utilizing offensive exploits.
- **Resource Limits**: Concurrency must be governed via semaphores (`tokio::sync::Semaphore`).
- **Cooperativity**: Always support and propagate `CancellationToken` to avoid infinite hangs during sweeps.

## Testing

- Unit tests use local loopback listener setups or mock responses.
- Ensure timeouts are configured low to keep test execution fast.

## Do Not

- Do not perform active vulnerability scanning or OS fingerprint sweeps using raw sockets.
- Do not run blocking E/S operations directly on the async executor thread without `spawn_blocking`.
