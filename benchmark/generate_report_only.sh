#!/bin/bash
# Script to regenerate only the report for the last benchmark run

# Helper function to check if we're on Windows
is_windows() {
    [[ "$(uname)" =~ "MINGW"|"MSYS"|"CYGWIN" ]] || [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]
}

# Find Python command more effectively, especially for Windows Git Bash
find_python() {
    if is_windows; then
        # Try specific Windows Python locations
        for py_path in \
            "/c/Users/${USER}/AppData/Local/Programs/Python/Python313/python.exe" \
            "/c/Users/${USER}/AppData/Local/Programs/Python/Python312/python.exe" \
            "/c/Users/${USER}/AppData/Local/Programs/Python/Python311/python.exe" \
            "/c/Users/${USER}/AppData/Local/Programs/Python/Python310/python.exe" \
            "/c/Python313/python.exe" \
            "/c/Python312/python.exe" \
            "/c/Python311/python.exe" \
            "/c/Python310/python.exe" \
            "python" \
            "python3"
        do
            if command -v "$py_path" &> /dev/null; then
                echo "$py_path"
                return
            fi
        done

        # Try using py launcher which is common on Windows
        if command -v py &> /dev/null; then
            echo "py"
            return
        fi

        # Look for Python in PATH
        if python --version &> /dev/null; then
            echo "python"
            return
        fi

        echo ""
    else
        # On non-Windows, try standard commands
        if command -v python3 &> /dev/null; then
            echo "python3"
        elif command -v python &> /dev/null; then
            echo "python"
        else
            echo ""
        fi
    fi
}

# Get the latest timestamp directory
LATEST_DIR=$(find benchmark_data/reports -mindepth 1 -maxdepth 1 -type d | sort -r | head -1)

if [ -z "$LATEST_DIR" ]; then
    echo "No benchmark results found."
    exit 1
fi

TIMESTAMP=$(basename "$LATEST_DIR")
echo "Regenerating report for benchmark run: $TIMESTAMP"

# Set up required variables and directories
BENCH_DIR="benchmark_data"
BENCH_RUNS=5  # Or whatever value was used in the original run

# Find Python command
PYTHON_CMD=$(find_python)
if [ -z "$PYTHON_CMD" ]; then
    echo "Error: Python not found. Please ensure Python is installed and in your PATH."
    echo "If Python is installed, specify the full path:"
    echo "PYTHON_CMD=/path/to/python $0"
    exit 1
fi

echo "Using Python command: $PYTHON_CMD"

# Verify Python command works
if ! $PYTHON_CMD --version &> /dev/null; then
    echo "Error: The Python command '$PYTHON_CMD' is not working properly."
    echo "Please specify the correct Python command or path."
    exit 1
fi

# Create the analysis script
cat > $BENCH_DIR/reports/$TIMESTAMP/analyze_benchmarks.py << 'EOFA'
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

# Debug: Print the first few lines of the CSV file
print("Debugging: First few lines of the CSV file:")
with open(results_file, 'r') as f:
    for i, line in enumerate(f):
        print(f"Line {i}: {line.strip()}")
        if i >= 5:
            break
    print("---")

try:
    # Read CSV with column names and force numeric columns to be numeric
    df = pd.read_csv(results_file, dtype={
        "lines": float,
        "time_seconds": float,
        "time_stdev": float,
        "memory_mb": float,
        "memory_stdev": float,
        "throughput": float,
        "outliers": float
    })


    # Print some basic information about the dataframe to help debug
    print("DataFrame info:")
    print(df.info())
    print("\nDataFrame head:")
    print(df.head())

    # Check for any string values in numeric columns
    for col in ["time_seconds", "time_stdev", "memory_mb", "memory_stdev", "throughput", "outliers"]:
        non_numeric = df[~pd.to_numeric(df[col], errors='coerce').notna()]
        if len(non_numeric) > 0:
            print(f"\nWarning: Found non-numeric values in column {col}:")
            print(non_numeric[[col]])

            # Convert to numeric, coercing errors to NaN
            df[col] = pd.to_numeric(df[col], errors='coerce')

    # Add size class column for better grouping
    df['size'] = df['file'].str.extract(r'(\d+[km])')
    size_order = ['10k', '100k', '1m', '10m']
    size_map = {s: i for i, s in enumerate(size_order)}
    df['size_order'] = df['size'].map(size_map)
    df = df.sort_values(['category', 'size_order', 'time_seconds'])

except Exception as e:
    print(f"Error processing CSV file: {e}")
    print("Please check the format of your benchmark_results.csv file")
    sys.exit(1)

