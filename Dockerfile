# Multi-toolchain Docker image containing Rust & Python environments
FROM rust:1.82-bookworm

# Install Python 3 and build utilities
RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Install Python requirements
COPY requirements.txt .
RUN pip3 install --no-cache-dir --break-system-packages -r requirements.txt

# Copy source code
COPY . .

# Pre-build Rust crate in release mode
WORKDIR /workspace/textdistancerust
RUN cargo build --release && cargo test

WORKDIR /workspace

# Add compiled release binary to PATH
ENV PATH="/workspace/textdistancerust/target/release:${PATH}"

# Default command runs Rust test suite
CMD ["cargo", "test", "--manifest-path", "textdistancerust/Cargo.toml"]
