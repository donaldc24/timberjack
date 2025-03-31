use atty::Stream;
use clap::Parser;
use memmap2::MmapOptions;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::time::Instant;

use timberjack::analyzer::LogAnalyzer;
use timberjack::cli::Args;
use timberjack::formatter::print_results;
use timberjack::parser::{LogFormat, ParserRegistry};

// Threshold for using parallel processing (in bytes)
const PARALLEL_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024; // 10MB
const MAX_UNIQUE_LINES: usize = 10000;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let using_stdin = atty::isnt(Stream::Stdin);

    if !using_stdin && args.file.is_none() {
        eprintln!("Error: No input source. Provide a file or pipe data to stdin.");
        std::process::exit(1);
    }

    // Skip the banner when using JSON output, or count
    if !args.json && !args.count {
        if using_stdin {
            println!("\nWaking LumberJacks...Timberjack is chopping from stdin\n");
        } else if let Some(file) = &args.file {
            println!("\nWaking LumberJacks...Timberjack is chopping: {}\n", file);
        }
    }

    // Create parser registry
    let parser_registry = ParserRegistry::new();

    // Determine format to use
    let format = match args.format.to_lowercase().as_str() {
        "auto" => {
            if let Some(file_path) = &args.file {
                // Open the file for sampling
                let file = File::open(file_path)?;
                if file.metadata()?.len() == 0 {
                    LogFormat::Generic
                } else {
                    // Memory map just the beginning of the file for sampling
                    let mmap = unsafe { MmapOptions::new().map(&file)? };

                    // Extract sample lines for format detection (first ~10 lines)
                    let mut sample_lines = Vec::with_capacity(10);
                    let mut start = 0;
                    let mut line_count = 0;

                    // Get up to 10 lines or 4KB, whichever comes first
                    let max_sample = std::cmp::min(4096, mmap.len());

                    for i in 0..max_sample {
                        if i == mmap.len() - 1 || mmap[i] == b'\n' {
                            if i > start {
                                // Extract a line - handle UTF-8 encoding properly
                                if let Ok(line) = std::str::from_utf8(&mmap[start..i]) {
                                    let trimmed_line = line.trim();
                                    if !trimmed_line.is_empty() {
                                        sample_lines.push(trimmed_line);
                                        line_count += 1;
                                        if line_count >= 10 {
                                            break;
                                        }
                                    }
                                }
                            }
                            start = i + 1;
                        }
                    }

                    // Detect format using the sample lines
                    let (detected_format, _) =
                        parser_registry.detect_format(&sample_lines.to_vec());

                    if !args.json && !args.count {
                        println!("Detected format: {:?}", detected_format);
                    }

                    detected_format
                }
            } else {
                LogFormat::Generic
            }
        }
        "json" => LogFormat::Json,
        "apache" => LogFormat::Apache,
        "syslog" => LogFormat::Syslog,
        _ => LogFormat::Generic,
    };

    // Set up pattern matching
    let pattern = match &args.chop {
        Some(pattern) => {
            if !args.json && !args.count {
                println!("Searching for pattern: {}", pattern);
            }
            Some(Regex::new(pattern).expect("Invalid regex pattern"))
        }
        None => None,
    };

    // Set up level filtering
    let level = args.level.as_deref();
    if let Some(level_str) = level {
        if !args.json && !args.count {
            println!("Filtering by level: {}", level_str);
        }
    }

    // Start timing analysis
    let start_time = Instant::now();

    // Create analyzer
    let mut analyzer = LogAnalyzer::new();

    // Set field filters if specified
    if !args.field.is_empty() {
        if !args.json && !args.count {
            println!("Filtering by fields: {:?}", args.field);
        }
        analyzer.set_field_filters(args.field);
    }

    // Get the appropriate parser
    let parser = parser_registry
        .get_parser(format)
        .expect("Failed to get parser for format");

    // Set the parser in the analyzer
    analyzer.set_parser(parser);

    // Determine processing mode (parallel or sequential)
    let use_parallel = if args.sequential {
        // Sequential flag overrides parallel
        false
    } else if args.parallel {
        // User explicitly requested parallel
        true
    } else {
        // Auto-detect based on file size
        should_use_parallel(args.file.as_deref())
    };

    // If only count is needed, use a fast counting method
    if args.count {
        let count = count_total_logs(args.file.as_deref(), pattern.as_ref(), level)?;
        println!("{}", count);
        return Ok(());
    }

    let result = if using_stdin {
        // Process from stdin
        process_from_stdin(
            &mut analyzer,
            pattern.as_ref(),
            level,
            args.trend,
            args.stats,
        )?
    } else if args.file.is_some() {
        // Process from file (your existing code)
        process_with_mmap(
            args.file.as_deref(),
            &mut analyzer,
            pattern.as_ref(),
            level,
            args.trend,
            args.stats,
            use_parallel,
        )?
    } else {
        // This shouldn't happen due to the earlier check
        unreachable!()
    };

    // Print processing time if not in JSON mode
    let elapsed = start_time.elapsed();
    if !args.json {
        if using_stdin {
            println!(
                "Analysis completed in {:.2}s (source: stdin)",
                elapsed.as_secs_f32()
            );
        } else if let Some(file) = &args.file {
            println!(
                "Analysis completed in {:.2}s (source: {})",
                elapsed.as_secs_f32(),
                file
            );
        }
    }

    if !args.json {
        println!();
    }

    print_results(
        &result,
        args.trend,
        args.stats,
        args.json,
        args.top_errors,
        args.show_unique,
    );

    Ok(())
}

