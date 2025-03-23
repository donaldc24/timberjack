#!/bin/bash
export PATH="$PATH:/c/Python313:/c/Python313/Scripts"
set -e  # Exit on error

# Helper function to check if we're on Windows
is_windows() {
    [[ "$(uname)" =~ "MINGW"|"MSYS"|"CYGWIN" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]
}

BENCH_DIR="benchmark_data"
mkdir -p $BENCH_DIR

check_dependencies() {
    echo "Checking dependencies..."
    if ! command -v python3 &> /dev/null && ! command -v python &> /dev/null; then
        if [ -x "/c/Python313/python" ]; then
            echo "Found Python at /c/Python313/python, adding alias..."
            python3() {
                /c/Python313/python "$@"
            }
            python() {
                /c/Python313/python "$@"
            }
        else
            echo "Python not found. Please install Python 3."
            exit 1
        fi
    elif command -v python &> /dev/null; then
        echo "Using 'python' instead of 'python3'"
        python3() {
            python "$@"
        }
    fi

    PYTHON_VERSION=$(python3 --version 2>&1)
    echo "Python version: $PYTHON_VERSION"

    if is_windows; then
        echo "Windows detected. Skipping system tools check."
        echo "Please make sure you have the required tools installed manually."
    fi

    # Check if goaccess is installed
    if ! command -v goaccess &> /dev/null; then
        echo "GoAccess not found. Please install with:"
        echo "  Ubuntu/Debian: sudo apt-get install goaccess"
        echo "  macOS: brew install goaccess"
        echo "Will skip GoAccess benchmarks."
    fi

    # Check if lnav is installed
    if ! command -v lnav &> /dev/null; then
        echo "lnav not found. Please install with:"
        echo "  Ubuntu/Debian: sudo apt-get install lnav"
        echo "  macOS: brew install lnav"
        echo "Will skip lnav benchmarks."
    fi

    # Try to install Python dependencies
    echo "Installing required Python packages..."
    pip3 install pandas matplotlib tabulate psutil &> /dev/null || pip install pandas matplotlib tabulate psutil || echo "Failed to install some Python packages (pandas, matplotlib, tabulate, psutil). Some features may not work."

    # Try to install logreport if available
    pip3 install logreport &> /dev/null || pip install logreport || echo "Failed to install logreport, will skip those benchmarks"

    echo "Dependencies check completed."
}

# Create test datasets
create_datasets() {
    echo "Creating test datasets..."

    # Build the dataset generator
    cargo build --bin create_benchmark_logs

    # Create datasets of different sizes
    cargo run --bin create_benchmark_logs -- 10000 $BENCH_DIR/bench_10k.log
    cargo run --bin create_benchmark_logs -- 100000 $BENCH_DIR/bench_100k.log

    # Only create the large dataset if explicitly requested or not on Windows
    if ! is_windows || [[ "$1" == "--with-large" ]]; then
        echo "Creating large (1M) dataset..."
        cargo run --bin create_benchmark_logs -- 1000000 $BENCH_DIR/bench_1m.log
    else
        echo "Skipping large dataset on Windows. Use --with-large to force creation."
    fi

    echo "Datasets created successfully."
}

# Benchmark function with granular timing and real memory measurement
benchmark() {
    tool=$1
    command=$2
    log=$3
    size=${log##*_}

    abs_log_path=$(cygpath -w "$(pwd)/$BENCH_DIR/$log")
    command_with_abs_path="${command/$BENCH_DIR\/$log/$abs_log_path}"
    escaped_command=$(echo "$command_with_abs_path" | sed 's/\\/\\\\/g')

    if [[ "$tool" == "goaccess" ]] && ! command -v goaccess &> /dev/null; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi
    if [[ "$tool" == "lnav" ]] && ! command -v lnav &> /dev/null; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi
    if [[ "$tool" == "ripgrep" ]] && ! command -v rg &> /dev/null; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi
    if [[ "$tool" == "logreport" ]] && ! python3 -c "import logreport" &> /dev/null 2>&1; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi

    echo "Benchmarking $tool on $size log file..."

    runs=5
    declare -a run_times
    successful_runs=0

    for i in $(seq 1 $runs); do
        echo "  Run $i/$runs..."
        result=$(python3 -c "
import time, subprocess
start = time.time()
result = subprocess.run('$escaped_command', shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
end = time.time()
time_taken = end - start if result.returncode == 0 else -1
print(f'{time_taken}' if result.returncode == 0 else f'-1,{result.stderr}')
")

        runtime=$(echo "$result" | cut -d',' -f1)
        if [[ "$runtime" == "-1" ]]; then
            echo "  Error: Command '$escaped_command' failed."
            echo "  Error details: $(echo "$result" | cut -d',' -f2-)"
        else
            run_times[$successful_runs]=$runtime
            successful_runs=$((successful_runs + 1))
        fi
    done

    if [[ $successful_runs -eq 0 ]]; then
        echo "  All runs failed for $tool on $log. Skipping results."
        return
    fi

    median_time=$(python3 -c "import statistics; print('%.3f' % statistics.median([$(echo "${run_times[*]}" | tr ' ' ',')]))")

    echo "$tool,$log,$median_time" >> $BENCH_DIR/benchmark_results.csv
    echo "  Median time: ${median_time}s"
}

# Consolidated run_benchmarks function
run_benchmarks() {
    echo "Running benchmarks..."

    # Clear previous results
    > $BENCH_DIR/benchmark_results.csv  # Truncate file

    cargo build --release

    log_files=()
    for file in "$BENCH_DIR/bench_10k.log" "$BENCH_DIR/bench_100k.log" "$BENCH_DIR/bench_1m.log"; do
        if [[ -f "$file" ]]; then
            log_files+=("${file##*/}")
            echo "Including log file: ${file##*/}"
        else
            echo "Skipping non-existent log file: ${file##*/}"
        fi
    done

    if [ ${#log_files[@]} -eq 0 ]; then
        echo "No log files found to benchmark!"
        exit 1
    fi

    TIMBER_PATH=$(cygpath -w "$(pwd)/target/release/timber.exe")
    if [[ ! -f "$(cygpath -u "$TIMBER_PATH")" ]]; then
        echo "Error: timber executable not found at $TIMBER_PATH"
        exit 1
    fi

    for log_file in "${log_files[@]}"; do
        benchmark "timber-stats" "$TIMBER_PATH --stats $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-level-ERROR" "$TIMBER_PATH --level ERROR $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-chop-ERROR" "$TIMBER_PATH --chop \"ERROR\" $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-chop-stats" "$TIMBER_PATH --chop \"ERROR\" --stats $BENCH_DIR/$log_file" "$log_file"
        benchmark "grep" "grep -c ERROR $BENCH_DIR/$log_file" "$log_file"

        if command -v rg &> /dev/null; then
            benchmark "ripgrep" "rg -c ERROR $BENCH_DIR/$log_file" "$log_file"
        fi

        if is_windows; then
            benchmark "findstr" "findstr /C:[ERROR] $BENCH_DIR/$log_file" "$log_file"
        else
            benchmark "awk" "awk -F'[][]' '{count[\$2]++} END {for (level in count) print level, count[level]}' $BENCH_DIR/$log_file" "$log_file"
        fi

        if command -v goaccess &> /dev/null; then
            benchmark "goaccess" "goaccess $BENCH_DIR/$log_file -o /dev/null --log-format='%d %t [%^] %^'" "$log_file"
        fi

        if command -v lnav &> /dev/null; then
            benchmark "lnav" "lnav -n -c ';select count(*) from logline;quit' $BENCH_DIR/$log_file" "$log_file"
        fi

        if python3 -c "import logreport" &> /dev/null 2>&1; then
            benchmark "logreport" "python3 -m logreport $BENCH_DIR/$log_file -o /dev/null" "$log_file"
        fi
    done

    echo "Benchmarking complete. Results in $BENCH_DIR/benchmark_results.csv"
}

# Function to generate reports
generate_reports() {
    echo "Generating benchmark reports..."
    mkdir -p $BENCH_DIR/reports

    cat > $BENCH_DIR/generate_charts.py << 'EOF'
import os
import sys
import pandas as pd
import matplotlib.pyplot as plt

BENCHMARK_DIR = "benchmark_data"
RESULTS_FILE = os.path.join(BENCHMARK_DIR, "benchmark_results.csv")
OUTPUT_DIR = os.path.join(BENCHMARK_DIR, "reports")

os.makedirs(OUTPUT_DIR, exist_ok=True)
if not os.path.exists(RESULTS_FILE):
    print(f"Error: Results file {RESULTS_FILE} not found")
    sys.exit(1)

# Read CSV, skip invalid rows
df = pd.read_csv(RESULTS_FILE, header=None, names=['tool', 'log', 'time_seconds'], on_bad_lines='skip')

# Extract file size, handle potential errors
def extract_size(log):
    try:
        return log.split('_')[1].split('.')[0]
    except (IndexError, AttributeError):
        return None

df['size'] = df['log'].apply(extract_size)
df = df.dropna(subset=['size'])  # Drop rows where size couldn't be extracted

# Pivot data for time
time_df = df.pivot(index='size', columns='tool', values='time_seconds')

# Sort by file size
size_order = ['10k', '100k', '1m']
time_df = time_df.reindex([s for s in size_order if s in time_df.index])

# Create bar chart for time
fig, ax = plt.subplots(figsize=(12, 8))
time_df.plot(kind='bar', ax=ax)
ax.set_title('Processing Time by Tool and File Size')
ax.set_xlabel('File Size')
ax.set_ylabel('Time (seconds)')
ax.set_yscale('log')
ax.legend(title='Tool')
ax.grid(True, which='both', linestyle='--', linewidth=0.5)
plt.tight_layout()
plt.savefig(os.path.join(OUTPUT_DIR, 'time_comparison.png'))
plt.close()

# Print summary table
print("Benchmark Results Summary")
print("========================")
for size in time_df.index:
    print(f"\nResults for {size} lines:")
    summary = pd.DataFrame({'Time (s)': time_df.loc[size].sort_values()})
    print(summary.to_string())

print("\nCharts generated in", OUTPUT_DIR)
EOF

    python3 $BENCH_DIR/generate_charts.py || python $BENCH_DIR/generate_charts.py
    echo "Benchmark analysis complete."
}

# Main script
main() {
    echo "=== Timber Benchmarking Tool ==="

    # Clean up old benchmark data
    echo "Cleaning up old benchmark data..."
    rm -rf $BENCH_DIR/*

    # Check for dependencies
    check_dependencies

    # Create datasets
    create_datasets "$@"

    # Run benchmarks
    run_benchmarks

    # Generate reports
    generate_reports

    echo "Benchmarking completed successfully!"
    echo "See results in $BENCH_DIR/reports directory"
}

# Execute the main function with all arguments
main "$@"