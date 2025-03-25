#!/bin/bash
# Enhanced benchmarking script for Timberjack
# Comprehensive performance testing across different log types and operations

export PATH="$PATH:/c/Python313:/c/Python313/Scripts"
set -e  # Exit on error

# Helper function to check if we're on Windows
is_windows() {
    [[ "$(uname)" =~ "MINGW"|"MSYS"|"CYGWIN" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]
}

# Determine the script's directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Benchmark configuration
BENCH_DIR="benchmark_data"
BENCH_RUNS=5           # Number of runs per benchmark
BENCH_WARMUP=true      # Whether to do warmup runs
BENCH_CACHE_MODE="warm"  # "warm" or "cold" cache
BENCH_COOLDOWN=1       # Seconds to wait between benchmark runs
WITH_LARGE=false       # Whether to include large (10M line) tests
SEQUENTIAL_ONLY=false  # Run benchmarks sequentially
PROCESS_NICE=-10       # Process priority (lower = higher priority)
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

mkdir -p $BENCH_DIR
mkdir -p $BENCH_DIR/reports
mkdir -p $BENCH_DIR/reports/$TIMESTAMP

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

    echo "$timestamp,CPU: $cpu_usage%,Memory: $mem_usage%" >> $BENCH_DIR/reports/$TIMESTAMP/system_stats.log
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
        echo "Windows detected. Checking for required tools..."
    fi

    # Check for comparison tools
    for tool in grep rg jq; do
        if ! command -v $tool &> /dev/null; then
            echo "Warning: $tool not found. Some benchmarks will be skipped."
        else
            echo "Found $tool: $(which $tool)"
        fi
    done

    # Install Python dependencies
    echo "Installing required Python packages..."
    pip3 install pandas matplotlib tabulate psutil &> /dev/null || pip install pandas matplotlib tabulate psutil || echo "Failed to install Python packages. Some features may not work."

    echo "Dependencies check completed."
}

# Create test datasets
create_datasets() {
    echo "Creating test datasets..."

    # Build the dataset generator
    cargo build --bin create_benchmark_logs

    # Create plaintext log datasets of different sizes
    echo "Creating plaintext log datasets..."
    cargo run --bin create_benchmark_logs -- 10000 $BENCH_DIR/bench_10k.log
    cargo run --bin create_benchmark_logs -- 100000 $BENCH_DIR/bench_100k.log
    cargo run --bin create_benchmark_logs -- 1000000 $BENCH_DIR/bench_1m.log

    if $WITH_LARGE; then
        echo "Creating large (10M) dataset..."
        cargo run --bin create_benchmark_logs -- 10000000 $BENCH_DIR/bench_10m.log
    fi

    # Create JSON log datasets of different sizes
    echo "Creating JSON log datasets..."
    python benchmark/create_json_logs.py "$BENCH_DIR" "$WITH_LARGE"

    echo "Datasets created successfully."
}