fn process_from_stdin(
    analyzer: &mut LogAnalyzer,
    pattern: Option<&Regex>,
    level_filter: Option<&str>,
    collect_trends: bool,
    collect_stats: bool,
) -> std::io::Result<timberjack::analyzer::AnalysisResult> {
    // Configure analyzer with pattern if included
    if let Some(pat) = pattern {
        analyzer.configure(Some(&pat.to_string()), level_filter);
    } else if level_filter.is_some() {
        analyzer.configure(None, level_filter);
    }

    // Create result object to collect analysis
    let mut result = timberjack::analyzer::AnalysisResult {
        deduplicated: true,
        ..Default::default()
    };

    // Process Line by Line from stdin
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);

    // Process Lines Directly
    for line_result in reader.lines() {
        let line = line_result?;
        if let Some((matched_line, level, timestamp)) = analyzer.analyze_line(
            &line,
            None,
            analyzer.get_level_filter(),
            collect_trends,
            collect_stats,
        ) {
            result.count += 1;

            if result.matched_lines.len() < MAX_UNIQUE_LINES {
                let line_count_entry = result.line_counts.entry(matched_line.clone()).or_insert(0);
                *line_count_entry += 1;

                if !result.matched_lines.contains(&matched_line) {
                    result.matched_lines.push(matched_line.clone());
                }
            }

            // Update trends if requested
            if collect_trends {
                if let Some(ts) = timestamp {
                    let hour = if ts.len() >= 13 {
                        ts[0..13].to_string()
                    } else {
                        ts
                    };
                    *result.time_trends.entry(hour).or_insert(0) += 1;
                }
            }

            // Update stats if requested
            if collect_stats {
                // Update level counts
                *result.levels_count.entry(level.clone()).or_insert(0) += 1;

                // Update error types
                if let Some(error_type) = analyzer.extract_error_type(&matched_line) {
                    *result.error_types.entry(error_type).or_insert(0) += 1;
                }

                // Update unique messages
                if let Some(message) = matched_line.split(']').nth(1).map(|s| s.trim().to_string())
                {
                    result.unique_messages.insert(message);
                } else {
                    result.unique_messages.insert(matched_line);
                }
            }
        }
    }

    Ok(result)
}

// Fast method to count total logs with optional filtering
fn count_total_logs(
    file_path: Option<&str>,
    pattern: Option<&Regex>,
    level_filter: Option<&str>,
) -> std::io::Result<usize> {
    let path = file_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No file path provided".to_string(),
        )
    })?;

    let file = File::open(path)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let mut total_count = 0;
    let mut analyzer = LogAnalyzer::new();

    // Configure with pattern and level if provided
    if let Some(pat) = pattern {
        analyzer.configure(Some(&pat.to_string()), level_filter);
    } else {
        analyzer.configure(None, level_filter);
    }

    // Fast counting using chunk processing
    const FAST_CHUNK_SIZE: usize = 1_048_576; // 1MB
    let mut position = 0;

    while position < mmap.len() {
        let chunk_end = std::cmp::min(position + FAST_CHUNK_SIZE, mmap.len());
        let chunk = &mmap[position..chunk_end];

        // Temporary result to just count lines
        let mut result = timberjack::analyzer::AnalysisResult::default();

        // Process chunk with minimal overhead
        analyzer.process_chunk_data(chunk, &mut result, false, false);

        total_count += result.count;

        // Move to next chunk
        position += chunk_end - position;
    }

    Ok(total_count)
}

// Process file using memory mapping
fn process_with_mmap(
    file_path: Option<&str>,
    analyzer: &mut LogAnalyzer,
    pattern: Option<&Regex>,
    level_filter: Option<&str>,
    collect_trends: bool,
    collect_stats: bool,
    use_parallel: bool,
) -> std::io::Result<timberjack::analyzer::AnalysisResult> {
    let path_str = file_path.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No file path provided".to_string(),
        )
    })?;

    let path = std::path::Path::new(path_str);
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", path_str),
        ));
    }

    let file = File::open(path)?;

    // Skip processing for empty files
    let file_size = file.metadata()?.len();
    if file_size == 0 {
        return Ok(timberjack::analyzer::AnalysisResult::default());
    }

    // Memory map the file
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    // Process the mapped memory
    Ok(analyzer.analyze_mmap(
        &mmap,
        pattern,
        level_filter,
        collect_trends,
        collect_stats,
        use_parallel,
    ))
}

// Determine if parallel processing should be used based on file size
fn should_use_parallel(file_path: Option<&str>) -> bool {
    match file_path {
        Some(path) => match std::fs::metadata(path) {
            Ok(metadata) => {
                let size = metadata.len();
                size > PARALLEL_THRESHOLD_BYTES
            }
            Err(_) => false,
        },
        None => false,
    }
}