# Function to generate comparison charts for a category
def generate_category_charts(category_df, category_name):
    print(f"\nAnalyzing {category_name} benchmarks...")

    # List of size classes in this category
    sizes = category_df['size'].unique()

    # Skip empty size classes
    sizes = [s for s in sizes if pd.notna(s)]

    if not sizes:
        print(f"No valid size classes found for {category_name}")
        return

    for size in sizes:
        size_df = category_df[category_df['size'] == size].copy()

        # Skip if no data
        if size_df.empty:
            print(f"No data for size {size}")
            continue

        # Ensure numeric columns are numeric
        for col in ["time_seconds", "memory_mb", "throughput"]:
            size_df[col] = pd.to_numeric(size_df[col], errors='coerce')

        # Skip if no valid data after conversion
        if size_df['time_seconds'].isna().all():
            print(f"No valid time data for size {size}")
            continue

        # Calculate relative performance compared to fastest tool
        min_time = size_df['time_seconds'].min()
        if pd.isna(min_time) or min_time <= 0:
            print(f"Invalid minimum time for size {size}: {min_time}")
            continue

        size_df['relative_speed'] = min_time / size_df['time_seconds']
        size_df['is_timber'] = size_df['tool'].str.startswith('timber')

        # Sort by time (ascending)
        size_df = size_df.sort_values('time_seconds')

        # Print table of results
        print(f"\nResults for {size} lines ({category_name}):")
        table_df = size_df[['tool', 'time_seconds', 'memory_mb', 'throughput', 'relative_speed']].copy()
        table_df.columns = ['Tool', 'Time (s)', 'Memory (MB)', 'Lines/second', 'Relative Speed']

        # Format columns, safely handling NaN values
        table_df['Time (s)'] = table_df['Time (s)'].apply(lambda x: '{:.4f}'.format(x) if pd.notna(x) else 'N/A')
        table_df['Memory (MB)'] = table_df['Memory (MB)'].apply(lambda x: '{:.2f}'.format(x) if pd.notna(x) else 'N/A')
        table_df['Lines/second'] = table_df['Lines/second'].apply(lambda x: '{:.0f}'.format(x) if pd.notna(x) else 'N/A')
        table_df['Relative Speed'] = table_df['Relative Speed'].apply(lambda x: '{:.2f}x'.format(x) if pd.notna(x) else 'N/A')

        print(tabulate(table_df, headers='keys', tablefmt='grid', showindex=False))

        try:
            # Generate time comparison chart
            plt.figure(figsize=(10, 6))
            # Filter out NaN values for plotting
            plot_df = size_df.dropna(subset=['time_seconds', 'time_stdev'])
            if len(plot_df) == 0:
                print(f"No valid data for time comparison chart for size {size}")
                continue

            bars = plt.barh(
                plot_df['tool'],
                plot_df['time_seconds'],
                xerr=plot_df['time_stdev'],
                color=[('royalblue' if is_timber else 'lightgray') for is_timber in plot_df['is_timber']],
                alpha=0.7
            )
            plt.xlabel('Time (seconds)')
            plt.title(f'{category_name} Performance Comparison ({size} lines)')
            plt.grid(axis='x', linestyle='--', alpha=0.7)

            # Add values on bars
            for i, bar in enumerate(bars):
                plt.text(
                    bar.get_width() + (plot_df['time_stdev'].iloc[i] if pd.notna(plot_df['time_stdev'].iloc[i]) else 0) + 0.01,
                    bar.get_y() + bar.get_height()/2,
                    f"{plot_df['time_seconds'].iloc[i]:.3f}s",
                    va='center'
                )

            plt.tight_layout()
            plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_time.png", dpi=300)
            plt.close()

            # Generate memory comparison chart
            plt.figure(figsize=(10, 6))
            # Filter out NaN values for plotting
            plot_df = size_df.dropna(subset=['memory_mb', 'memory_stdev'])
            if len(plot_df) == 0:
                print(f"No valid data for memory comparison chart for size {size}")
                continue

            bars = plt.barh(
                plot_df['tool'],
                plot_df['memory_mb'],
                xerr=plot_df['memory_stdev'],
                color=[('darkgreen' if is_timber else 'lightgray') for is_timber in plot_df['is_timber']],
                alpha=0.7
            )
            plt.xlabel('Memory (MB)')
            plt.title(f'{category_name} Memory Usage ({size} lines)')
            plt.grid(axis='x', linestyle='--', alpha=0.7)

            # Add values on bars
            for i, bar in enumerate(bars):
                plt.text(
                    bar.get_width() + (plot_df['memory_stdev'].iloc[i] if pd.notna(plot_df['memory_stdev'].iloc[i]) else 0) + 0.01,
                    bar.get_y() + bar.get_height()/2,
                    f"{plot_df['memory_mb'].iloc[i]:.1f}MB",
                    va='center'
                )

            plt.tight_layout()
            plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_memory.png", dpi=300)
            plt.close()

            # Generate throughput comparison chart
            plt.figure(figsize=(10, 6))
            # Filter out NaN values for plotting
            plot_df = size_df.dropna(subset=['throughput'])
            if len(plot_df) == 0:
                print(f"No valid data for throughput comparison chart for size {size}")
                continue

            bars = plt.barh(
                plot_df['tool'],
                plot_df['throughput'],
                color=[('purple' if is_timber else 'lightgray') for is_timber in plot_df['is_timber']],
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
                    f"{plot_df['throughput'].iloc[i]:.0f}",
                    va='center'
                )

            plt.tight_layout()
            plt.savefig(f"{category_name.lower().replace(' ', '_')}_{size}_throughput.png", dpi=300)
            plt.close()

            # Save results to CSV for reference
            size_df.to_csv(f"{category_name.lower().replace(' ', '_')}_{size}_results.csv", index=False)
        except Exception as e:
            print(f"Error generating charts for {size}: {e}")
            import traceback
            traceback.print_exc()

