#!/bin/bash
# Comprehensive build script for PRISM Hybrid Vision
# Handles both Rust and C++ compilation

set -e  # Exit on any error
set -u  # Exit on undefined variable

echo "================================================"
echo "  PRISM Hybrid Vision Build and Run Script"
echo "================================================"
echo

# Function to show usage
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo "Options:"
    echo "  --rust-only    Build only Rust components"
    echo "  --cpp-only     Build only C++ components"
    echo "  --run          Build and run the application"
    echo "  --help         Show this help message"
    echo
    exit 1
}

# Parse command line arguments
BUILD_RUST=true
BUILD_CPP=true
RUN_APP=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --rust-only)
            BUILD_RUST=true
            BUILD_CPP=false
            shift
            ;;
        --cpp-only)
            BUILD_RUST=false
            BUILD_CPP=true
            shift
            ;;
        --run)
            BUILD_RUST=true
            BUILD_CPP=true
            RUN_APP=true
            shift
            ;;
        --help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

# Get script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$SCRIPT_DIR"

echo "Project root: $PROJECT_ROOT"
echo

# Build C++ components
if [ "$BUILD_CPP" = true ]; then
    echo "--- Building C++ Components ---"

    # Create build directory
    BUILD_DIR="$PROJECT_ROOT/Firmware/build"
    mkdir -p "$BUILD_DIR"

    # Run CMake
    echo "Running CMake configuration..."
    cd "$BUILD_DIR"
    if ! cmake .. -DBUILD_TESTING=OFF; then
        echo "Error: CMake configuration failed"
        exit 1
    fi

    # Build with Make
    echo "Building with Make..."
    if ! make -j$(nproc); then
        echo "Error: Make build failed"
        exit 1
    fi

    echo "C++ build completed successfully!"
    echo
fi

# Build Rust components
if [ "$BUILD_RUST" = true ]; then
    echo "--- Building Rust Components ---"

    # Check if Cargo.toml exists
    if [ ! -f "$PROJECT_ROOT/Cargo.toml" ]; then
        echo "Warning: Cargo.toml not found, skipping Rust build"
    else
        echo "Building Rust project with Cargo..."
        cd "$PROJECT_ROOT"
        if ! cargo build --release; then
            echo "Error: Rust build failed"
            exit 1
        fi
        echo "Rust build completed successfully!"
        echo
    fi
fi

# Run the application
if [ "$RUN_APP" = true ]; then
    echo "--- Running Application ---"

    # Try to run the C++ executable first (as indicated by build.rs and src/main.rs)
    if [ -f "$PROJECT_ROOT/Firmware/build/atmos-panner" ]; then
        echo "Running C++ application: ./Firmware/build/atmos-panner"
        "$PROJECT_ROOT/Firmware/build/atmos-panner"
    # Fallback to Rust binary
    elif [ -f "$PROJECT_ROOT/target/release/prism-hybrid-vision" ]; then
        echo "Running Rust application: ./target/release/prism-hybrid-vision"
        "$PROJECT_ROOT/target/release/prism-hybrid-vision"
    elif [ -f "$PROJECT_ROOT/target/debug/prism-hybrid-vision" ]; then
        echo "Running Rust application (debug): ./target/debug/prism-hybrid-vision"
        "$PROJECT_ROOT/target/debug/prism-hybrid-vision"
    else
        echo "Error: No executable found to run!"
        echo "Built executables:"
        echo "  - C++: $PROJECT_ROOT/Firmware/build/atmos-panner"
        echo "  - Rust: $PROJECT_ROOT/target/release/prism-hybrid-vision"
        echo "  - Rust (debug): $PROJECT_ROOT/target/debug/prism-hybrid-vision"
        exit 1
    fi
fi

echo
echo "================================================"
echo "  Build and Run Process Completed Successfully!"
echo "================================================"