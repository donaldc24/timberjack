#!/bin/bash
# test_command.sh

TIMBER_PATH="$(pwd)/target/release/timber.exe"
LOG_FILE="$(pwd)/benchmark_data/bench_10k.log"

echo "Testing command: $TIMBER_PATH --chop ERROR $LOG_FILE"
"$TIMBER_PATH" --chop ERROR "$LOG_FILE"

echo "Exit code: $?"