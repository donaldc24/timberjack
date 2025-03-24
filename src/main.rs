use clap::Parser;
use regex::Regex;
use memmap2::MmapOptions;
use std::fs::File;
use std::time::Instant;

use timber_rs::analyzer::LogAnalyzer;
use timber_rs::cli::Args;
use timber_rs::formatter::print_results;
use timber_rs::parser::{ParserRegistry, LogFormat};

// Threshold for using parallel processing (in bytes)
const PARALLEL_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024; // 10MB

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Skip the banner when using JSON output or count for cleaner output
    if !args.json && !args.count {
        println!("\nWaking LumberJacks...Timber is chopping: {}\n", args.file);
    }

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

    // Create parser registry
    let parser_registry = ParserRegistry::new();

    // Determine format to use
    let format = match args.format.to_lowercase().as_str() {
        "auto" => {
            // Open the file for sampling
            let file = File::open(&args.file)?;
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
                                sample_lines.push(line);
                                line_count += 1;
                                if line_count >= 10 {
                                    break;
                                }
                            }
                        }
                        start = i + 1;
                    }
                }

                // Detect format using the sample lines
                let (detected_format, _) = parser_registry.detect_format(&sample_lines);

                if !args.json && !args.count {
                    println!("Detected format: {:?}", detected_format);
                }

                detected_format
            }
        },
        "json" => LogFormat::Json,
        "apache" => LogFormat::Apache,
        "syslog" => LogFormat::Syslog,
        _ => LogFormat::Generic,
    };

    // Get the appropriate parser
    let parser = parser_registry.get_parser(format)
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
        should_use_parallel(&args.file)
    };

    // If only count is needed, use a fast counting method
    if args.count {
        let count = count_total_logs(&args.file, pattern.as_ref(), level)?;
        println!("{}", count);
        return Ok(());
    }

    // Process the file using memory mapping
    let result = process_with_mmap(
        &args.file,
        &mut analyzer,
        pattern.as_ref(),
        level,
        args.trend,
        args.stats,
        use_parallel,
    )?;

    // Print processing time if not in JSON mode
    let elapsed = start_time.elapsed();
    if !args.json {
        println!("Analysis completed in {:.2}s", elapsed.as_secs_f32());
    }

    // Print the results
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

// Fast method to count total logs with optional filtering
fn count_total_logs(
    file_path: &str,
    pattern: Option<&Regex>,
    level_filter: Option<&str>,
) -> std::io::Result<usize> {
    let file = File::open(file_path)?;
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
        let mut result = timber_rs::analyzer::AnalysisResult::default();

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
    file_path: &str,
    analyzer: &mut LogAnalyzer,
    pattern: Option<&Regex>,
    level_filter: Option<&str>,
    collect_trends: bool,
    collect_stats: bool,
    use_parallel: bool,
) -> std::io::Result<timber_rs::analyzer::AnalysisResult> {
    let path = std::path::Path::new(file_path);
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("File not found: {}", file_path)
        ));
    }

    let file = File::open(path)?;

    // Skip processing for empty files
    let file_size = file.metadata()?.len();
    if file_size == 0 {
        return Ok(timber_rs::analyzer::AnalysisResult::default());
    }

    // Memory map the file
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    // Process the mapped memory
    if use_parallel && file_size > PARALLEL_THRESHOLD_BYTES {
        // Use analyzer's parallel mmap processing method
        Ok(analyzer.analyze_mmap_parallel(&mmap, pattern, level_filter, collect_trends, collect_stats))
    } else {
        // Use analyzer's sequential mmap processing method
        Ok(analyzer.analyze_mmap(&mmap, pattern, level_filter, collect_trends, collect_stats))
    }
}

// Determine if parallel processing should be used based on file size
fn should_use_parallel(file_path: &str) -> bool {
    match std::fs::metadata(file_path) {
        Ok(metadata) => {
            let size = metadata.len();
            size > PARALLEL_THRESHOLD_BYTES
        },
        Err(_) => false, // If we can't determine file size, assume small
    }
}