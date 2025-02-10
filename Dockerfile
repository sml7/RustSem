# ---- STAGE 1: Builder ----
FROM debian:bullseye-slim AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libfontconfig1-dev \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
RUN . $HOME/.cargo/env && rustup default stable

ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Cargo.toml + Cargo.lock kopieren
COPY Cargo.toml Cargo.lock ./

# Dummy "main.rs" für Dependency-Caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Jetzt den Rest kopieren
COPY . .
RUN cargo build --release

# ---- STAGE 2: Laufzeit-Image (schlank) ----
FROM debian:bullseye-slim

# Hier: Runtime-Bibliotheken installieren
RUN apt-get update && apt-get install -y \
    libfontconfig1 \
    libssl1.1 || apt-get install -y libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rust_web_app /app/rust_web_app
COPY cleaned_data.csv /app/cleaned_data.csv

EXPOSE 3000
CMD ["./rust_web_app"]
