# Timber 🪓

[![Rust CI](https://github.com/donaldc24/timber/workflows/Rust%20CI/badge.svg)](https://github.com/donaldc24/timber/actions)
[![Crates.io](https://img.shields.io/crates/v/timber-rs.svg)](https://crates.io/crates/timber-rs)

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

```bash
# Basic usage
timber path/to/logfile.log

# Search for a specific pattern
timber --chop "Exception" path/to/logfile.log

# Filter by log level
timber --level ERROR path/to/logfile.log

# Show time-based trends
timber --trend path/to/logfile.log

# Display statistical summary
timber --stats path/to/logfile.log

# Count matching logs (fast mode)
timber --count --chop "Exception" path/to/logfile.log

# Combine options
timber --chop "timeout" --level ERROR --trend --stats path/to/logfile.log

# Force parallel or sequential processing
timber --parallel path/to/logfile.log
timber --sequential path/to/logfile.log

# Get JSON output
timber --json path/to/logfile.log

# Show unique messages
timber --show-unique path/to/logfile.log
```

### Command-Line Options

| Option | Description |
|--------|-------------|
| `--chop <PATTERN>` | Search for log lines matching the given pattern |
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

- [Command Line Interface](docs/cli.md) - Comprehensive guide to all CLI options and examples
- [Performance Optimizations](docs/performance.md) - Technical details on performance features and optimization tips
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