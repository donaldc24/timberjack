#!/bin/bash
# Simple script to compare sequential vs parallel processing performance

set -e  # Exit on error

# Build the latest version
echo "Building Timber..."
cargo build --release

TIMBER_PATH="./target/release/timber"
BENCH_DIR="benchmark_data"
mkdir -p $BENCH_DIR

# Create test dataset if it doesn't exist
if [ ! -f "$BENCH_DIR/bench_1m.log" ]; then
    echo "Creating benchmark dataset..."
    cargo run --bin create_benchmark_logs -- 1000000 $BENCH_DIR/bench_1m.log
fi

# Run sequential mode benchmark
echo -e "\nRunning sequential mode benchmark..."
time $TIMBER_PATH --sequential --stats $BENCH_DIR/bench_1m.log > /dev/null

# Run parallel mode benchmark
echo -e "\nRunning parallel mode benchmark..."
time $TIMBER_PATH --parallel --stats $BENCH_DIR/bench_1m.log > /dev/null

# Run auto-detect mode benchmark
echo -e "\nRunning auto-detect mode benchmark..."
time $TIMBER_PATH --stats $BENCH_DIR/bench_1m.log > /dev/null

echo -e "\nBenchmark comparison complete!"