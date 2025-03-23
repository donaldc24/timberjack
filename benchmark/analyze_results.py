#!/usr/bin/env python3
"""
Analyze Timber benchmark results and compare with other tools.
This script generates detailed reports and visualizations from benchmark data.
"""

import os
import sys
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np
from tabulate import tabulate

# Configuration
BENCHMARK_DIR = "benchmark_data"
RESULTS_FILE = os.path.join(BENCHMARK_DIR, "benchmark_results.csv")
OUTPUT_DIR = os.path.join(BENCHMARK_DIR, "reports")

def ensure_dirs():
    """Ensure output directories exist"""
    os.makedirs(OUTPUT_DIR, exist_ok=True)

def load_data():
    """Load benchmark results from CSV file"""
    if not os.path.exists(RESULTS_FILE):
        print(f"Error: Results file {RESULTS_FILE} not found")
        sys.exit(1)

    df = pd.read_csv(RESULTS_FILE)

    # Extract file sizes from log names and convert to numeric values for sorting
    df['size_name'] = df['log'].apply(lambda x: x.split('_')[1].split('.')[0])

    # Create numeric size column for proper sorting
    size_mapping = {'10k': 10_000, '100k': 100_000, '1m': 1_000_000}
    df['size_numeric'] = df['size_name'].map(size_mapping)

    # Sort by tool name and size
    df = df.sort_values(['tool', 'size_numeric'])

    return df

def generate_time_comparison(df):
    """Generate time comparison chart across tools and file sizes"""
    # Pivot data for plotting
    pivot_df = df.pivot(index='size_name', columns='tool', values='time_seconds')

    # Sort by file size
    size_order = ['10k', '100k', '1m']
    pivot_df = pivot_df.reindex(size_order)

    # Plot
    fig, ax = plt.subplots(figsize=(12, 8))
    pivot_df.plot(kind='bar', ax=ax)

    ax.set_title('Processing Time by Tool and File Size', fontsize=16)
    ax.set_xlabel('File Size', fontsize=14)
    ax.set_ylabel('Time (seconds)', fontsize=14)
    ax.set_yscale('log')  # Use log scale for better visibility
    ax.legend(title='Tool', fontsize=12)
    ax.grid(True, which='both', linestyle='--', linewidth=0.5)

    # Add values on top of bars
    for container in ax.containers:
        ax.bar_label(container, fmt='%.2f', fontsize=8)

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, 'time_comparison.png'), dpi=300)
    plt.close()

def generate_memory_comparison(df):
    """Generate memory usage comparison chart across tools and file sizes"""
    # Pivot data for plotting
    pivot_df = df.pivot(index='size_name', columns='tool', values='memory_mb')

    # Sort by file size
    size_order = ['10k', '100k', '1m']
    pivot_df = pivot_df.reindex(size_order)

    # Plot
    fig, ax = plt.subplots(figsize=(12, 8))
    pivot_df.plot(kind='bar', ax=ax)

    ax.set_title('Memory Usage by Tool and File Size', fontsize=16)
    ax.set_xlabel('File Size', fontsize=14)
    ax.set_ylabel('Memory (MB)', fontsize=14)
    ax.set_yscale('log')  # Use log scale for better visibility
    ax.legend(title='Tool', fontsize=12)
    ax.grid(True, which='both', linestyle='--', linewidth=0.5)

    # Add values on top of bars
    for container in ax.containers:
        ax.bar_label(container, fmt='%.1f', fontsize=8)

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, 'memory_comparison.png'), dpi=300)
    plt.close()

