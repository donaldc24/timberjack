# Changelog

All notable changes to Timber will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.3] - 2025-03-23

### Added
- SIMD acceleration for pattern matching
- Fast `--count` flag for efficient log counting
- JSON output support via `--json` flag
- Show unique messages with `--show-unique` flag
- Custom top error limit with `--top-errors <N>` option
- Explicit parallel/sequential processing controls

### Changed
- Switched to memory-mapped file processing for improved performance
- Optimized memory usage with smarter string allocation
- Enhanced deduplication of repeated log lines
- Improved regex pattern matching performance
- More efficient timestamp extraction
- Better handling of very large files
- Updated benchmarking suite with comprehensive comparisons

### Fixed
- Reduced memory usage for large log files
- Improved handling of malformed log lines
- Better error reporting for invalid regex patterns
- Fixed potential issues with concurrent access in parallel mode

### Performance
- 2-3x overall performance improvement
- Competitive with grep/ripgrep for counting operations
- Memory usage reduced by approximately 40%
- Significantly faster pattern matching with SIMD
- Automatic parallelization for large files

## [0.1.0-alpha.2] - 2025-03-22

### Changed
- Renamed crate to `timber-rs` for crates.io publication
- Updated documentation to reflect new crate name
- Configured binary to maintain `timber` command name

### Fixed
- Resolved crate publication issues
- Updated test suites to work with new crate name

## [0.1.0-alpha.1] - 2025-03-22

### Added
- Initial project structure
- Basic log file analysis functionality
- Command-line interface with clap
- Core log parsing capabilities
- Pattern searching
- Log level filtering
- Time-based trend analysis
- Statistical summaries
- GitHub repository setup
- Continuous Integration with GitHub Actions

### Features
- Regex-based pattern matching
- Log level filtering (ERROR, WARN, INFO)
- Time trend detection
- Basic statistical reporting
- Flexible log file processing

## Planned Features

### Short-term Roadmap
- [ ] Format-specific parsers for common log formats (JSON, Apache, etc.)
- [ ] Package manager distributions (Homebrew, apt, etc.)
- [ ] VS Code extension
- [ ] Multi-file analysis
- [ ] Enhanced error handling

### Long-term Vision
- Pattern identification and correlation
- Root cause suggestion
- Advanced visualization
- Distributed log analysis
- IDE integrations

## Contributing

We welcome contributions! Please see our [CONTRIBUTING.md](CONTRIBUTING.md) for details on how to get started.