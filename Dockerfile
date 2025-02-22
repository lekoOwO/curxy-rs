FROM rust:1.83-slim-bookworm AS builder

WORKDIR /usr/src/app
COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# Use distroless as runtime image
FROM gcr.io/distroless/cc-debian12

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/curxy-rs /app/

USER nonroot
ENTRYPOINT ["/app/curxy-rs"] 