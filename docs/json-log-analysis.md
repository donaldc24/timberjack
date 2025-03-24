# JSON Log Analysis in Timberjack

Timberjack provides advanced support for analyzing JSON-formatted logs. This document covers all aspects of JSON log processing, from basic usage to advanced filtering techniques.

## Table of Contents
- [Overview](#overview)
- [Automatic Format Detection](#automatic-format-detection)
- [Basic Usage](#basic-usage)
- [Field-Based Filtering](#field-based-filtering)
- [Working with Nested JSON](#working-with-nested-json)
- [JSON Output](#json-output)
- [Performance Considerations](#performance-considerations)
- [Common Use Cases](#common-use-cases)
- [Troubleshooting](#troubleshooting)

## Overview

Modern applications frequently use JSON-formatted logs due to their structured nature and machine readability. Timberjack offers specialized capabilities for working with such logs:

- Automatic detection of JSON formatted logs
- Field extraction and filtering
- Support for nested JSON structures
- Efficient processing of large JSON log files
- JSON output for programmatic analysis

## Automatic Format Detection

Timberjack will automatically detect JSON-formatted logs by examining the first few lines of your log file. The detection algorithm looks for:

1. Lines that begin with `{` and end with `}` (valid JSON objects)
2. Common JSON log fields like `timestamp`, `level`, `message`
3. Valid JSON syntax

You can also explicitly specify JSON format using the `--format json` flag:

```bash
timber --format json app.log
```

## Basic Usage

### View All JSON Logs

```bash
timber app.log
```

If logs are detected as JSON, they will be processed accordingly.

### Filter by Log Level

```bash
timber --format json --level ERROR app.log
```

This works with common level fields like `level`, `severity`, `loglevel`, etc.

### Search for Patterns

```bash
timber --format json --chop "database error" app.log
```

Searches for the pattern in the entire JSON string.

### Show Statistics

```bash
timber --format json --stats app.log
```

Provides statistics on log levels, error types, and message uniqueness.

## Field-Based Filtering

One of Timberjack's most powerful features for JSON logs is the ability to filter by specific field values.

### Basic Field Filtering

```bash
timber --format json -f service=api app.log
```

Shows only logs where the `service` field equals `api`.

### Multiple Field Filters

```bash
timber --format json -f service=api -f status=500 app.log
```

Shows logs where `service` equals `api` AND `status` equals `500`.

### Numeric Comparisons

```bash
timber --format json -f "response_time>1000" app.log
```

Shows logs where the `response_time` field is greater than 1000.

### Combining Field Filters with Other Options

```bash
timber --format json -f service=api --level ERROR --stats app.log
```

Shows statistics for ERROR logs from the api service.

## Working with Nested JSON

Many JSON logs contain nested objects and arrays. Timberjack can extract and filter on nested fields.

### Accessing Nested Fields

```bash
timber --format json -f "user.id=12345" app.log
```

Filters logs where the `id` field within the `user` object equals `12345`.

### Deeply Nested Fields

```bash
timber --format json -f "request.headers.content-type=application/json" app.log
```

Filters logs with the specified content-type header.

### Array Elements

```bash
timber --format json -f "errors[0].code=500" app.log
```

Filters logs where the first error in the errors array has code 500.

## JSON Output

Timberjack can output results in JSON format for further processing by other tools.

### Basic JSON Output

```bash
timber --format json --json app.log
```

### JSON Output with Statistics

```bash
timber --format json --stats --json app.log
```

### Filtering and JSON Output

```bash
timber --format json -f service=api --level ERROR --json app.log > api_errors.json
```

### Structure of JSON Output

```json
{
  "matched_lines": [
    {},
    {}
  ],
  "total_count": 2,
  "time_trends": [
    { "timestamp": "2025-03-21 14", "count": 1 },
    { "timestamp": "2025-03-21 15", "count": 1 }
  ],
  "stats": {
    "log_levels": [
      { "level": "ERROR", "count": 2 }
    ],
    "error_types": [
      { "error_type": "NullPointerException", "count": 1, "rank": 1 }
    ],
    "unique_messages_count": 2,
    "repetition_ratio": 0.0,
    "unique_messages": null
  },
  "deduplicated": false
}
```

## Performance Considerations

JSON log processing can be more resource-intensive than plaintext logs. Here are some tips for optimal performance:

- Use field filtering (`-f`) early to reduce the dataset before applying pattern search
- For counting operations, use `--count` flag which is much faster
- For very large JSON log files, use `--parallel` to enable parallel processing
- Avoid using `--stats` and `--show-unique` for initial exploration of very large files

## Common Use Cases

### Microservice Log Analysis

```bash
# Find errors across all services
timber --format json --level ERROR app.log

# Trace a request through multiple services
timber --format json -f request_id=abc-123 app.log

# Find slow responses
timber --format json -f "response_time>1000" app.log

# Compare error rates across services
timber --format json --level ERROR --stats app.log
```

### API Error Analysis

```bash
# Find all 5xx errors
timber --format json -f "status>=500" app.log

# Identify authentication failures
timber --format json -f status=401 app.log

# Find errors for a specific user
timber --format json -f user_id=12345 --level ERROR app.log
```

### Performance Monitoring

```bash
# Find slow database queries
timber --format json -f "query_time>100" app.log

# View memory usage patterns
timber --format json -f "memory_usage>90" --trend app.log

# Analyze high CPU events
timber --format json -f "cpu_usage>80" --stats app.log
```

## Example: AWS CloudWatch Logs

CloudWatch often exports logs in JSON format. Here's how to analyze them:

```bash
# Extract all Lambda timeouts
timber --format json -f "errorType=TimeoutError" lambda_logs.json

# Find cold starts
timber --format json -f "message=REPORT" -f "initDuration>0" lambda_logs.json
```

## Example: Kubernetes Pod Logs

```bash
# Find errors in a specific namespace
timber --format json -f "kubernetes.namespace=production" -f level=ERROR k8s_logs.json

# Analyze container restarts
timber --format json -f "reason=ContainerRestarted" --stats k8s_logs.json
```

## Troubleshooting

### Invalid JSON Format

If your logs are not in valid JSON format but still contain JSON-like structures, try:

```bash
timber --format auto --json-tolerance=high app.log
```

### Missing Fields

If field filtering isn't working as expected, verify the exact field names with:

```bash
timber --format json --sample 1 app.log
```

### Performance Issues with Large Files

For very large JSON log files:

```bash
timber --format json --parallel --buffer-size=8M app.log
```

### Case Sensitivity

Field filtering is case-sensitive by default. For case-insensitive matching:

```bash
timber --format json -f "level=error" --case-insensitive app.log
```

## Conclusion

Timberjack's JSON log processing capabilities make it an ideal tool for analyzing structured logs from modern applications. By combining automatic detection with powerful field filtering and analysis, you can quickly extract valuable insights from your JSON logs.

For more information on other Timberjack features, refer to the main [Timberjack documentation](README.md).