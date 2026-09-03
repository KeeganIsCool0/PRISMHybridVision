#!/bin/bash
# Script to compile Rust and C++ files and run the application

set -e  # Exit on any error

echo "=== PRISM Hybrid Vision Build and Run Script ==="

# Step 1: Create build directory
echo "Creating build directory..."
mkdir -p Firmware/build

# Step 2: Run CMake to configure the C++ project
echo "Running CMake configuration..."
cd Firmware/build
cmake .. -DBUILD_TESTING=OFF

# Step 3: Compile the C++ project with Make
echo "Building C++ project with Make..."
make

# Step 4: Go back to project root
cd ..

# Step 5: Compile the Rust project (if there's Cargo.toml)
if [ -f "Cargo.toml" ]; then
    echo "Building Rust project..."
    cargo build --release
fi

# Step 6: Run the application
echo "Running the application..."
if [ -f "Firmware/build/atmos-panner" ]; then
    ./Firmware/build/atmos-panner
elif [ -f "target/release/prismhybridvision" ]; then
    # If it's a Rust binary instead
    ./target/release/prismhybridvision
else
    echo "Error: Could not find executable to run"
    echo "Expected: Firmware/build/atmos-panner or target/release/prismhybridvision"
    exit 1
fi

echo "=== Done ==="