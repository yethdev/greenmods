# GreenMods VPS Notes

Target box: 10 vCore ARM64, 16 GB RAM, 512 GB NVMe, 2500 Mbps included traffic.

## Build

Build on the VPS so Rust can use the native ARM64 toolchain and avoid cross-linker friction:

```sh
sudo apt update
sudo apt install -y build-essential pkg-config curl git
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
cargo build --release -p greenmods
```

The release profile uses thin LTO, one codegen unit, stripped symbols, and aborting panics. That keeps the server binary compact and gives the 10-core CPU room to optimize hot paths.

## Services

Run these as separate local services:

- GreenMods API/UI on `127.0.0.1:4000`.
- PostgreSQL on local private networking or `127.0.0.1`.
- Meilisearch on `127.0.0.1:7700`.
- S3-compatible storage for mod files and gallery images.
- Nginx or Caddy for TLS, compression, request body limits, and proxy buffering.

For Nginx, set the upload cap intentionally. GreenMods rejects bad file names and junk uploads, but the proxy should still stop oversized bodies early:

```nginx
client_max_body_size 512m;
proxy_request_buffering on;
proxy_read_timeout 120s;
```

## Runtime

Use a systemd service with restart limits:

```ini
[Unit]
Description=GreenMods
After=network-online.target postgresql.service

[Service]
WorkingDirectory=/opt/greenmods
ExecStart=/opt/greenmods/target/release/greenmods
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

## Sizing

The app process should sit comfortably below 1 GB under normal traffic. Keep the remaining memory for Postgres page cache, Meilisearch indexes, and reverse-proxy buffers. Put Postgres data, Meilisearch data, and object-storage cache on NVMe, then snapshot the database and buckets separately.

## Backups

Back up these on a schedule:

- PostgreSQL logical dump daily.
- S3 bucket/object storage daily.
- `ModHost.toml` and the source checkout after every deployment.

Do not back up access logs forever. Rotate them and keep only what you need for abuse investigation.