def generate_scaling_analysis(df):
    """Generate analysis of how each tool scales with file size"""
    # Create a figure for the scaling analysis
    fig, ax = plt.subplots(figsize=(12, 8))

    tools = df['tool'].unique()
    markers = ['o', 's', '^', 'D', 'v', '<', '>', 'p', '*', 'h', 'H', '+', 'x', 'd', '|']

    # Plot lines for each tool
    for i, tool in enumerate(tools):
        tool_data = df[df['tool'] == tool]
        ax.plot(tool_data['size_numeric'], tool_data['time_seconds'],
                marker=markers[i % len(markers)], label=tool, linewidth=2, markersize=8)

    ax.set_title('Performance Scaling by File Size', fontsize=16)
    ax.set_xlabel('Number of Lines', fontsize=14)
    ax.set_ylabel('Processing Time (seconds)', fontsize=14)
    ax.set_xscale('log')
    ax.set_yscale('log')
    ax.grid(True, which='both', linestyle='--', linewidth=0.5)
    ax.legend(title='Tool', fontsize=12)

    # Add labels for specific points
    size_labels = {10_000: '10k', 100_000: '100k', 1_000_000: '1m'}
    ax.set_xticks(list(size_labels.keys()))
    ax.set_xticklabels(list(size_labels.values()))

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, 'scaling_analysis.png'), dpi=300)
    plt.close()

    # Calculate and save scaling factors
    scaling_data = []

    for tool in tools:
        tool_data = df[df['tool'] == tool].sort_values('size_numeric')

        if len(tool_data) >= 2:
            # Calculate scaling factor between each consecutive file size
            for i in range(1, len(tool_data)):
                size_ratio = tool_data.iloc[i]['size_numeric'] / tool_data.iloc[i-1]['size_numeric']
                time_ratio = tool_data.iloc[i]['time_seconds'] / tool_data.iloc[i-1]['time_seconds']
                scaling_factor = time_ratio / size_ratio

                scaling_data.append({
                    'Tool': tool,
                    'Size Change': f"{tool_data.iloc[i-1]['size_name']} → {tool_data.iloc[i]['size_name']}",
                    'Time Increase': f"{time_ratio:.2f}x",
                    'Scaling Factor': f"{scaling_factor:.3f}"
                })

    # Convert to DataFrame and save
    scaling_df = pd.DataFrame(scaling_data)
    scaling_df.to_csv(os.path.join(OUTPUT_DIR, 'scaling_factors.csv'), index=False)

    # Print scaling analysis
    print("\nScaling Analysis (how processing time increases with file size):")
    print(tabulate(scaling_df, headers='keys', tablefmt='grid'))

def generate_rankings(df):
    """Generate performance rankings for each file size"""
    size_names = df['size_name'].unique()

    for size in size_names:
        size_df = df[df['size_name'] == size].copy()

        # Get rankings for time and memory
        size_df['time_rank'] = size_df['time_seconds'].rank()
        size_df['memory_rank'] = size_df['memory_mb'].rank()

        # Calculate combined score (equally weighted)
        size_df['combined_score'] = (size_df['time_rank'] + size_df['memory_rank']) / 2

        # Sort by combined score
        size_df = size_df.sort_values('combined_score')

        # Select and format columns for output
        result_df = size_df[['tool', 'time_seconds', 'memory_mb', 'time_rank', 'memory_rank', 'combined_score']]
        result_df.columns = ['Tool', 'Time (s)', 'Memory (MB)', 'Time Rank', 'Memory Rank', 'Combined Score']

        # Format numeric columns
        result_df['Time (s)'] = result_df['Time (s)'].map('{:.3f}'.format)
        result_df['Memory (MB)'] = result_df['Memory (MB)'].map('{:.2f}'.format)

        # Save to CSV
        result_df.to_csv(os.path.join(OUTPUT_DIR, f'ranking_{size}.csv'), index=False)

        # Print ranking table
        print(f"\nTool Rankings for {size} lines:")
        print(tabulate(result_df, headers='keys', tablefmt='grid'))

def generate_timber_specific_analysis(df):
    """Generate Timber-specific performance analysis"""
    timber_data = df[df['tool'] == 'timber'].copy()

    if len(timber_data) == 0:
        print("No data for Timber found in the results")
        return

    # Calculate lines processed per second
    timber_data['lines_per_second'] = timber_data['size_numeric'] / timber_data['time_seconds']

    # Calculate memory efficiency (lines per MB)
    timber_data['lines_per_mb'] = timber_data['size_numeric'] / timber_data['memory_mb']

    # Format for output
    timber_analysis = timber_data[['size_name', 'time_seconds', 'memory_mb', 'lines_per_second', 'lines_per_mb']]
    timber_analysis.columns = ['File Size', 'Time (s)', 'Memory (MB)', 'Lines/Second', 'Lines/MB']

    # Format numeric columns
    timber_analysis['Time (s)'] = timber_analysis['Time (s)'].map('{:.3f}'.format)
    timber_analysis['Memory (MB)'] = timber_analysis['Memory (MB)'].map('{:.2f}'.format)
    timber_analysis['Lines/Second'] = timber_analysis['Lines/Second'].map('{:.0f}'.format)
    timber_analysis['Lines/MB'] = timber_analysis['Lines/MB'].map('{:.0f}'.format)

    # Save to CSV
    timber_analysis.to_csv(os.path.join(OUTPUT_DIR, 'timber_analysis.csv'), index=False)

    # Print timber analysis
    print("\nTimber Performance Analysis:")
    print(tabulate(timber_analysis, headers='keys', tablefmt='grid'))

