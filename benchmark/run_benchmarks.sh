#!/bin/bash
export PATH="$PATH:/c/Python313:/c/Python313/Scripts"
set -e  # Exit on error

# Helper function to check if we're on Windows
is_windows() {
    [[ "$(uname)" =~ "MINGW"|"MSYS"|"CYGWIN" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]
}

# Benchmark configuration
BENCH_DIR="benchmark_data"
BENCH_RUNS=5           # Number of runs per benchmark
BENCH_WARMUP=true      # Whether to do warmup runs
BENCH_CACHE_MODE="warm"  # "warm" or "cold" cache
BENCH_COOLDOWN=1       # Seconds to wait between benchmark runs
WITH_LARGE=false       # Whether to include 1M line tests
SEQUENTIAL_ONLY=false  # Run benchmarks sequentially
PROCESS_NICE=-10       # Process priority (lower = higher priority)

mkdir -p $BENCH_DIR
mkdir -p $BENCH_DIR/reports

# Function to stabilize system resources
stabilize_system() {
    echo "Stabilizing system resources..."
    # Give system time to settle background tasks
    sleep 3

    # On Windows, attempt to set process priority
    if is_windows; then
        # Using PowerShell to set priority if possible
        powershell -Command "Get-Process -Id $$ | Set-ProcessPriority -Priority 'High'" 2>/dev/null || true
    else
        # On Linux/macOS, use nice if we have permission
        renice -n $PROCESS_NICE $$ > /dev/null 2>&1 || true
    fi

    echo "System stabilized."
}

# Function to ensure file is cached
ensure_cached() {
    local file=$1
    echo "Ensuring $file is cached..."
    # Read file to bring into cache
    if is_windows; then
        # Using PowerShell to read file
        powershell -Command "Get-Content -Path '$file' -ReadCount 0 > \$null" 2>/dev/null
    else
        # Unix way
        cat "$file" > /dev/null 2>&1
    fi
}

# Function to clear cache (may require admin privileges)
clear_cache() {
    echo "Attempting to clear file system cache..."
    if is_windows; then
        # On Windows with admin rights
        powershell -Command "Write-Output 'Clearing cache...'; Get-ChildItem $BENCH_DIR -Recurse | ForEach-Object { Clear-Content -Path \$_.FullName -Stream 'Zone.Identifier' 2>null }" 2>/dev/null || true
    else
        # On Linux with admin rights
        if [ "$(id -u)" = "0" ]; then
            echo 3 > /proc/sys/vm/drop_caches 2>/dev/null || true
        fi
        sync
    fi
}

# Function to log system state
log_system_state() {
    local timestamp=$(date +"%Y-%m-%d %H:%M:%S")
    local cpu_usage=""
    local mem_usage=""

    if is_windows; then
        cpu_usage=$(powershell -Command "Get-Counter '\Processor(_Total)\% Processor Time' | Select-Object -ExpandProperty CounterSamples | Select-Object -ExpandProperty CookedValue" 2>/dev/null | tr -d '\r')
        mem_usage=$(powershell -Command "Get-Counter '\Memory\% Committed Bytes In Use' | Select-Object -ExpandProperty CounterSamples | Select-Object -ExpandProperty CookedValue" 2>/dev/null | tr -d '\r')
    else
        cpu_usage=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}' 2>/dev/null || echo "N/A")
        mem_usage=$(free | grep Mem | awk '{print $3/$2 * 100.0}' 2>/dev/null || echo "N/A")
    fi

    echo "$timestamp,CPU: $cpu_usage%,Memory: $mem_usage%" >> $BENCH_DIR/system_stats.log
}

# Function to cool down between tests
cool_down() {
    sleep $BENCH_COOLDOWN
}

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
    if $WITH_LARGE || ! is_windows; then
        echo "Creating large (1M) dataset..."
        cargo run --bin create_benchmark_logs -- 1000000 $BENCH_DIR/bench_1m.log
    else
        echo "Skipping large dataset on Windows. Use --with-large to force creation."
    fi

    echo "Datasets created successfully."
}

