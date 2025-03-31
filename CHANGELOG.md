# Changelog

All notable changes to Timberjack will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-beta.1] - 2025-04-01
### Added

- Full stdin support for log processing
- Ability to pipe logs from other commands
- Automatic log format detection for stdin input
- Stdin-specific performance optimizations
- Support for compressed log streams via zcat/gunzip
- Memory-mapped processing for stdin streams larger than 10MB

### Enhanced

- Improved memory efficiency for streaming log inputs
- Optimized parallel processing for stdin-based logs
- Better error handling for different input stream scenarios

### Changed

- Updated CLI to seamlessly handle file and stdin inputs
- Expanded command-line examples to showcase stdin capabilities

### Performance

- Minimal overhead when processing streamed logs
- Automatic thread scaling for large stdin inputs
- Efficient memory management for log streams

## [0.1.0-alpha.4] - 2025-03-24
### 🔄 Rebranding Release

- Name Change

    - Project Renamed: From "Timber" to "Timberjack" to avoid naming conflicts with existing tools
Package Renamed: Now published as timberjack on crates.io (previously timber-rs)
Repository Renamed: GitHub repository now at donaldc24/timberjack

- Command Line Experience

    - Preserved CLI Command: The binary name remains timber for backward compatibility
Updated CLI Help Text: Help documentation now references the new name
Updated Banner: Startup messages now reference Timberjack
Added Note: Informative note about the rebranding in help text

- Documentation & Branding

    - Updated README: All documentation updated to reflect new name
Added Migration Guide: New MIGRATING.md file explains the transition
New Visual Identity: Refreshed logo and color scheme
New Domains: Secured timberjack.dev and timberjack.rs

- Internal Changes

    - Updated Package Metadata: All package identifiers updated to reflect Timberjack
Code References: Internal code references to project name updated
Build System: Updated CI/CD pipelines to reflect new name

- Deprecated

    - The timber-rs crate is now deprecated in favor of timberjack

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