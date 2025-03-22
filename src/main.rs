use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

use timber_rs::analyzer::LogAnalyzer;
use timber_rs::cli::Args;
use timber_rs::formatter::print_results;

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

    // Open and read the file
    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.expect("Could not read line"));

    // Create analyzer and analyze the file
    let analyzer = LogAnalyzer::new();
    let result = analyzer.analyze_lines(lines, pattern.as_ref(), level, args.trend, args.stats);

    // Print the results
    if !args.json {
        println!();
    }

    print_results(
        &result,
        args.trend,
        args.stats,
        args.json
    );

    Ok(())
}