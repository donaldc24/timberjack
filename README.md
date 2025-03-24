# Timber 🪓

[![Rust CI](https://github.com/donaldc24/timber/workflows/Rust%20CI/badge.svg)](https://github.com/donaldc24/timber/actions)
[![Crates.io](https://img.shields.io/crates/v/timber-rs.svg)](https://crates.io/crates/timber-rs)
[![Security Audit](https://github.com/donaldc24/timber/workflows/Security%20audit/badge.svg)](https://github.com/donaldc24/timber/actions?query=workflow%3A%22Security+audit%22)

**Timber: Fell Your Logs Fast** - A lightning-fast CLI log analysis tool built in Rust.

## 📋 Overview

Timber is a log-agnostic CLI tool that chops through noise to deliver patterns, trends, and stats from your logs. It's designed to be portable, requiring no servers or complex setup, and works with logs from any source—Java, Rust, Python, or any text-based logs.

## ✨ Features

- **Fast Pattern Search**: Find matches with regex support and SIMD acceleration
- **Log Level Filtering**: Focus on specific severity levels (ERROR, WARN, INFO, etc.)
- **Time-based Trend Analysis**: See how log patterns change over time
- **Statistical Summaries**: Get insights on log levels, error types, and message uniqueness
- **Efficient Processing**: Handles large log files with minimal resource usage
- **High Performance**: Competitive with specialized tools like grep and ripgrep
- **Memory-Mapped Processing**: Efficient handling of large files
- **Parallel Processing**: Automatic multi-threading for large files

## 🚀 Installation

### Cargo (Recommended)
```bash
cargo install timber-rs
```

### From Source
```bash
# Clone the repository
git clone https://github.com/donaldc24/timber.git
cd timber

# Build with Cargo
cargo build --release

# Install locally
cargo install --path .
```

## 🔨 Usage

### Basic Examples

```bash
# Basic usage - view all log entries
timber path/to/logfile.log

# Search for a specific pattern (regex supported)
timber --chop "Exception" path/to/logfile.log

# Filter by log level
timber --level ERROR path/to/logfile.log

# Show time-based trends
timber --trend path/to/logfile.log

# Display statistical summary
timber --stats path/to/logfile.log
```

### Advanced Examples

```bash
# Count matching logs (fast mode)
timber --count --chop "Exception" path/to/logfile.log

# Combine pattern search with level filtering
timber --chop "timeout|connection refused" --level ERROR path/to/logfile.log

# Comprehensive analysis with trends and statistics
timber --chop "database" --level ERROR --trend --stats path/to/logfile.log

# Analyze a file with explicit parallel processing
timber --parallel --stats large_logfile.log

# View detailed error statistics with more top errors
timber --stats --top-errors 10 path/to/logfile.log

# Show unique messages in the stats output
timber --stats --show-unique path/to/logfile.log
```

### JSON Output

```bash
# Get basic JSON output
timber --json path/to/logfile.log

# Get JSON output with statistics
timber --stats --json path/to/logfile.log

# Filter and get JSON for programmatic use
timber --chop "Exception" --level ERROR --json path/to/logfile.log > errors.json

# Pipe JSON to jq for further processing
timber --stats --json path/to/logfile.log | jq '.stats.error_types'

# Count with JSON output
timber --count --chop "ERROR" --json path/to/logfile.log
```

Example JSON output for `--stats --json`:

```json
{
  "matched_lines": [
    "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42",
    "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86"
  ],
  "total_count": 2,
  "time_trends": null,
  "stats": {
    "log_levels": [
      {"level": "ERROR", "count": 2}
    ],
    "error_types": [
      {"error_type": "NullPointerException", "count": 1, "rank": 1},
      {"error_type": "Connection timeout", "count": 1, "rank": 2}
    ],
    "unique_messages_count": 2,
    "repetition_ratio": 0.0,
    "unique_messages": null
  },
  "deduplicated": false
}
```

### Command-Line Options

| Option | Description |
|--------|-------------|
| `--chop <PATTERN>` | Search for log lines matching the given pattern (regex supported) |
| `--level <LEVEL>` | Filter logs by level (ERROR, WARN, INFO, etc.) |
| `--trend` | Show time-based trends of log occurrences |
| `--stats` | Show summary statistics (levels, error types, uniqueness) |
| `--count` | Only output the total count of matching logs (fast mode) |
| `--json` | Output results in JSON format for programmatic use |
| `--show-unique` | Show unique messages in the output |
| `--top-errors <N>` | Number of top error types to show (default: 5) |
| `--parallel` | Force parallel processing (auto-detected by default) |
| `--sequential` | Force sequential processing (for debugging) |
| `--help` | Display help information |
| `--version` | Display version information |

## 📊 Example Output

### Pattern Search

```
2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42
2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86

Felled: 2 logs

Timber finished chopping the log! 🪵
```

### With Stats

```
2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42
2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86

Felled: 2 logs

Stats summary:

  Log levels:
    ERROR: 2 logs

  Top error types:
    1. NullPointerException: 1 occurrence
    2. Connection timeout: 1 occurrence

  Unique messages: 2
  Repetition ratio: 0.0%

Timber finished chopping the log! 🪵
```

### With Time Trends

```
2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42
2025-03-21 15:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86

Felled: 2 logs

Time trends:
  2025-03-21 14 - 1 log occurred during this hour
  2025-03-21 15 - 1 log occurred during this hour

Timber finished chopping the log! 🪵
```

### Count Mode

```bash
# Count all logs
$ timber --count app.log
1245

# Count ERROR logs
$ timber --count --level ERROR app.log
42

# Count pattern matches
$ timber --count --chop "NullPointerException" app.log
5
```

### JSON Output

```bash
# JSON output with stats
$ timber --stats --json app.log

{
  "matched_lines": [
    "2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42",
    "2025-03-21 14:03:00,012 [ERROR] Connection timeout in NetworkClient.java:86"
  ],
  "total_count": 2,
  "time_trends": null,
  "stats": {
    "log_levels": [
      {"level": "ERROR", "count": 2}
    ],
    "error_types": [
      {"error_type": "NullPointerException", "count": 1, "rank": 1},
      {"error_type": "Connection timeout", "count": 1, "rank": 2}
    ],
    "unique_messages_count": 2,
    "repetition_ratio": 0.0,
    "unique_messages": null
  },
  "deduplicated": false
}
```

### JSON With Unique Messages

```bash
# Include unique messages in JSON output
$ timber --stats --show-unique --json app.log

{
  "matched_lines": [...],
  "total_count": 2,
  "stats": {
    "log_levels": [...],
    "error_types": [...],
    "unique_messages_count": 2,
    "repetition_ratio": 0.0,
    "unique_messages": [
      "NullPointerException in WebController.java:42",
      "Connection timeout in NetworkClient.java:86"
    ]
  },
  "deduplicated": false
}
```

### Count Only Mode

```
2
```

## ⚡ Performance

Timber is designed for speed and efficiency:

- **Memory-mapped file processing**: Fast access to files of any size
- **SIMD acceleration**: Uses CPU vector instructions for faster pattern matching
- **Parallel processing**: Automatically uses multiple cores for large files
- **Smart deduplication**: Efficiently handles repeated log lines

### Benchmarks

| Operation | 10K lines | 100K lines | 1M lines |
|-----------|-----------|------------|----------|
| timber --chop-count | 0.164s | 0.181s | 0.401s |
| grep | 0.166s | 0.181s | 0.296s |
| ripgrep | 0.198s | 0.183s | 0.236s |
| timber --level-count | 0.167s | 0.199s | 0.487s |
| timber --chop | 0.169s | 0.239s | 0.640s |
| timber --stats | 0.258s | 0.444s | 2.735s |

For counting and pattern matching operations, Timber is competitive with specialized tools like grep and ripgrep while providing much richer analysis capabilities.

## 📚 Documentation

- [Command Line Interface](docs/CLI.md) - Comprehensive guide to all CLI options and examples
- [Performance Optimizations](docs/PERFORMANCE.md) - Technical details on performance features and optimization tips
- [CHANGELOG](CHANGELOG.md) - Detailed version history and changes

## 📝 Roadmap

### Short-term Goals
- [x] Basic log file analysis
- [x] Pattern searching
- [x] Log level filtering
- [x] Time-based trend analysis
- [x] Statistical summaries
- [x] Memory-mapped file processing
- [x] SIMD acceleration
- [x] Parallel processing
- [x] Count mode

### Upcoming Features
- Format-specific parsers
- Package manager distributions
- VS Code extension
- Multi-file analysis
- Interactive TUI mode

### Long-term Vision
- Advanced error correlation
- Root cause suggestions
- Pattern identification
- Cloud log aggregation support
- Advanced visualization

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) before getting started.

## 🐛 Reporting Issues

Found a bug or have a feature request? Please open an issue on our [GitHub Issues](https://github.com/donaldc24/timber/issues) page.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🌟 Acknowledgments

Inspired by the need for fast, efficient log analysis in modern software development.