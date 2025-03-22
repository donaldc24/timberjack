# Timber 🪓

[![Rust CI](https://github.com/donaldc24/timber/workflows/Rust%20CI/badge.svg)](https://github.com/donaldc24/timber/actions)
[![Crates.io](https://img.shields.io/crates/v/timber.svg)](https://crates.io/crates/timber)

**Timber: Fell Your Logs Fast** - A lightning-fast CLI log analysis tool built in Rust.

## 📋 Overview

Timber is a log-agnostic CLI tool that chops through noise to deliver patterns, trends, and stats from your logs. It's designed to be portable, requiring no servers or complex setup, and works with logs from any source—Java, Rust, Python, or any text-based logs.

## ✨ Features

- **Fast Pattern Search**: Find matches with regex support
- **Log Level Filtering**: Focus on specific severity levels (ERROR, WARN, INFO, etc.)
- **Time-based Trend Analysis**: See how log patterns change over time
- **Statistical Summaries**: Get insights on log levels, error types, and message uniqueness
- **Efficient Processing**: Handles large log files with minimal resource usage

## 🚀 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/your-username/timber.git
cd timber

# Build with Cargo
cargo build --release

# The binary will be in target/release/timber
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

# Combine options
timber --chop "timeout" --level ERROR --trend --stats path/to/logfile.log
```

### Command-Line Options

| Option | Description |
|--------|-------------|
| `--chop <PATTERN>` | Search for log lines matching the given pattern |
| `--level <LEVEL>` | Filter logs by level (ERROR, WARN, INFO, etc.) |
| `--trend` | Show time-based trends of log occurrences |
| `--stats` | Show summary statistics (levels, error types, uniqueness) |
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

## 📝 Roadmap

### Short-term Goals
- [x] Basic log file analysis
- [x] Pattern searching
- [x] Log level filtering
- [x] Time-based trend analysis
- [x] Statistical summaries

### Upcoming Features
- JSON output support
- Advanced format auto-detection
- Multi-file analysis
- Performance optimization
- IDE integrations (VS Code, IntelliJ)
- Interactive TUI mode

### Long-term Vision
- Machine learning-based log insights
- Cloud log aggregation support
- Advanced error correlation
- Distributed log analysis

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.