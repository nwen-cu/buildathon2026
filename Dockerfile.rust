# linux/amd64 recon image: compiled control replica (aim-nv / idle / lead-gun).
FROM rust:1.85-bookworm@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4 AS build
WORKDIR /src
COPY rust/Cargo.toml rust/Cargo.lock /src/
COPY rust/src /src/src
RUN cargo build --release --locked

FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
WORKDIR /bot
COPY --from=build /src/target/release/mechbot /bot/mechbot
COPY PROBE /bot/PROBE
ENTRYPOINT ["/bot/mechbot"]
