# Timberjack 🪓

[![Rust CI](https://github.com/donaldc24/timberjack/workflows/Rust%20CI/badge.svg)](https://github.com/donaldc24/timberjack/actions)
[![Crates.io](https://img.shields.io/crates/v/timberjack.svg)](https://crates.io/crates/timberjack)
[![Security Audit](https://github.com/donaldc24/timberjack/workflows/Security%20audit/badge.svg)](https://github.com/donaldc24/timberjack/actions?query=workflow%3A%22Security+audit%22)

**Timberjack: Fell Your Logs Fast** - A lightning-fast CLI log analysis tool built in Rust.

## 📋 Overview

Timberjack is a log-agnostic CLI tool that chops through noise to deliver patterns, trends, and stats from your logs. It's designed to be portable, requiring no servers or complex setup, and works with logs from any source—Java, Rust, Python, or any text-based logs.

## ✨ Key Features

- **Fast JSON Processing**: 70-90x faster than jq for large JSON logs
- **Pattern Matching**: On par with grep for small files, with richer analysis capabilities
- **Parallel Processing**: Up to 57% faster analysis of large files with automatic multi-threading
- **Statistical Analysis**: Get insights on log levels, error types, and message uniqueness
- **Memory Efficiency**: Low memory footprint even with large log files
- **Automatic Format Detection**: Intelligently handles both plaintext and JSON logs
- **Stdin Support**: Seamless piping and streaming log analysis

## 🚀 Installation

```bash
cargo install timberjack
```

## 🌐 Stdin and Piping Superpowers

Timberjack now supports stdin input, making log analysis incredibly flexible:

```bash
# Pipe logs from any source
cat system.log | timber --level ERROR
docker logs mycontainer | timber --chop "connection"
journalctl | timber --stats

# Quick counting
cat large_log.log | timber --count
cat access.log | grep "404" | timber --count

# Advanced analysis
kubectl logs pod/web-server | timber --json --trend
```

**Pro Tips:**
- Automatically detects log formats from stdin
- Supports compressed logs via zcat/gunzip
- Memory-mapped processing for large input streams
- Works with JSON, plaintext, and mixed log formats

## 🔨 Quick Examples

```bash
# Basic log analysis with automatic format detection
timber path/to/logfile.log

# Find errors and show statistics
timber --level ERROR --stats app.log

# Analyze JSON logs and filter by specific fields
timber --format json -f service=api -f status=500 logs.json

# Get just the count of errors (fastest mode)
timber --count --level ERROR app.log

# Use parallel processing for large files
timber --parallel large_logfile.log
```

## 📊 Benchmark Results

Our latest benchmarks show impressive performance across various operations:

### JSON Processing (vs jq)
| File Size | Timberjack | jq | Speedup |
|-----------|------------|-----|---------|
| 10K lines | 0.074s | 0.814s | **11x faster** |
| 100K lines | 0.156s | 7.149s | **46x faster** |
| 1M lines | 0.967s | 69.529s | **72x faster** |

### Pattern Matching
| File Size | timber-chop-count | grep | ripgrep |
|-----------|------------------|------|---------|
| 10K lines | 0.069s | 0.072s | 0.138s |
| 100K lines | 0.092s | 0.079s | 0.112s |
| 1M lines | 0.405s | 0.128s | 0.140s |

### Parallel Processing (10M lines)
| Operation | Sequential | Parallel | Improvement |
|-----------|------------|----------|-------------|
| Standard Processing | 10.609s | 7.157s | **32% faster** |
| Pattern Matching | 6.541s | 3.514s | **46% faster** |
| JSON Processing | 30.603s | 12.976s | **58% faster** |

## 📚 Detailed Usage

### Pattern Searching

```bash
# Find logs containing "Exception"
timber --chop "Exception" app.log

# Count occurrences (fastest)
timber --count --chop "Exception" app.log

# Combine with level filtering
timber --chop "timeout" --level ERROR app.log
```

### JSON Log Analysis

```bash
# Process JSON logs (auto-detected)
timber app.json

# Filter by JSON fields
timber --format json -f service=api app.json

# Complex field filtering
timber --format json -f service=api -f level=ERROR -f "response_time>1000" app.json
```

### Statistical Analysis

```bash
# Get comprehensive statistics
timber --stats app.log

# Show time-based trends
timber --trend app.log

# Show unique messages in stats
timber --stats --show-unique app.log

# Output top 10 error types
timber --stats --top-errors 10 app.log
```

### Performance Options

```bash
# Force parallel processing for large files
timber --parallel large.log

# Memory-efficient mode
timber --count large.log

# Output JSON for programmatic use
timber --stats --json app.log > analysis.json
```

## 🔍 When to Use Timberjack

- **Complex JSON Logs**: Timberjack outperforms specialized tools like jq by 46-72x on large files
- **All-in-One Analysis**: Replaces grep, jq, and custom scripts with one unified tool
- **Large Log Files**: Automatic parallelization for multi-gigabyte logs
- **Statistical Insights**: When you need more than just matching lines
- **CI/CD Pipelines**: Fast and reliable log analysis in automated environments

## 🛠️ Command-Line Options

| Option | Description |
|--------|-------------|
| `--chop <PATTERN>` | Search for logs matching pattern (regex supported) |
| `--level <LEVEL>` | Filter by log level (ERROR, WARN, INFO, etc.) |
| `--trend` | Show time-based trends of log occurrences |
| `--stats` | Show summary statistics |
| `--count` | Output only the count (faster) |
| `--format <FORMAT>` | Specify log format (auto, json) |
| `-f, --field <FIELD=VALUE>` | Filter by field value (for JSON logs) |
| `--parallel` | Force parallel processing |
| `--json` | Output results in JSON format |
| `--top-errors <N>` | Number of top error types to show (default: 5) |
| `--show-unique` | Show unique messages in stats output |

## 📝 Roadmap

- VS Code extension (coming May 2025)
- Multi-file analysis
- Interactive TUI mode
- Advanced pattern correlation

## 🤝 Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## 📄 License

Timberjack is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.