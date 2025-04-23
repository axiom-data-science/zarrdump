FROM rust:1.86.0-slim-bookworm

# Metadata
LABEL org.opencontainers.image.authors="Luke Campbell <luke@axds.co>"
LABEL org.opencontainers.image.url="https://git.axiom/axiom/zarrdump/"
LABEL org.opencontainers.image.source="https://git.axiom/axiom/zarrdump/"
LABEL org.opencontainers.image.licenses="MIT"

RUN apt-get update && apt-get install -y build-essential && apt-get clean -y

# Build the release binary
WORKDIR /opt/zarrdump
COPY src ./src
COPY README.md LICENSE Cargo.toml ./
RUN cargo build --release

# Copy release binary to fresh buster-slim image
FROM debian:bookworm-slim
COPY --from=0 /opt/zarrdump/target/release/zarrdump /usr/bin/zarrdump
ENTRYPOINT ["/usr/bin/zarrdump"]
