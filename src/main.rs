use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use clap::Parser;
use regex::Regex;

#[derive(Parser, Debug)]
#[clap(name = "timber")]
#[clap(about = "Timber: Fell Your Logs Fast", long_about = None)]
struct Args {
    /// Log file to analyze
    file: String,

    /// Pattern to search for
    #[clap(short, long)]
    chop: Option<String>,

    //// Filter by log level (ERROR, WARN, INFO, etc...)
    #[clap(short, long)]
    level: Option<String>,

    /// Show time-based trends
    #[clap(long)]
    trend: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    println!("Waking Lumberjacks! Chopping: {}", args.file);

    let pattern = match &args.chop {
        Some(pattern) => {
            println!("Searching for pattern: {}", pattern);
            Some(Regex::new(pattern).expect("Invalid regex pattern"))
        },
        None => None,
    };

    let level = args.level.as_deref();
    if let Some(level_str) = level {
        println!("Filtering by level: {}", level_str);
    }

    let level_regex = Regex::new(r"\[((?i)ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE)\]|(?i:ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE):").expect("Failed to create level regex");

    let timestamp_regex = Regex::new(r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})").expect("Failed to create timestamp regex");

    let file = File::open(&args.file)?;
    let reader = BufReader::new(file);

    let mut count = 0;
    let mut time_trends= HashMap::new();

    for line in reader.lines() {
        let line = line?;

        let level_matches = if let Some(filter_level) = level {
            if let Some(caps) = level_regex.captures(&line) {
                let found_level = caps.get(1)
                    .map_or_else(
                        || caps.get(0).map_or("", |m| m.as_str()),
                        |m| m.as_str()
                    );

                found_level.to_uppercase() == filter_level.to_uppercase()
            } else {
                false
            }
        } else {
            true
        };

        let pattern_matches = pattern.as_ref()
            .map_or(true, |re| re.is_match(&line));

        if level_matches && pattern_matches {
            println!("{}", line);
            count += 1;

            if args.trend {
                if let Some(caps) = timestamp_regex.captures(&line) {
                    if let Some(timestamp) = caps.get(1) {
                        let timestamp_str = timestamp.as_str();
                        let hour = if timestamp_str.len() >= 13 {
                            &timestamp_str[0..13]
                        } else {
                            timestamp_str
                        };

                        *time_trends.entry(hour.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    println!("\nFelled: {} logs", count);

    if args.trend && !time_trends.is_empty() {
        println!("\nTime trends:");

        let mut trend_entries: Vec<_> = time_trends.iter().collect();
        trend_entries.sort_by(|a, b| a.0.cmp(b.0));

        for (hour, count) in trend_entries {
            println!("  {}:00 - {} log{} occurred during this hour",
                     hour,
                     count,
                     if *count == 1 { "" } else { "s" }  // Proper pluralization
            );
        }
    }

    println!("Timber finished chopping the log!");
    Ok(())
}
