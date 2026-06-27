# AGENTS.md

## Scope

This directory implements the network discovery (liveness) phase of MyLAN. It detects local network interfaces, routes, and active hosts using a combination of ARP cache reading, ICMP probes, TCP-connect sweeps, mDNS, SSDP, and privileged ARP scans.

## Local Structure

- `src/iface.rs` — Discovers active LAN interfaces, local IPs, and gateways using the `netdev` crate.
- `src/arp.rs` — Parses `/proc/net/arp` to harvest MAC addresses resolved by the OS kernel.
- `src/tcp_ping.rs` — Performs concurrent TCP-connect sweeps on common LAN ports.
- `src/mdns.rs` & `src/ssdp.rs` — Listeners for passive multicast service announcements.
- `src/sudo.rs` — Privileged active discovery techniques (e.g., raw ARP sweep) utilizing packet injection via `pnet`.
- `src/icmp.rs` — ICMP ping sweeps supporting both raw sockets (sudo) and `SOCK_DGRAM` (non-root).

## Local Commands

```bash
# Run tests for the mylan-discovery crate only
cargo test -p mylan-discovery
```

## Local Conventions

- **Graceful Degradation**: Always verify that operations falling back to non-sudo paths (such as when raw socket permission is denied) fail/downgrade gracefully.
- **Throttling**: Limit maximum concurrency on active network scans using `tokio::sync::Semaphore` (configured via `DiscoverOptions::concurrency`).
- **Platform Restrictions**: Isolate Linux-specific mechanisms (e.g., `/proc` file paths) and provide portable fallbacks.

## Testing

- Unit tests mock interface properties and verify observation aggregation logic.
- Avoid introducing real network dependencies in standard unit tests.

## Do Not

- Do not perform active intrusive attacks (e.g., ARP spoofing, Wi-Fi deauth, MITM).
- Do not block the async executor thread; wrap blocking socket reads or writes in `tokio::task::spawn_blocking`.
