# MyLAN agent on Raspberry Pi

Native build guide for RPi 3/4/5 (64-bit `aarch64`) or RPi Zero/1 (32-bit `armhf`).
The agent runs as a systemd service with the API embedded in-process (ADR-4).

## Cross-compile from an x86 host

Add the target and the cross linker:

```sh
rustup target add aarch64-unknown-linux-gnu
sudo apt-get install gcc-aarch64-linux-gnu
```

Configure `~/.cargo/config.toml`:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

Build (RPi 3/4/5 64-bit):

```sh
cargo build --release --target aarch64-unknown-linux-gnu -p mylan-agent
```

For RPi Zero/1 (32-bit `armhf`):

```sh
rustup target add arm-unknown-linux-gnueabihf
sudo apt-get install gcc-arm-linux-gnueabihf
cargo build --release --target arm-unknown-linux-gnueabihf -p mylan-agent
```

## Install on the RPi

Copy the binary, config, and systemd unit:

```sh
scp target/aarch64-unknown-linux-gnu/release/mylan-agent pi@<rpi>:/usr/local/bin/
scp agent/mylan-agent.toml pi@<rpi>:/etc/mylan/
scp agent/systemd/mylan-agent.service pi@<rpi>:/etc/systemd/system/
```

Create the `mylan` user and state dir, then enable + start:

```sh
ssh pi@<rpi> 'sudo useradd -r -s /usr/sbin/nologin mylan || true
              sudo mkdir -p /var/lib/mylan /etc/mylan
              sudo chown mylan:mylan /var/lib/mylan
              sudo systemctl daemon-reload
              sudo systemctl enable --now mylan-agent'
```

## Privilege notes (ARP sweep)

ARP sweep needs `CAP_NET_RAW` (or sudo) to see MACs of all hosts. Without it,
the agent degrades gracefully to ICMP/TCP-ping/mDNS/SSDP (P1 — never crashes,
less coverage). Grant the capability to the binary:

```sh
ssh pi@<rpi> 'sudo setcap cap_net_raw+ep /usr/local/bin/mylan-agent'
```

(Running the service as root is not recommended.)

## Verify

```sh
ssh pi@<rpi> 'systemctl is-active mylan-agent \
              && curl -sf http://127.0.0.1:43117/api/v1/status'
```

If `setcap` is not applied, the agent still runs but with reduced discovery
coverage — check `journalctl -u mylan-agent` for the degradation log line.