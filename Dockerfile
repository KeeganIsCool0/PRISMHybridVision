FROM ubuntu:22.04

# Avoid interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive

# Install necessary build dependencies and runtime libraries
RUN apt-get update && apt-get install -y \
    curl \
    make \
    g++ \
    cmake \
    git \
    libpipewire-0.3-dev \
    libasound2-dev \
    libjack-jackd2-dev \
    libgl1-mesa-dev \
    libx11-dev \
    libxcursor-dev \
    libxrandr-dev \
    libxinerama-dev \
    libxi-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Create app directory
WORKDIR /app

# Remove any existing build directory to avoid CMake conflicts
RUN rm -rf Firmware/build

# Copy the source code
COPY . .

# Build the project
RUN mkdir -p Firmware/build && \
    cd Firmware/build && \
    cmake .. -DBUILD_TESTING=OFF && \
    make && \
    cd /app && \
    cargo build --release

# Set the environment variable that the Rust binary expects
ENV ATMOS_PANNER_EXE=/app/Firmware/build/atmos-panner

# Run the binary
ENTRYPOINT ["./target/release/prism-hybrid-vision"]