# Benchmark function with granular timing and variance tracking
benchmark() {
    local category=$1
    local tool=$2
    local command=$3
    local file=$4
    local size=${file##*_}

    if [[ -z "$file" || -z "$command" ]]; then
        echo "Skipping benchmark due to missing file or command"
        return
    fi

    # Create an ID for this benchmark
    local benchmark_id="${category}-${tool}-${size}"

    # Skip if tool not installed
    if [[ "$tool" == "jq" ]] && ! command -v jq &> /dev/null; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi
    if [[ "$tool" == "ripgrep" ]] && ! command -v rg &> /dev/null; then
        echo "Skipping $tool benchmark (not installed)"
        return
    fi

    # Prepare absolute path for Windows
    if is_windows; then
        # Get the full path without using pwd, which can cause issues in Git Bash
        local abs_file_path=$(realpath "$file")
        # Use cygpath to convert to Windows path format
        abs_file_path=$(cygpath -w "$abs_file_path")
        # Use echo to print the transformed command for debugging
        echo "Running Windows command: ${command/$file/$abs_file_path}"
        local command_with_path="${command/$file/$abs_file_path}"
    else
        local abs_file_path=$(realpath "$file")
        echo "Running command: ${command/$file/$abs_file_path}"
        local command_with_path="${command/$file/$abs_file_path}"
    fi

    local escaped_command=$(echo "$command_with_path" | sed 's/\\/\\\\/g')

    echo "Benchmarking $tool on $size ($category)..."

    # Prepare file caching based on mode
    if [[ "$BENCH_CACHE_MODE" == "warm" ]]; then
        ensure_cached "$file"
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
    declare -a mem_usages
    successful_runs=0

    for i in $(seq 1 $BENCH_RUNS); do
        echo "  Run $i/$BENCH_RUNS..."

        # Log system state before each run
        log_system_state

        # Run benchmark and capture time and memory
        result=$(python3 -c "
import time, subprocess, psutil, os
start = time.time()
process = psutil.Process(os.getpid())
mem_before = process.memory_info().rss / (1024 * 1024)  # MB
try:
    result = subprocess.run('$escaped_command', shell=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    returncode = result.returncode
    stderr = result.stderr
except Exception as e:
    returncode = -1
    stderr = str(e)
end = time.time()
mem_after = process.memory_info().rss / (1024 * 1024)  # MB
mem_used = mem_after - mem_before
time_taken = end - start if returncode == 0 else -1
print(f'{time_taken},{mem_used}' if returncode == 0 else f'-1,0,{stderr}')
")

        # Parse results
        IFS=',' read -r runtime mem_used error_message <<< "$result"

        if [[ "$runtime" == "-1" ]]; then
            echo "  Error: Command failed."
            echo "  Error details: $error_message"
        else
            run_times[$successful_runs]=$runtime
            mem_usages[$successful_runs]=$mem_used
            successful_runs=$((successful_runs + 1))
        fi

        # Cool down between runs
        cool_down
    done

    if [[ $successful_runs -eq 0 ]]; then
        echo "  All runs failed for $tool on $file. Skipping results."
        return
    fi

    # Calculate statistics with outlier removal
    stats=$(python3 -c "
import statistics, numpy as np
times = [$(echo "${run_times[*]}" | tr ' ' ',')]
mems = [$(echo "${mem_usages[*]}" | tr ' ' ',')]

# Remove outliers (values more than 2 standard deviations from mean)
time_mean = statistics.mean(times)
time_stdev = statistics.stdev(times) if len(times) > 1 else 0
time_filtered = [t for t in times if abs(t - time_mean) <= 2 * time_stdev] or times
time_median = statistics.median(time_filtered)
time_stdev = statistics.stdev(time_filtered) if len(time_filtered) > 1 else 0

# Do the same for memory
mem_mean = statistics.mean(mems)
mem_stdev = statistics.stdev(mems) if len(mems) > 1 else 0
mem_filtered = [m for m in mems if abs(m - mem_mean) <= 2 * mem_stdev] or mems
mem_median = statistics.median(mem_filtered)
mem_stdev = statistics.stdev(mem_filtered) if len(mem_filtered) > 1 else 0

# Output stats
outliers = len(times) - len(time_filtered)
print(f'{time_median:.6f},{time_stdev:.6f},{mem_median:.2f},{mem_stdev:.2f},{outliers}')
")

    # Parse stats
    IFS=',' read -r median_time std_dev_time median_mem std_dev_mem outliers <<< "$stats"

    if [[ "$outliers" -gt 0 ]]; then
        echo "  Median time: ${median_time}s (σ = ${std_dev_time}s, removed $outliers outliers)"
    else
        echo "  Median time: ${median_time}s (σ = ${std_dev_time}s)"
    fi
    echo "  Memory usage: ${median_mem}MB (σ = ${std_dev_mem}MB)"

    # Calculate throughput
    local file_lines=$(wc -l < "$file")
    throughput=$(python -c "print($file_lines / $median_time)")

    # Save results to CSV
    echo "$category,$tool,$file,$file_lines,$median_time,$std_dev_time,$median_mem,$std_dev_mem,$throughput,$outliers" >> $BENCH_DIR/reports/$TIMESTAMP/benchmark_results.csv
}

# Run benchmarks for pattern matching
run_pattern_matching_benchmarks() {
    echo "Running pattern matching benchmarks..."

    for size in "10k" "100k" "1m"; do
        # Skip large files if not enabled
        if [[ "$size" == "10m" ]] && ! $WITH_LARGE; then
            continue
        fi

        log_file="$BENCH_DIR/bench_${size}.log"

        # Timberjack commands
        benchmark "pattern" "timber-chop" "$TIMBER_PATH --chop ERROR $log_file" "$log_file"
        benchmark "pattern" "timber-chop-count" "$TIMBER_PATH --count --chop ERROR $log_file" "$log_file"

        # Comparison tools
        benchmark "pattern" "grep" "grep -c ERROR $log_file" "$log_file"
        if command -v rg &> /dev/null; then
            benchmark "pattern" "ripgrep" "rg -c ERROR $log_file" "$log_file"
        fi
        benchmark "pattern" "awk" "awk \"/ERROR/ {count++} END {print count}\" $log_file" "$log_file"
        done

}

# Run benchmarks for log level filtering
run_level_filtering_benchmarks() {
    echo "Running log level filtering benchmarks..."

    for size in "10k" "100k" "1m"; do
        # Skip large files if not enabled
        if [[ "$size" == "10m" ]] && ! $WITH_LARGE; then
            continue
        fi

        log_file="$BENCH_DIR/bench_${size}.log"

        # Timberjack commands
        benchmark "level" "timber-level" "$TIMBER_PATH --level ERROR $log_file" "$log_file"
        benchmark "level" "timber-level-count" "$TIMBER_PATH --count --level ERROR $log_file" "$log_file"

        # Comparison tools - these don't actually understand log levels but we compare anyway
        benchmark "level" "grep-level" "grep -c \"\\[ERROR\\]\" $log_file" "$log_file"
        if command -v rg &> /dev/null; then
          benchmark "level" "ripgrep-level" "rg -c \"\\[ERROR\\]\" $log_file" "$log_file"
        fi

    done
}

# Run benchmarks for JSON log processing
run_json_benchmarks() {
    echo "Running JSON log processing benchmarks..."

    # Create a temporary Python script for JSON parsing
    local tmp_script="$BENCH_DIR/temp_json_counter.py"
    cat > "$tmp_script" << 'EOF'
import json
import sys

def count_errors(filename):
    count = 0
    with open(filename) as f:
        for line in f:
            try:
                obj = json.loads(line)
                if obj.get('level') == 'ERROR':
                    count += 1
            except:
                pass
    return count

if __name__ == "__main__":
    print(count_errors(sys.argv[1]))
EOF
    chmod +x "$tmp_script"

    for size in "10k" "100k" "1m"; do
        # Skip large files if not enabled
        if [[ "$size" == "10m" ]] && ! $WITH_LARGE; then
            continue
        fi

        json_file="$BENCH_DIR/bench_json_${size}.json"

        # Timberjack commands
        benchmark "json" "timber-json-level" "$TIMBER_PATH --format json -f level=ERROR $json_file" "$json_file"
        benchmark "json" "timber-json-service" "$TIMBER_PATH --format json -f service=api $json_file" "$json_file"
        benchmark "json" "timber-json-multi" "$TIMBER_PATH --format json -f service=api -f level=ERROR $json_file" "$json_file"
        benchmark "json" "timber-json-count" "$TIMBER_PATH --count --format json -f level=ERROR $json_file" "$json_file"

        # Comparison tools
        if command -v jq &> /dev/null; then
            if is_windows; then
                benchmark "json" "jq-simple" "jq \"select(.level==\\\"ERROR\\\")\" $json_file > NUL" "$json_file"
                benchmark "json" "jq-complex" "jq \"select(.level==\\\"ERROR\\\" and .service==\\\"api\\\")\" $json_file > NUL" "$json_file"
            else
                benchmark "json" "jq-simple" "jq \"select(.level==\\\"ERROR\\\")\" $json_file > /dev/null" "$json_file"
                benchmark "json" "jq-complex" "jq \"select(.level==\\\"ERROR\\\" and .service==\\\"api\\\")\" $json_file > /dev/null" "$json_file"
            fi
        fi

        if is_windows; then
            echo "Skipping grep-json benchmark on Windows (known compatibility issue)"
        else
            benchmark "json" "grep-json" "grep -c \"\\\"level\\\": \\\"ERROR\\\"\" $json_file" "$json_file"
        fi

        # Python json parser benchmark - using external script for reliability
        benchmark "json" "python-json" "python \"$tmp_script\" \"$json_file\"" "$json_file"
    done

    # Clean up
    rm -f "$tmp_script"
}

# Run benchmarks for statistical analysis
run_stats_benchmarks() {
    echo "Running statistical analysis benchmarks..."

    # Create temporary Python scripts instead of inline Python
    local text_analyzer="$BENCH_DIR/text_analyzer.py"
    local json_analyzer="$BENCH_DIR/json_analyzer.py"

    # Create Python script for text log analysis
    cat > "$text_analyzer" << 'EOF'
import sys, json, collections, re
from datetime import datetime

def analyze_text_log(file_path):
    levels = collections.Counter()
    timestamps = collections.defaultdict(int)
    error_types = collections.Counter()
    unique_msgs = set()

    with open(file_path) as f:
        for line in f:
            # Extract level
            level_match = re.search(r'\[([A-Z]+)\]', line)
            if level_match:
                levels[level_match.group(1)] += 1

            # Extract timestamp
            ts_match = re.search(r'(\d{4}-\d{2}-\d{2} \d{2})', line)
            if ts_match:
                timestamps[ts_match.group(1)] += 1

            # Extract error type
            if 'ERROR' in line:
                error_match = re.search(r'([A-Za-z]+Exception|[A-Za-z]+Error|Connection timeout)', line)
                if error_match:
                    error_types[error_match.group(1)] += 1

            # Extract message
            msg_match = re.search(r'\[(?:[A-Z]+)\] (.*)', line)
            if msg_match:
                unique_msgs.add(msg_match.group(1))

    print(f'Total log entries: {sum(levels.values())}')
    print(f'Log levels: {dict(levels)}')
    print(f'Unique messages: {len(unique_msgs)}')
    print(f'Error types: {dict(error_types)}')

if __name__ == "__main__":
    if len(sys.argv) > 1:
        analyze_text_log(sys.argv[1])
    else:
        print("No file specified")
EOF

    # Create Python script for JSON log analysis
    cat > "$json_analyzer" << 'EOF'
import sys, json, collections
from datetime import datetime

def analyze_json_log(file_path):
    levels = collections.Counter()
    services = collections.Counter()
    status_codes = collections.Counter()
    error_types = collections.Counter()
    unique_msgs = set()

    with open(file_path) as f:
        for line in f:
            try:
                log = json.loads(line)
                if 'level' in log:
                    levels[log['level']] += 1
                if 'service' in log:
                    services[log['service']] += 1
                if 'status' in log:
                    status_codes[log['status']] += 1
                if 'error' in log and isinstance(log['error'], dict) and 'type' in log['error']:
                    error_types[log['error']['type']] += 1
                if 'message' in log:
                    unique_msgs.add(log['message'])
            except json.JSONDecodeError:
                pass

    print(f'Total log entries: {sum(levels.values())}')
    print(f'Log levels: {dict(levels)}')
    print(f'Services: {dict(services)}')
    print(f'Status codes: {dict(status_codes)}')
    print(f'Unique messages: {len(unique_msgs)}')
    print(f'Error types: {dict(error_types)}')

if __name__ == "__main__":
    if len(sys.argv) > 1:
        analyze_json_log(sys.argv[1])
    else:
        print("No file specified")
EOF

    # Make scripts executable
    chmod +x "$text_analyzer"
    chmod +x "$json_analyzer"

    for size in "10k" "100k" "1m"; do
        # Skip large files if not enabled
        if [[ "$size" == "10m" ]] && ! $WITH_LARGE; then
            continue
        fi

        log_file="$BENCH_DIR/bench_${size}.log"
        json_file="$BENCH_DIR/bench_json_${size}.json"

        # Timberjack commands - plain logs
        benchmark "stats" "timber-stats" "$TIMBER_PATH --stats $log_file" "$log_file"
        benchmark "stats" "timber-stats-level" "$TIMBER_PATH --stats --level ERROR $log_file" "$log_file"
        benchmark "stats" "timber-stats-trend" "$TIMBER_PATH --stats --trend $log_file" "$log_file"

        # Timberjack commands - JSON logs
        benchmark "stats" "timber-json-stats" "$TIMBER_PATH --format json --stats $json_file" "$json_file"
        benchmark "stats" "timber-json-stats-field" "$TIMBER_PATH --format json --stats -f level=ERROR $json_file" "$json_file"

        # Use external Python scripts instead of inline code
        if is_windows; then
            benchmark "stats" "python-stats" "python \"$text_analyzer\" \"$log_file\" > NUL" "$log_file"
            benchmark "stats" "python-json-stats" "python \"$json_analyzer\" \"$json_file\" > NUL" "$json_file"
        else
            benchmark "stats" "python-stats" "python \"$text_analyzer\" \"$log_file\" > /dev/null" "$log_file"
            benchmark "stats" "python-json-stats" "python \"$json_analyzer\" \"$json_file\" > /dev/null" "$json_file"
        fi
    done

    # Clean up
    rm -f "$text_analyzer" "$json_analyzer"
}

# Run benchmarks for large file processing
run_large_file_benchmarks() {
    echo "Running large file processing benchmarks..."

    # Only run if large file tests are enabled
    if ! $WITH_LARGE; then
        echo "Skipping large file benchmarks (use --with-large to enable)"
        return
    fi

    # Test with the largest files
    log_file="$BENCH_DIR/bench_10m.log"
    json_file="$BENCH_DIR/bench_json_10m.json"

    # Sequential vs parallel processing
    benchmark "parallel" "timber-seq" "$TIMBER_PATH --sequential $log_file" "$log_file"
    benchmark "parallel" "timber-par" "$TIMBER_PATH --parallel $log_file" "$log_file"

    # Pattern search with parallel processing
    benchmark "parallel" "timber-seq-chop" "$TIMBER_PATH --sequential --chop ERROR $log_file" "$log_file"
    benchmark "parallel" "timber-par-chop" "$TIMBER_PATH --parallel --chop ERROR $log_file" "$log_file"

    # JSON with parallel processing
    benchmark "parallel" "timber-seq-json" "$TIMBER_PATH --sequential --format json $json_file" "$json_file"
    benchmark "parallel" "timber-par-json" "$TIMBER_PATH --parallel --format json $json_file" "$json_file"

    # Compare to GNU Parallel with grep (if available)
    if command -v parallel &> /dev/null; then
        benchmark "parallel" "grep-parallel" "cat $log_file | parallel --pipe -N1000 grep ERROR | wc -l" "$log_file"
    fi
}

# Generate reports from benchmark results
generate_reports() {
    echo "Generating benchmark reports..."

    # Create analysis script
    cat > "$BENCH_DIR/reports/$TIMESTAMP/analyze_benchmarks.py" << 'EOFA'
import os
import sys
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from tabulate import tabulate

# Load benchmark results
results_file = "benchmark_results.csv"
if not os.path.exists(results_file):
    print(f"Error: Results file {results_file} not found")
    sys.exit(1)

# Read CSV with column names
df = pd.read_csv(results_file, names=[
    "category", "tool", "file", "lines", "time_seconds",
    "time_stdev", "memory_mb", "memory_stdev", "throughput", "outliers"
])

# Add size class column for better grouping
df['size'] = df['file'].str.extract(r'(\d+[km])')
size_order = ['10k', '100k', '1m', '10m']
size_map = {s: i for i, s in enumerate(size_order)}
df['size_order'] = df['size'].map(size_map)
df = df.sort_values(['category', 'size_order', 'time_seconds'])

# Function to generate comparison charts for a category
def generate_category_charts(category_df, category_name):
    print(f"\nAnalyzing {category_name} benchmarks...")

    # List of size classes in this category
    sizes = category_df['size'].unique()

    for size in sizes:
        size_df = category_df[category_df['size'] == size].copy()

        # Calculate relative performance compared to fastest tool
        min_time = size_df['time_seconds'].min()
        size_df['relative_speed'] = min_time / size_df['time_seconds']
        size_df['is_timber'] = size_df['tool'].str.startswith('timber')

        # Sort by time (ascending)
        size_df = size_df.sort_values('time_seconds')

        # Print table of results
        print(f"\nResults for {size} lines ({category_name}):")
        table_df = size_df[['tool', 'time_seconds', 'memory_mb', 'throughput', 'relative_speed']].copy()
        table_df.columns = ['Tool', 'Time (s)', 'Memory (MB)', 'Lines/second', 'Relative Speed']
        table_df['Time (s)'] = table_df['Time (s)'].map('{:.4f}'.format)
        table_df['Memory (MB)'] = table_df['Memory (MB)'].map('{:.2f}'.format)
        table_df['Lines/second'] = table_df['Lines/second'].map('{:.0f}'.format)
        table_df['Relative Speed'] = table_df['Relative Speed'].map('{:.2f}x'.format)
        print(tabulate(table_df, headers='keys', tablefmt='grid', showindex=False))

        # Generate time comparison chart
        plt.figure(figsize=(10, 6))
        bars = plt.barh(
            size_df['tool'],
            size_df['time_seconds'],
            xerr=size_df['time_stdev'],
            color=[('royalblue' if is_timber else 'lightgray') for is_timber in size_df['is_timber']],
            alpha=0.7
        )
        plt.xlabel('Time (seconds)')
        plt.title(f'{category_name} Performance Comparison ({size} lines)')
        plt.grid(axis='x', linestyle='--', alpha=0.7)

        # Add values on bars
        for i, bar in enumerate(bars):
            plt.text(
                bar.get_width() + bar.get_xerr() + 0.01,
                bar.get_y() + bar.get_height()/2,
                f"{size_df['time_seconds'].iloc[i]:.3f}s",
                va='center'
            )

        plt.tight_layout()
        plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_time.png", dpi=300)
        plt.close()

        # Generate memory comparison chart
        plt.figure(figsize=(10, 6))
        bars = plt.barh(
            size_df['tool'],
            size_df['memory_mb'],
            xerr=size_df['memory_stdev'],
            color=[('darkgreen' if is_timber else 'lightgray') for is_timber in size_df['is_timber']],
            alpha=0.7
        )
        plt.xlabel('Memory (MB)')
        plt.title(f'{category_name} Memory Usage ({size} lines)')
        plt.grid(axis='x', linestyle='--', alpha=0.7)

        # Add values on bars
        for i, bar in enumerate(bars):
            plt.text(
              bar.get_width() + bar.get_xerr() + 0.01,
                              bar.get_y() + bar.get_height()/2,
                              f"{size_df['memory_mb'].iloc[i]:.1f}MB",
                              va='center'
                          )

                      plt.tight_layout()
                      plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_memory.png", dpi=300)
                      plt.close()

                      # Generate throughput comparison chart
                      plt.figure(figsize=(10, 6))
                      bars = plt.barh(
                          size_df['tool'],
                          size_df['throughput'],
                          color=[('purple' if is_timber else 'lightgray') for is_timber in size_df['is_timber']],
                          alpha=0.7
                      )
                      plt.xlabel('Lines processed per second')
                      plt.title(f'{category_name} Throughput ({size} lines)')
                      plt.grid(axis='x', linestyle='--', alpha=0.7)

                      # Add values on bars
                      for i, bar in enumerate(bars):
                          plt.text(
                              bar.get_width() + 0.01,
                              bar.get_y() + bar.get_height()/2,
                              f"{size_df['throughput'].iloc[i]:.0f}",
                              va='center'
                          )

                      plt.tight_layout()
                      plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_throughput.png", dpi=300)
                      plt.close()

                      # Save results to CSV for reference
                      size_df.to_csv(f"{category_name.lower().replace(' ', '_')}_{size}_results.csv", index=False)

              # Generate scaling comparison across file sizes
              def generate_scaling_charts(df):
                  print("\nAnalyzing scaling behavior across file sizes...")

                  # Get all unique tools and categories
                  tools = df['tool'].unique()
                  categories = df['category'].unique()

                  for category in categories:
                      category_df = df[df['category'] == category].copy()

                      # Skip if fewer than 2 size classes
                      if len(category_df['size'].unique()) < 2:
                          continue

                      # Create scaling chart
                      plt.figure(figsize=(12, 8))

                      for tool in tools:
                          tool_df = category_df[category_df['tool'] == tool]

                          # Skip tools with insufficient data
                          if len(tool_df) < 2:
                              continue

                          # Sort by size
                          tool_df = tool_df.sort_values('size_order')

                          # Plot lines for each tool
                          plt.plot(
                              tool_df['lines'],
                              tool_df['time_seconds'],
                              'o-',
                              label=tool,
                              linewidth=2,
                              markersize=8
                          )

                      plt.xlabel('File Size (lines)')
                      plt.ylabel('Processing Time (seconds)')
                      plt.title(f'{category} Scaling Behavior')
                      plt.xscale('log')
                      plt.yscale('log')
                      plt.grid(True, which='both', linestyle='--', alpha=0.7)
                      plt.legend(title='Tool')

                      plt.tight_layout()
                      plt.savefig(f"{category.lower()}_scaling.png", dpi=300)
                      plt.close()

                      # Calculate scaling factors
                      print(f"\nScaling factors for {category}:")
                      scaling_data = []

                      for tool in tools:
                          tool_df = category_df[category_df['tool'] == tool].sort_values('lines')

                          if len(tool_df) >= 2:
                              for i in range(1, len(tool_df)):
                                  size_ratio = tool_df['lines'].iloc[i] / tool_df['lines'].iloc[i-1]
                                  time_ratio = tool_df['time_seconds'].iloc[i] / tool_df['time_seconds'].iloc[i-1]
                                  scaling_factor = time_ratio / size_ratio

                                  scaling_data.append({
                                      'Tool': tool,
                                      'Size Change': f"{tool_df['size'].iloc[i-1]} → {tool_df['size'].iloc[i]}",
                                      'Lines Ratio': f"{size_ratio:.1f}x",
                                      'Time Ratio': f"{time_ratio:.2f}x",
                                      'Scaling Factor': f"{scaling_factor:.3f}"
                                  })

                      # Print scaling analysis
                      scaling_df = pd.DataFrame(scaling_data)
                      print(tabulate(scaling_df, headers='keys', tablefmt='grid', showindex=False))
                      scaling_df.to_csv(f"{category.lower()}_scaling_factors.csv", index=False)

              # Generate timber-focused analysis
              def generate_timber_analysis(df):
                  timber_df = df[df['tool'].str.startswith('timber')].copy()

                  if len(timber_df) == 0:
                      print("No Timberjack data found for analysis")
                      return

                  print("\nTimberjack Performance Analysis:")

                  # Group by category and tool
                  grouped = timber_df.groupby(['category', 'tool', 'size'])

                  # Calculate average performance metrics
                  metrics = grouped.agg({
                      'time_seconds': 'mean',
                      'memory_mb': 'mean',
                      'throughput': 'mean'
                  }).reset_index()

                  # Format the metrics for display
                  metrics['time_seconds'] = metrics['time_seconds'].map('{:.4f}'.format)
                  metrics['memory_mb'] = metrics['memory_mb'].map('{:.2f}'.format)
                  metrics['throughput'] = metrics['throughput'].map('{:.0f}'.format)

                  # Print and save the metrics
                  print(tabulate(metrics, headers=['Category', 'Tool', 'Size', 'Time (s)', 'Memory (MB)', 'Lines/s'],
                                tablefmt='grid', showindex=False))
                  metrics.to_csv('timber_performance_summary.csv', index=False)

                  # Create feature vs performance visualization
                  feature_perf = timber_df.pivot_table(
                      index='tool',
                      columns='size',
                      values='throughput',
                      aggfunc='mean'
                  ).reset_index()

                  # Sort by feature complexity (number of operations)
                  feature_order = [
                      'timber-chop-count',
                      'timber-level-count',
                      'timber-chop',
                      'timber-level',
                      'timber-json-count',
                      'timber-json-level',
                      'timber-json-service',
                      'timber-json-multi',
                      'timber-stats',
                      'timber-stats-level',
                      'timber-stats-trend',
                      'timber-json-stats',
                      'timber-json-stats-field'
                  ]

                  # Filter to only include tools in feature_order
                  feature_perf = feature_perf[feature_perf['tool'].isin(feature_order)]
                  feature_perf['order'] = feature_perf['tool'].map({tool: i for i, tool in enumerate(feature_order)})
                  feature_perf = feature_perf.sort_values('order')

                  # Create a heatmap of features vs performance
                  plt.figure(figsize=(12, 10))

                  # Extract just the data columns (size classes)
                  heatmap_data = feature_perf.drop(['tool', 'order'], axis=1)

                  # Create normalized version for better visualization
                  max_val = heatmap_data.max().max()
                  normalized_data = heatmap_data.div(max_val)

                  # Plot heatmap
                  plt.imshow(normalized_data, cmap='viridis', aspect='auto')

                  # Add labels
                  plt.colorbar(label='Relative Throughput')
                  plt.yticks(range(len(feature_perf)), feature_perf['tool'])
                  plt.xticks(range(len(heatmap_data.columns)), heatmap_data.columns, rotation=45)
                  plt.title('Timberjack Feature Performance by File Size')
                  plt.tight_layout()
                  plt.savefig('timber_feature_performance.png', dpi=300)
                  plt.close()

              # Execute analysis functions
              print("Analyzing benchmark results...")

              # Process by category
              for category in df['category'].unique():
                  category_df = df[df['category'] == category]
                  generate_category_charts(category_df, category)

              # Generate scaling analysis
              generate_scaling_charts(df)

              # Generate Timberjack-specific analysis
              generate_timber_analysis(df)

              print(f"\nAnalysis complete. Charts saved in current directory.")
EOFA

                  # Run the analysis script
                  cd $BENCH_DIR/reports/$TIMESTAMP
                  python3 analyze_benchmarks.py || python analyze_benchmarks.py

                  echo "Benchmark reports generated:"
                  echo "  $BENCH_DIR/reports/$TIMESTAMP/"

                  # Create index.html to view results
                  cat > $BENCH_DIR/reports/$TIMESTAMP/index.html << HTML
              <!DOCTYPE html>
              <html>
              <head>
                  <title>Timberjack Benchmark Results</title>
                  <style>
                      body { font-family: Arial, sans-serif; margin: 20px; line-height: 1.6; }
                      h1, h2, h3 { color: #2c3e50; }
                      .container { max-width: 1200px; margin: 0 auto; }
                      .chart-container { margin-bottom: 30px; border: 1px solid #eee; padding: 10px; }
                      img { max-width: 100%; height: auto; }
                      table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }
                      th, td { text-align: left; padding: 8px; border-bottom: 1px solid #ddd; }
                      th { background-color: #f2f2f2; }
                      tr:hover { background-color: #f5f5f5; }
                      .summary { background-color: #f8f9fa; padding: 15px; border-radius: 4px; margin-bottom: 20px; }
                  </style>
              </head>
              <body>
                  <div class="container">
                      <h1>Timberjack Benchmark Results</h1>
                      <div class="summary">
                          <h2>Summary</h2>
                          <p>Benchmark run on $(date)</p>
                          <p>Each test was run $BENCH_RUNS times with $BENCH_CACHE_MODE cache</p>
                      </div>

                      <h2>Pattern Matching Performance</h2>
                      <div class="chart-container">
                          <img src="pattern_10k_time.png" alt="Pattern Matching 10K">
                          <img src="pattern_100k_time.png" alt="Pattern Matching 100K">
                          <img src="pattern_1m_time.png" alt="Pattern Matching 1M">
                      </div>

                      <h2>Log Level Filtering Performance</h2>
                      <div class="chart-container">
                          <img src="level_10k_time.png" alt="Level Filtering 10K">
                          <img src="level_100k_time.png" alt="Level Filtering 100K">
                          <img src="level_1m_time.png" alt="Level Filtering 1M">
                      </div>

                      <h2>JSON Processing Performance</h2>
                      <div class="chart-container">
                          <img src="json_10k_time.png" alt="JSON Processing 10K">
                          <img src="json_100k_time.png" alt="JSON Processing 100K">
                          <img src="json_1m_time.png" alt="JSON Processing 1M">
                      </div>

                      <h2>Statistical Analysis Performance</h2>
                      <div class="chart-container">
                          <img src="stats_10k_time.png" alt="Stats Analysis 10K">
                          <img src="stats_100k_time.png" alt="Stats Analysis 100K">
                          <img src="stats_1m_time.png" alt="Stats Analysis 1M">
                      </div>

                      <h2>Scaling Behavior</h2>
                      <div class="chart-container">
                          <img src="pattern_scaling.png" alt="Pattern Scaling">
                          <img src="json_scaling.png" alt="JSON Scaling">
                          <img src="stats_scaling.png" alt="Stats Scaling">
                      </div>

                      <h2>Timberjack Feature Performance</h2>
                      <div class="chart-container">
                          <img src="timber_feature_performance.png" alt="Feature Performance">
                      </div>
                  </div>
              </body>
              </html>
HTML

                  echo "View results at: $BENCH_DIR/reports/$TIMESTAMP/index.html"
              }

              # Main function to orchestrate the benchmarking
              run_benchmarks() {
                  echo "Running benchmarks..."

                  # Clear previous results
                  echo "category,tool,file,lines,time_seconds,time_stdev,memory_mb,memory_stdev,throughput,outliers" > $BENCH_DIR/reports/$TIMESTAMP/benchmark_results.csv
                  echo "timestamp,cpu,memory" > $BENCH_DIR/reports/$TIMESTAMP/system_stats.log

                  cargo build --release

                  # Find the timber executable
                 if is_windows; then
                     TIMBER_PATH="$(pwd)/target/release/timber.exe"
                     # Convert the path to Windows-style
                     TIMBER_PATH=$(cygpath -w "$TIMBER_PATH")
                 else
                     TIMBER_PATH="$(pwd)/target/release/timber"
                 fi


                  if [ ! -f "$TIMBER_PATH" ]; then
                      echo "Error: timber executable not found at $TIMBER_PATH"
                      exit 1
                  fi

                  # Stabilize system before benchmarking
                  stabilize_system

                  # Run different benchmark categories
                  run_pattern_matching_benchmarks
                  run_level_filtering_benchmarks
                  run_json_benchmarks
                  run_stats_benchmarks

                  # Only run large file benchmarks if enabled
                  if $WITH_LARGE; then
                      run_large_file_benchmarks
                  fi

                  echo "Benchmarking complete. Generating reports..."
                  generate_reports
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
                          --help)
                              echo "Timberjack Benchmark Script"
                              echo ""
                              echo "Usage: $0 [options]"
                              echo ""
                              echo "Options:"
                              echo "  --with-large        Include large file (10M line) tests"
                              echo "  --sequential        Run benchmarks sequentially (no parallel)"
                              echo "  --runs=N            Number of benchmark runs (default: 5)"
                              echo "  --cache=warm|cold   Cache mode (default: warm)"
                              echo "  --cooldown=N        Seconds to wait between runs (default: 1)"
                              echo "  --no-warmup         Skip warmup runs"
                              echo "  --help              Show this help message"
                              exit 0
                              ;;
                          *)
                              echo "Unknown option: $1"
                              echo "Use --help for usage information."
                              exit 1
                              ;;
                      esac
                  done
              }

              # Main script
              main() {
                  echo "=== Timberjack Comprehensive Benchmarking Tool ==="
                  echo "Timestamp: $(date)"

                  # Parse command line arguments
                  parse_args "$@"

                  # Check dependencies
                  check_dependencies

                  # Create test datasets
                  create_datasets

                  # Run benchmarks
                  run_benchmarks

                  echo "Benchmarking completed successfully!"
                  echo "See results in $BENCH_DIR/reports/$TIMESTAMP/"
              }

              # Execute the main function with all arguments
              main "$@"