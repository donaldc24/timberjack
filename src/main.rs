use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};

use timber::analyzer::LogAnalyzer;
use timber::cli::Args;
use timber::formatter::print_results;

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    println!("\nWaking LumberJacks...Timber is chopping: {}\n", args.file);

    // Set up pattern matching
    let pattern = match &args.chop {
        Some(pattern) => {
            println!("Searching for pattern: {}", pattern);
            Some(Regex::new(pattern).expect("Invalid regex pattern"))
        }
        None => None,
    };

    // Set up level filtering
    let level = args.level.as_deref();
    if let Some(level_str) = level {
        println!("Filtering by level: {}", level_str);
    }

    // Open and read the file
    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);
    let lines = reader.lines().map(|l| l.expect("Could not read line"));

    // Create analyzer and analyze the file
    let analyzer = LogAnalyzer::new();
    let result = analyzer.analyze_lines(lines, pattern.as_ref(), level, args.trend, args.stats);

    // Print the results
    println!();
    print_results(&result, args.trend, args.stats);

    Ok(())
}
