# Timber Command Line Interface Documentation

## Overview

Timber provides a comprehensive command-line interface for analyzing log files. The CLI is designed to be intuitive and flexible, allowing you to quickly find and analyze patterns in your logs.

## Basic Usage

```
timber [OPTIONS] <FILE>
```

Where `<FILE>` is the path to the log file you want to analyze.

## Options

### Pattern Searching

| Option | Description | Example |
|--------|-------------|---------|
| `--chop <PATTERN>` | Search for log lines matching the given pattern (regex supported) | `timber --chop "Exception" app.log` |

### Filtering

| Option | Description | Example |
|--------|-------------|---------|
| `--level <LEVEL>` | Filter logs by level (ERROR, WARN, INFO, etc.) | `timber --level ERROR app.log` |
| `-f, --field <FIELD=VALUE>` | Filter by field values (can be specified multiple times) | `timber -f service=api -f user_id=12345 app.log` |

### Analysis Options

| Option | Description | Example |
|--------|-------------|---------|
| `--trend` | Show time-based trends of log occurrences | `timber --trend app.log` |
| `--stats` | Show summary statistics (levels, error types, uniqueness) | `timber --stats app.log` |
| `--count` | Only output the total count of matching logs (fast mode) | `timber --count --chop "ERROR" app.log` |
| `--show-unique` | Show unique messages in the output | `timber --stats --show-unique app.log` |
| `--top-errors <N>` | Number of top error types to show (default: 5) | `timber --stats --top-errors 10 app.log` |

### Output Options

| Option | Description | Example |
|--------|-------------|---------|
| `--json` | Output results in JSON format for programmatic use | `timber --stats --json app.log` |

### Performance Options

| Option | Description | Example |
|--------|-------------|---------|
| `--parallel` | Force parallel processing (auto-detected by default) | `timber --parallel large.log` |
| `--sequential` | Force sequential processing (for debugging) | `timber --sequential app.log` |

### Help and Information

| Option | Description | Example |
|--------|-------------|---------|
| `--help` | Display help information | `timber --help` |
| `--version` | Display version information | `timber --version` |

## Combining Options

Options can be combined to perform more complex analyses:

```bash
# Find ERROR log entries containing "timeout" and show stats
timber --chop "timeout" --level ERROR --stats app.log

# Get time trends for WARN level logs
timber --level WARN --trend app.log

# Count ERROR logs related to database
timber --count --level ERROR --chop "database" app.log

# Filter JSON logs by field values
timber --format json -f service=api -f user_id=12345 app.log
```

## Performance Considerations

- The `--count` flag is the fastest way to get a count of matching log entries
- For large files, Timber automatically uses parallel processing unless `--sequential` is specified
- Memory-mapped file processing is always used for efficient handling of large files
- SIMD acceleration is automatically used when available for pattern matching

## Examples

### Basic Pattern Search

```bash
timber --chop "Exception" app.log
```

Output:
```
2025-03-21 14:00:00,123 [ERROR] NullPointerException in WebController.java:42
2025-03-21 14:02:15,456 [ERROR] IllegalArgumentException in UserService.java:87

Felled: 2 logs

Timber finished chopping the log! 🪵
```

### Count Only Mode

```bash
timber --count --level ERROR app.log
```

Output:
```
42
```

### Statistical Analysis

```bash
timber --stats app.log
```

Output:
```
[log entries...]

Felled: 124 logs

Stats summary:

  Log levels:
    ERROR: 12 logs
    WARN: 28 logs
    INFO: 84 logs

  Top error types:
    1. NullPointerException: 5 occurrences
    2. Connection timeout: 3 occurrences
    3. IllegalArgumentException: 2 occurrences
    4. Authentication failed: 1 occurrence
    5. OutOfMemoryError: 1 occurrence

  Unique messages: 87
  Repetition ratio: 29.8%

Timber finished chopping the log! 🪵
```

### Time Trends with JSON Output

```bash
timber --trend --json app.log
```

Output:
```json
{
  "matched_lines": [...],
  "total_count": 124,
  "time_trends": [
    {"timestamp": "2025-03-21 14", "count": 45},
    {"timestamp": "2025-03-21 15", "count": 79}
  ],
  "stats": null,
  "deduplicated": false
}
```