use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use timber_rs::analyzer::LogAnalyzer;
use timber_rs::cli::Args;
use timber_rs::formatter::print_results;

// Threshold for using parallel processing (in bytes)
const PARALLEL_THRESHOLD_BYTES: u64 = 10 * 1024 * 1024; // 10MB

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Skip the banner when using JSON output for cleaner piping
    if !args.json {
        println!("\nWaking LumberJacks...Timber is chopping: {}\n", args.file);
    }

    // Set up pattern matching
    let pattern = match &args.chop {
        Some(pattern) => {
            if !args.json {
                println!("Searching for pattern: {}", pattern);
            }
            Some(Regex::new(pattern).expect("Invalid regex pattern"))
        }
        None => None,
    };

    // Set up level filtering
    let level = args.level.as_deref();
    if let Some(level_str) = level {
        if !args.json {
            println!("Filtering by level: {}", level_str);
        }
    }

    // Start timing analysis
    let start_time = Instant::now();

    // Create analyzer
    let analyzer = LogAnalyzer::new();

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

    // Process the file according to chosen mode
    let result = if use_parallel {
        // For large files, use parallel processing
        if !args.json {
            println!("Using parallel processing");
        }

        // Read all lines into memory for parallel processing
        let file = File::open(&args.file)?;
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .filter_map(Result::ok)
            .collect();

        analyzer.analyze_lines_parallel(lines, pattern.as_ref(), level, args.trend, args.stats)
    } else {
        // For smaller files, use sequential processing
        if !args.json && should_use_parallel(&args.file) {
            println!("Using sequential processing (override)");
        }

        let file = File::open(&args.file)?;
        let reader = BufReader::new(file);
        let lines = reader.lines().map(|l| l.expect("Could not read line"));

        analyzer.analyze_lines(lines, pattern.as_ref(), level, args.trend, args.stats)
    };

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