# Benchmark function with granular timing and variance tracking
benchmark() {
    tool=$1
    command=$2
    log=$3
    size=${log##*_}

    # Prepare absolute path for Windows
    if is_windows; then
        abs_log_path=$(cygpath -w "$(pwd)/$BENCH_DIR/$log")
        command_with_abs_path="${command/$BENCH_DIR\/$log/$abs_log_path}"
    else
        abs_log_path="$(pwd)/$BENCH_DIR/$log"
        command_with_abs_path="${command/$BENCH_DIR\/$log/$abs_log_path}"
    fi

    escaped_command=$(echo "$command_with_abs_path" | sed 's/\\/\\\\/g')

    # Skip if tool not installed
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

    # Prepare file caching based on mode
    if [[ "$BENCH_CACHE_MODE" == "warm" ]]; then
        ensure_cached "$BENCH_DIR/$log"
    elif [[ "$BENCH_CACHE_MODE" == "cold" ]]; then
        clear_cache
    fi

    # Log system state before benchmarking
    log_system_state

    # Warm-up run if enabled
    if $BENCH_WARMUP; then
        echo "  Warming up..."
        eval "$escaped_command" > /dev/null 2>&1 || true
        cool_down
    fi

    # Run actual benchmarks
    declare -a run_times
    successful_runs=0

    for i in $(seq 1 $BENCH_RUNS); do
        echo "  Run $i/$BENCH_RUNS..."

        # Log system state before each run
        log_system_state

        # Run benchmark and capture time
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

        # Cool down between runs
        cool_down
    done

    if [[ $successful_runs -eq 0 ]]; then
        echo "  All runs failed for $tool on $log. Skipping results."
        return
    fi

    # Calculate statistics with outlier removal
    stats=$(python3 -c "
import statistics, numpy as np
times = [$(echo "${run_times[*]}" | tr ' ' ',')]
# Remove outliers (values more than 2 standard deviations from mean)
mean = statistics.mean(times)
stdev = statistics.stdev(times) if len(times) > 1 else 0
filtered_times = [t for t in times if abs(t - mean) <= 2 * stdev] or times
median = statistics.median(filtered_times)
if len(filtered_times) < len(times):
    removed = len(times) - len(filtered_times)
    print(f'{median:.3f},{stdev:.3f},{removed}')
else:
    stdev = statistics.stdev(filtered_times) if len(filtered_times) > 1 else 0
    print(f'{median:.3f},{stdev:.3f},0')
")

    median_time=$(echo "$stats" | cut -d',' -f1)
    std_dev=$(echo "$stats" | cut -d',' -f2)
    outliers=$(echo "$stats" | cut -d',' -f3)

    if [[ "$outliers" -gt 0 ]]; then
        echo "  Median time: ${median_time}s (σ = ${std_dev}s, removed $outliers outliers)"
    else
        echo "  Median time: ${median_time}s (σ = ${std_dev}s)"
    fi

    # Save results with more statistics
    echo "$tool,$log,$median_time,$std_dev,$outliers" >> $BENCH_DIR/benchmark_results.csv
}

# Consolidated run_benchmarks function
run_benchmarks() {
    echo "Running benchmarks..."

    # Clear previous results
    echo "tool,log,time_seconds,std_dev,outliers" > $BENCH_DIR/benchmark_results.csv
    echo "timestamp,cpu,memory" > $BENCH_DIR/system_stats.log

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

    # Stabilize system before benchmarking
    stabilize_system

    for log_file in "${log_files[@]}"; do
        echo "Starting benchmarks for $log_file..."

        # Run all timber variants
        benchmark "timber-stats" "$TIMBER_PATH --stats $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-level-ERROR" "$TIMBER_PATH --level ERROR $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-chop-ERROR" "$TIMBER_PATH --chop \"ERROR\" $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-chop-stats" "$TIMBER_PATH --chop \"ERROR\" --stats $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-chop-count" "$TIMBER_PATH --chop \"ERROR\" --count $BENCH_DIR/$log_file" "$log_file"
        benchmark "timber-level-count" "$TIMBER_PATH --level \"ERROR\" --count $BENCH_DIR/$log_file" "$log_file"

        # Run comparison tools
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

        # Extra stabilization between file sizes
        echo "Completed benchmarks for $log_file"
        cool_down
        cool_down
        cool_down
    done

    echo "Benchmarking complete. Results in $BENCH_DIR/benchmark_results.csv"
}

# Function to generate reports
generate_reports() {
    echo "Generating benchmark reports..."

    cat > $BENCH_DIR/generate_charts.py << 'EOF'
import os
import sys
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from tabulate import tabulate

BENCHMARK_DIR = "benchmark_data"
RESULTS_FILE = os.path.join(BENCHMARK_DIR, "benchmark_results.csv")
OUTPUT_DIR = os.path.join(BENCHMARK_DIR, "reports")

os.makedirs(OUTPUT_DIR, exist_ok=True)
if not os.path.exists(RESULTS_FILE):
    print(f"Error: Results file {RESULTS_FILE} not found")
    sys.exit(1)

# Read CSV with all data including standard deviation
df = pd.read_csv(RESULTS_FILE)

# Convert log file names to sizes for sorting
df['size'] = df['log'].str.extract(r'(\d+k|\d+m)').iloc[:, 0]
size_order = ['10k', '100k', '1m']
size_map = {s: i for i, s in enumerate(size_order)}
df['size_ord'] = df['size'].map(size_map)
df = df.sort_values(['size_ord', 'time_seconds'])

# Process data by file size
for size in size_order:
    if not df['size'].str.contains(size).any():
        continue

    size_df = df[df['size'] == size].sort_values('time_seconds')

    # Create nicely formatted output table
    print(f"\nResults for {size} lines:")
    table_df = size_df[['tool', 'time_seconds']].copy()
    print(tabulate(table_df, headers='keys', tablefmt='plain', showindex=False))

    # Save to CSV for reference
    table_df.to_csv(os.path.join(OUTPUT_DIR, f'results_{size}.csv'), index=False)

    # Plot with error bars
    plt.figure(figsize=(10, 6))
    plt.barh(size_df['tool'], size_df['time_seconds'],
             xerr=size_df['std_dev'],
             color='skyblue',
             alpha=0.7)
    plt.xlabel('Time (seconds)')
    plt.title(f'Performance Comparison ({size} lines)')
    plt.grid(axis='x', linestyle='--', alpha=0.7)

    # Add values on bars
    for i, v in enumerate(size_df['time_seconds']):
        plt.text(v + 0.01, i, f"{v:.3f}s", va='center')

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, f'performance_{size}.png'), dpi=300)
    plt.close()

# Create scaling chart
pivot_df = df.pivot_table(index='size_ord', columns='tool', values='time_seconds')
pivot_df.index = [size_order[i] for i in pivot_df.index]

# Plot scaling chart
plt.figure(figsize=(12, 8))
for tool in pivot_df.columns:
    if tool in pivot_df.columns:
        plt.plot(pivot_df.index, pivot_df[tool], marker='o', label=tool, linewidth=2)

plt.xlabel('Log Size')
plt.ylabel('Time (seconds)')
plt.title('Performance Scaling by File Size')
plt.legend()
plt.grid(True, linestyle='--', alpha=0.7)
plt.yscale('log')
plt.tight_layout()
plt.savefig(os.path.join(OUTPUT_DIR, 'scaling_comparison.png'), dpi=300)
plt.close()

# Create variance analysis
plt.figure(figsize=(12, 8))
for size in size_order:
    if not df['size'].str.contains(size).any():
        continue

    size_df = df[df['size'] == size].copy()
    plt.scatter(size_df['time_seconds'], size_df['std_dev'],
                alpha=0.7, label=size, s=100)

    # Annotate points with tool names
    for i, row in size_df.iterrows():
        plt.annotate(row['tool'],
                    (row['time_seconds'], row['std_dev']),
                    xytext=(5, 5),
                    textcoords='offset points')

plt.xlabel('Time (seconds)')
plt.ylabel('Standard Deviation (seconds)')
plt.title('Time vs. Variance')
plt.legend()
plt.grid(True, linestyle='--', alpha=0.7)
plt.tight_layout()
plt.savefig(os.path.join(OUTPUT_DIR, 'variance_analysis.png'), dpi=300)
plt.close()

# System stats visualization if file exists
system_stats_file = os.path.join(BENCHMARK_DIR, "system_stats.log")
if os.path.exists(system_stats_file):
    try:
        # Parse system stats
        sys_stats = pd.read_csv(system_stats_file)
        sys_stats['timestamp'] = pd.to_datetime(sys_stats['timestamp'])

        # Extract numeric values from CPU and memory columns
        sys_stats['cpu_value'] = sys_stats['cpu'].str.extract(r'(\d+\.?\d*)').astype(float)
        sys_stats['memory_value'] = sys_stats['memory'].str.extract(r'(\d+\.?\d*)').astype(float)

        # Create timeline chart
        plt.figure(figsize=(12, 6))
        plt.subplot(2, 1, 1)
        plt.plot(sys_stats.index, sys_stats['cpu_value'], 'r-', label='CPU Usage (%)')
        plt.title('System Resource Timeline During Benchmarks')
        plt.ylabel('CPU Usage (%)')
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.legend()

        plt.subplot(2, 1, 2)
        plt.plot(sys_stats.index, sys_stats['memory_value'], 'b-', label='Memory Usage (%)')
        plt.xlabel('Benchmark Progress')
        plt.ylabel('Memory Usage (%)')
        plt.grid(True, linestyle='--', alpha=0.7)
        plt.legend()

        plt.tight_layout()
        plt.savefig(os.path.join(OUTPUT_DIR, 'system_resources.png'), dpi=300)
        plt.close()
    except Exception as e:
        print(f"Could not generate system stats visualization: {e}")

print("Charts generated in", OUTPUT_DIR)
EOF

    python3 $BENCH_DIR/generate_charts.py || python $BENCH_DIR/generate_charts.py
    echo "Benchmark analysis complete."
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --with-large)
                WITH_LARGE=true
                shift
                ;;
            --sequential)
                SEQUENTIAL_ONLY=true
                shift
                ;;
            --runs=*)
                BENCH_RUNS="${1#*=}"
                shift
                ;;
            --cache=*)
                BENCH_CACHE_MODE="${1#*=}"
                shift
                ;;
            --cooldown=*)
                BENCH_COOLDOWN="${1#*=}"
                shift
                ;;
            --no-warmup)
                BENCH_WARMUP=false
                shift
                ;;
            *)
                echo "Unknown option: $1"
                echo "Valid options: --with-large, --sequential, --runs=N, --cache=[warm|cold], --cooldown=N, --no-warmup"
                exit 1
                ;;
        esac
    done
}

# Main script
main() {
    echo "=== Timber Benchmarking Tool ==="

    # Parse command line arguments
    parse_args "$@"

    # Clean up old benchmark data
    echo "Cleaning up old benchmark data..."
    rm -rf $BENCH_DIR/*

    # Check for dependencies
    check_dependencies

    # Create datasets
    if $WITH_LARGE; then
        create_datasets --with-large
    else
        create_datasets
    fi

    # Run benchmarks
    run_benchmarks

    # Generate reports
    generate_reports

    echo "Benchmarking completed successfully!"
    echo "See results in $BENCH_DIR/reports directory"
}

# Execute the main function with all arguments
main "$@"