def generate_comparative_analysis(df):
    """Generate comparative analysis between Timber and other tools"""
    # For each file size, calculate the ratio of each tool's time to Timber's time
    size_names = df['size_name'].unique()
    comparative_data = []

    for size in size_names:
        size_df = df[df['size_name'] == size].copy()

        # Get Timber's performance as baseline
        timber_time = size_df[size_df['tool'] == 'timber']['time_seconds'].values
        timber_memory = size_df[size_df['tool'] == 'timber']['memory_mb'].values

        if len(timber_time) == 0:
            continue  # Skip if no Timber data

        timber_time = timber_time[0]
        timber_memory = timber_memory[0]

        # Calculate ratios for each tool
        for _, row in size_df.iterrows():
            if row['tool'] != 'timber':
                time_ratio = timber_time / row['time_seconds']
                memory_ratio = timber_memory / row['memory_mb']

                comparative_data.append({
                    'File Size': size,
                    'Tool': row['tool'],
                    'Time Ratio': time_ratio,  # >1 means tool is faster than Timber
                    'Memory Ratio': memory_ratio,  # >1 means tool uses less memory than Timber
                    'Timber Time (s)': timber_time,
                    'Tool Time (s)': row['time_seconds'],
                    'Timber Memory (MB)': timber_memory,
                    'Tool Memory (MB)': row['memory_mb']
                })

    if not comparative_data:
        print("No comparative data available")
        return

    # Convert to DataFrame
    comparative_df = pd.DataFrame(comparative_data)

    # Add formatted columns for display
    comparative_df['Time Comparison'] = comparative_df.apply(
        lambda x: f"Timber is {1/x['Time Ratio']:.2f}x slower" if x['Time Ratio'] < 1
        else f"Timber is {x['Time Ratio']:.2f}x faster", axis=1
    )

    comparative_df['Memory Comparison'] = comparative_df.apply(
        lambda x: f"Timber uses {1/x['Memory Ratio']:.2f}x more memory" if x['Memory Ratio'] < 1
        else f"Timber uses {x['Memory Ratio']:.2f}x less memory", axis=1
    )

    # Save to CSV
    comparative_df.to_csv(os.path.join(OUTPUT_DIR, 'comparative_analysis.csv'), index=False)

    # Print comparative analysis
    print("\nComparative Analysis (Timber vs. Other Tools):")
    display_df = comparative_df[['File Size', 'Tool', 'Time Comparison', 'Memory Comparison']]
    print(tabulate(display_df, headers='keys', tablefmt='grid'))

    # Create visualization for comparative analysis
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(18, 8))

    # Group by tool and file size, then plot time ratio
    pivot_time = comparative_df.pivot(index='Tool', columns='File Size', values='Time Ratio')
    pivot_time.plot(kind='barh', ax=ax1)
    ax1.set_title('Speed Comparison (Time Ratio)', fontsize=16)
    ax1.set_xlabel('Ratio (>1 means Timber is faster)', fontsize=14)
    ax1.axvline(x=1, color='r', linestyle='--')
    ax1.grid(True, which='both', linestyle='--', linewidth=0.5)

    # Group by tool and file size, then plot memory ratio
    pivot_memory = comparative_df.pivot(index='Tool', columns='File Size', values='Memory Ratio')
    pivot_memory.plot(kind='barh', ax=ax2)
    ax2.set_title('Memory Efficiency Comparison (Memory Ratio)', fontsize=16)
    ax2.set_xlabel('Ratio (>1 means Timber uses less memory)', fontsize=14)
    ax2.axvline(x=1, color='r', linestyle='--')
    ax2.grid(True, which='both', linestyle='--', linewidth=0.5)

    plt.tight_layout()
    plt.savefig(os.path.join(OUTPUT_DIR, 'comparative_analysis.png'), dpi=300)
    plt.close()

def main():
    """Main function to run the analysis"""
    print("Analyzing benchmark results...")
    ensure_dirs()

    # Load data
    df = load_data()

    # Generate visualizations and reports
    generate_time_comparison(df)
    generate_memory_comparison(df)
    generate_scaling_analysis(df)
    generate_rankings(df)
    generate_timber_specific_analysis(df)
    generate_comparative_analysis(df)

    print(f"\nAnalysis complete. Reports saved to {OUTPUT_DIR}")

if __name__ == "__main__":
    main()