# Generate scaling comparison across file sizes
def generate_scaling_charts(df):
    print("\nAnalyzing scaling behavior across file sizes...")

    # Get all unique tools and categories
    tools = df['tool'].unique()
    categories = df['category'].unique()

    for category in categories:
        category_df = df[df['category'] == category].copy()

        # Skip if fewer than 2 size classes
        valid_sizes = category_df['size'].dropna().unique()
        if len(valid_sizes) < 2:
            print(f"Not enough size classes for scaling analysis in {category}")
            continue

        try:
            # Create scaling chart
            plt.figure(figsize=(12, 8))

            for tool in tools:
                tool_df = category_df[category_df['tool'] == tool]

                # Skip tools with insufficient data
                valid_tool_df = tool_df.dropna(subset=['size_order', 'time_seconds', 'lines'])
                if len(valid_tool_df) < 2:
                    continue

                # Sort by size
                valid_tool_df = valid_tool_df.sort_values('size_order')

                # Plot lines for each tool
                plt.plot(
                    valid_tool_df['lines'],
                    valid_tool_df['time_seconds'],
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
                tool_df = category_df[category_df['tool'] == tool].dropna(subset=['size_order', 'time_seconds', 'lines'])
                tool_df = tool_df.sort_values('lines')

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
            if scaling_data:
                scaling_df = pd.DataFrame(scaling_data)
                print(tabulate(scaling_df, headers='keys', tablefmt='grid', showindex=False))
                scaling_df.to_csv(f"{category.lower()}_scaling_factors.csv", index=False)
            else:
                print(f"No valid scaling data for {category}")
        except Exception as e:
            print(f"Error generating scaling charts for {category}: {e}")
            import traceback
            traceback.print_exc()

# Generate timber-focused analysis
def generate_timber_analysis(df):
    try:
        timber_df = df[df['tool'].str.startswith('timber')].copy()

        if len(timber_df) == 0:
            print("No Timberjack data found for analysis")
            return

        print("\nTimberjack Performance Analysis:")

        # Remove rows with NaN values in key columns
        timber_df = timber_df.dropna(subset=['time_seconds', 'memory_mb', 'throughput'])
        if timber_df.empty:
            print("No valid Timberjack data after filtering NaNs")
            return

        # Group by category and tool
        grouped = timber_df.groupby(['category', 'tool', 'size'])

        # Calculate average performance metrics
        metrics = grouped.agg({
            'time_seconds': 'mean',
            'memory_mb': 'mean',
            'throughput': 'mean'
        }).reset_index()

        # Format the metrics for display
        metrics['time_seconds'] = metrics['time_seconds'].apply(lambda x: '{:.4f}'.format(x) if pd.notna(x) else 'N/A')
        metrics['memory_mb'] = metrics['memory_mb'].apply(lambda x: '{:.2f}'.format(x) if pd.notna(x) else 'N/A')
        metrics['throughput'] = metrics['throughput'].apply(lambda x: '{:.0f}'.format(x) if pd.notna(x) else 'N/A')

        # Print and save the metrics
        print(tabulate(metrics, headers=['Category', 'Tool', 'Size', 'Time (s)', 'Memory (MB)', 'Lines/s'],
                      tablefmt='grid', showindex=False))
        metrics.to_csv('timber_performance_summary.csv', index=False)

        # Try to create feature vs performance visualization
        try:
            # Create pivot table - safely handling possible errors
            feature_perf = pd.pivot_table(
                timber_df,
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

            # Filter to only include tools in feature_order that exist in the data
            feature_perf = feature_perf[feature_perf['tool'].isin(feature_order)]
            if not feature_perf.empty:
                # Create mapping with existing tools
                order_map = {tool: i for i, tool in enumerate(feature_order) if tool in feature_perf['tool'].values}
                if order_map:  # Only proceed if we have mappings
                    feature_perf['order'] = feature_perf['tool'].map(order_map)
                    feature_perf = feature_perf.sort_values('order')

                    # Create a heatmap of features vs performance
                    plt.figure(figsize=(12, 10))

                    # Extract just the data columns (size classes)
                    heatmap_data = feature_perf.drop(['tool', 'order'], axis=1)

                    # Create normalized version for better visualization
                    max_val = heatmap_data.max().max()
                    if pd.notna(max_val) and max_val > 0:
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
                    else:
                        print("Cannot create heatmap - invalid maximum value")
                else:
                    print("No matching tools found for ordering")
            else:
                print("Empty feature performance dataframe")
        except Exception as e:
            print(f"Error generating timber feature heatmap: {e}")
            import traceback
            traceback.print_exc()
    except Exception as e:
        print(f"Error in timber analysis: {e}")
        import traceback
        traceback.print_exc()

# Execute analysis functions
print("Analyzing benchmark results...")

try:
    # Process by category
    for category in df['category'].unique():
        category_df = df[df['category'] == category]
        generate_category_charts(category_df, category)

    # Generate scaling analysis
    generate_scaling_charts(df)

    # Generate Timberjack-specific analysis
    generate_timber_analysis(df)

    print(f"\nAnalysis complete. Charts saved in current directory.")
except Exception as e:
    print(f"Error during analysis: {e}")
    import traceback
    traceback.print_exc()
EOFA

# Run the analysis script
cd $BENCH_DIR/reports/$TIMESTAMP
echo "Running analysis with $PYTHON_CMD..."
$PYTHON_CMD analyze_benchmarks.py

if [ $? -ne 0 ]; then
    echo "Error running Python script."
    echo "For Windows Git Bash users, you might need to specify the Python path directly:"
    echo "Try running this script with: PYTHON_CMD=/c/Python313/python.exe $0"
    echo "Or specify any of these Python commands that might work on your system:"
    echo "  1. python"
    echo "  2. python3"
    echo "  3. py -3"
    echo "  4. /c/Python313/python.exe"
    exit 1
fi

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
            <p>Each test was run $BENCH_RUNS times</p>
        </div>

        <h2>Pattern Matching Performance</h2>
        <div class="chart-container">
            <img src="pattern_10k_time.png" alt="Pattern Matching 10K" onerror="this.style.display='none'">
            <img src="pattern_100k_time.png" alt="Pattern Matching 100K" onerror="this.style.display='none'">
            <img src="pattern_1m_time.png" alt="Pattern Matching 1M" onerror="this.style.display='none'">
        </div>

        <h2>Log Level Filtering Performance</h2>
        <div class="chart-container">
            <img src="level_10k_time.png" alt="Level Filtering 10K" onerror="this.style.display='none'">
            <img src="level_100k_time.png" alt="Level Filtering 100K" onerror="this.style.display='none'">
            <img src="level_1m_time.png" alt="Level Filtering 1M" onerror="this.style.display='none'">
        </div>

        <h2>JSON Processing Performance</h2>
        <div class="chart-container">
            <img src="json_10k_time.png" alt="JSON Processing 10K" onerror="this.style.display='none'">
            <img src="json_100k_time.png" alt="JSON Processing 100K" onerror="this.style.display='none'">
            <img src="json_1m_time.png" alt="JSON Processing 1M" onerror="this.style.display='none'">
        </div>

        <h2>Statistical Analysis Performance</h2>
        <div class="chart-container">
            <img src="stats_10k_time.png" alt="Stats Analysis 10K" onerror="this.style.display='none'">
            <img src="stats_100k_time.png" alt="Stats Analysis 100K" onerror="this.style.display='none'">
            <img src="stats_1m_time.png" alt="Stats Analysis 1M" onerror="this.style.display='none'">
        </div>

        <h2>Scaling Behavior</h2>
        <div class="chart-container">
            <img src="pattern_scaling.png" alt="Pattern Scaling" onerror="this.style.display='none'">
            <img src="json_scaling.png" alt="JSON Scaling" onerror="this.style.display='none'">
            <img src="stats_scaling.png" alt="Stats Scaling" onerror="this.style.display='none'">
        </div>

        <h2>Timberjack Feature Performance</h2>
        <div class="chart-container">
            <img src="timber_feature_performance.png" alt="Feature Performance" onerror="this.style.display='none'">
        </div>
    </div>
</body>
</html>
HTML

echo "View results at: $BENCH_DIR/reports/$TIMESTAMP/index.html"