# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a [Mythic](https://github.com/MythicMeta/Mythic) C2 profile container implementing a reverse TCP channel, written entirely in Rust. It is designed to be installed and run as a Mythic C2 container via `mythic-cli`.

## Repository Layout

```
C2_Profiles/reverse_tcp/
├── Makefile                  # Build, run, clean, stop targets
├── Dockerfile                # Production image (pulls from GHCR)
├── .docker/Dockerfile        # Multi-stage builder image (used by CI)
└── reverse_tcp/
    ├── mythic/               # reverse_tcp_profile binary (RabbitMQ / profile sync)
    │   ├── src/main.rs       # MythicC2Container setup + RPC handlers
    │   ├── src/builder.rs    # C2 parameter definitions
    │   └── src/logging.rs
    ├── c2_code/              # reverse_tcp_server binary (TCP ↔ gRPC proxy)
    │   ├── src/main.rs
    │   ├── src/server.rs     # TCP listener + bidirectional gRPC streaming
    │   └── src/error.rs
    └── bin/                  # Compiled binaries land here (gitignored)
```

## Two-Binary Architecture

| Binary | Crate | Role |
|---|---|---|
| `reverse_tcp_profile` | `mythic-rabbitmq` | Connects to Mythic via RabbitMQ; syncs the profile definition and handles config/OPSEC check callbacks |
| `reverse_tcp_server` | `mythic-grpc` | TCP listener (default :4444); proxies agent messages to Mythic via bidirectional gRPC Push C2 streaming |

The profile binary manages the server binary's lifecycle — it spawns/monitors `reverse_tcp_server` as a child process.

## Key External Crates

- [`mythic-rabbitmq`](https://github.com/thespicybyte/mythic_rabbitmq) — local path dependency (`/home/spicybyte/projects/mythic_rabbitmq`). Rust port of Mythic's Go RabbitMQ library.
- [`mythic-grpc`](https://github.com/thespicybyte/mythic_grpc) — Git dependency. Rust port of Mythic's Go gRPC library.

## Build Commands

```bash
# Build both binaries (musl static, x86_64)
make build

# Clean all build artifacts and binaries
make clean

# Run locally (copies pre-built binaries from / into bin/, then starts profile)
make run

# Kill both processes
make stop
```

Individual crate builds (from their respective directories):
```bash
cd reverse_tcp/mythic && cargo build --release --target x86_64-unknown-linux-musl
cd reverse_tcp/c2_code && cargo build --release --target x86_64-unknown-linux-musl
```

The build requires `musl-tools` and `protobuf-compiler` (`apt`), and the `x86_64-unknown-linux-musl` rustup target.

## Environment Variables

`.env` (in the repo root, not committed with secrets) controls local development:

| Variable | Default | Description |
|---|---|---|
| `RABBITMQ_HOST` | `127.0.0.1` | RabbitMQ host for profile container |
| `RABBITMQ_PORT` | `5672` | RabbitMQ port |
| `RABBITMQ_PASSWORD` | — | RabbitMQ password |
| `MYTHIC_SERVER_HOST` | `127.0.0.1` | Mythic server host |
| `MYTHIC_SERVER_GRPC_PORT` | `17444` | Mythic gRPC port |
| `LISTEN_ADDR` | `0.0.0.0` | TCP bind address for `reverse_tcp_server` |
| `LISTEN_PORT` | `4444` | TCP port for `reverse_tcp_server` |
| `PROJECT_BIN_DIR` | `.` | Directory where `reverse_tcp_profile` looks for `reverse_tcp_server` |

## Wire Protocol

Agent ↔ server messages use **4-byte big-endian length-prefixed framing** over raw TCP. The payload is base64-encoded JSON forwarded to Mythic via gRPC `PushC2MessageFromAgent`.

## Release / CI

- CI (`.github/workflows/docker.yml`) builds and pushes to `ghcr.io/thespicybyte/reverse_tcp` on pushes to `main` or semver tags.
- On a tag push, the workflow also bumps `Dockerfile` and `config.json` (`remote_images.reverse_tcp`) to reference the new tag, then commits back to `main`.
- The `Dockerfile` at the repo root is the **production** image (single `FROM` pulling from GHCR). The `.docker/Dockerfile` is the **multi-stage builder** used by CI.
- The `config.json` at the repo root is the Mythic install manifest — keep `remote_images.reverse_tcp` in sync with the latest release tag.
