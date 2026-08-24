# Pin both Rust and cargo-cooldown instead of implicitly using their latest releases.
FROM rust:1.98.0-bullseye

ARG CARGO_COOLDOWN_VERSION=0.3.4
RUN cargo install --locked --version "${CARGO_COOLDOWN_VERSION}" cargo-cooldown \
    && cargo cooldown --version

# Keep runtime Cargo state disposable and writable when the container is run
# with the host user's UID/GID. The installed subcommand remains on PATH at
# /usr/local/cargo/bin/cargo-cooldown.
ENV HOME=/tmp/cargo-user \
    CARGO_HOME=/tmp/cargo-user/.cargo

WORKDIR /workspace
CMD ["bash"]
