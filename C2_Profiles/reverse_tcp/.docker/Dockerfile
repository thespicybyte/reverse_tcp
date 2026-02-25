# ---- Builder ----
FROM rust:1.93 as builder
WORKDIR /Mythic/
COPY [".", "."]
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update && apt-get install -y musl-tools protobuf-compiler
RUN make build

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y make && rm -rf /var/lib/apt/lists/*

COPY --from=builder /Mythic/reverse_tcp/bin/reverse_tcp_profile /reverse_tcp_profile
COPY --from=builder /Mythic/reverse_tcp/bin/reverse_tcp_server /reverse_tcp_server

WORKDIR /Mythic/
COPY [".", "."]

CMD make run
