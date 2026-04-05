# --- STAGE 1: The Builder ---
FROM rust:bookworm AS builder

# 1. Install system tools
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev wget unzip libatomic1 \
    && rm -rf /var/lib/apt/lists/*

# 2. Setup the Vosk Library (Architecture Aware)
ARG TARGETARCH
RUN case "${TARGETARCH}" in \
    "amd64")  VOSK_ARCH="x86_64" ;; \
    "arm64")  VOSK_ARCH="aarch64" ;; \
    *) VOSK_ARCH="aarch64" ;; esac && \
    wget https://github.com/alphacep/vosk-api/releases/download/v0.3.45/vosk-linux-${VOSK_ARCH}-0.3.45.zip && \
    unzip vosk-linux-${VOSK_ARCH}-0.3.45.zip && \
    cp vosk-linux-${VOSK_ARCH}-0.3.45/libvosk.so /usr/local/lib/ && \
    cp vosk-linux-${VOSK_ARCH}-0.3.45/vosk_api.h /usr/local/include/ && \
    ldconfig

# 3. NEW: Download and Setup the Vosk Model
# We do this in a separate folder to keep things organized
WORKDIR /app
RUN mkdir -p vosk && \
    wget https://alphacephei.com/vosk/models/vosk-model-small-en-us-0.15.zip && \
    unzip vosk-model-small-en-us-0.15.zip -d vosk/ && \
    rm vosk-model-small-en-us-0.15.zip

# 4. Build the Rust Application
COPY . .
# We use the name 'backend-cli' as identified in your Cargo.toml
RUN cargo build --release

# --- STAGE 2: The Runtime ---
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 libatomic1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the .so library from the builder
COPY --from=builder /usr/local/lib/libvosk.so /usr/local/lib/
RUN ldconfig

WORKDIR /app

# 1. Copy the compiled binary (renaming it to 'backend' for simplicity)
COPY --from=builder /app/target/release/backend-cli ./backend

# 2. Copy the configuration files
COPY --from=builder /app/config ./config

# 3. Copy the model folder we downloaded in Stage 1
COPY --from=builder /app/vosk ./vosk

# 4. Set the Environment Variable so the code knows where the model is
ENV VOSK_DIR=/app/vosk/vosk-model-small-en-us-0.15

EXPOSE 5150

# Start the Loco backend
CMD ["./backend", "start"]