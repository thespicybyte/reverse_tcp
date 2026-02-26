# reverse_tcp

A Mythic C2 profile container implementing a reverse TCP channel, written entirely in Rust.

## Overview

`reverse_tcp` listens on a TCP socket, receives messages from agents using a 4-byte length-prefixed framing protocol, and forwards them to the Mythic server via the Push C2 gRPC interface. Responses from Mythic are forwarded back to agents over the same TCP connection.

The entire backend is written in Rust, made possible by two crates that are Rust ports of Mythic's official Go libraries:

- [`mythic-rabbitmq`](https://github.com/thespicybyte/mythic_rabbitmq) — Rust port of Mythic's Go RabbitMQ library. Used by C2 profile containers to sync profile definitions, handle config checks, OPSEC checks, IOC extraction, and other callbacks from the Mythic server.
- [`mythic-grpc`](https://github.com/thespicybyte/mythic_grpc) — Rust port of Mythic's Go gRPC library. Used by translation containers and C2 server code to forward agent messages to Mythic via bidirectional gRPC streaming (Push C2).

This means C2 profiles, translation containers, payload type containers, and C2 servers can all be written in Rust.

## Architecture

The container runs two binaries:

| Binary | Crate | Purpose |
|---|---|---|
| `reverse_tcp_profile` | `mythic-rabbitmq` | Syncs the C2 profile definition to Mythic via RabbitMQ; handles config/OPSEC checks |
| `reverse_tcp_server` | `mythic-grpc` | TCP listener that proxies agent messages to/from Mythic via gRPC Push C2 |

## C2 Profile Parameters

| Parameter | Type | Default | Description |
|---|---|---|---|
| `port` | Number | `4444` | TCP port to listen on |
| `killdate` | Date | 365 days | Agent kill date |
| `encrypted_exchange_check` | Boolean | `false` | Enable encrypted key exchange |
| `AESPSK` | Choice | `none` | Encryption: `aes256_hmac` or `none` |
| `callback_interval` | Number | `10` | Callback interval in seconds |

## Installation via mythic-cli

Install the container from the pre-built image in the GitHub Container Registry:

```bash
sudo ./mythic-cli install github https://github.com/thespicybyte/reverse_tcp
```

To start the container manually:

```bash
sudo ./mythic-cli start reverse_tcp
```

## Configuration

The C2 server reads its listen address and port from environment variables at runtime:

| Variable | Default | Description |
|---|---|---|
| `LISTEN_ADDR` | `0.0.0.0` | IP address to bind |
| `LISTEN_PORT` | `4444` | TCP port to bind |

## License

BSD-3-Clause
