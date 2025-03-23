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
