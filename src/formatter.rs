use crate::analyzer::AnalysisResult;
use serde::Serialize;
use std::cmp::Reverse;
use std::io::{self, Write};

// New struct specifically for JSON output
#[derive(Serialize)]
struct JsonOutput {
    matched_lines: Vec<LineWithCount>,
    total_count: usize,
    time_trends: Option<Vec<TimeTrend>>,
    stats: Option<Stats>,
    deduplicated: bool,
}

#[derive(Serialize)]
struct LineWithCount {
    line: String,
    count: usize,
}

#[derive(Serialize)]
struct TimeTrend {
    timestamp: String,
    count: usize,
}

#[derive(Serialize)]
struct Stats {
    log_levels: Vec<LevelCount>,
    error_types: Vec<ErrorTypeCount>,
    unique_messages_count: usize,
    repetition_ratio: f64,
    unique_messages: Option<Vec<String>>,
}

#[derive(Serialize)]
struct LevelCount {
    level: String,
    count: usize,
}

#[derive(Serialize)]
struct ErrorTypeCount {
    error_type: String,
    count: usize,
    rank: usize,
}

// Main production function - prints directly to stdout for maximum performance
pub fn print_results(
    result: &AnalysisResult,
    show_trends: bool,
    show_stats: bool,
    json_output: bool,
    top_errors: usize,
    show_unique: bool,
) {
    if json_output {
        // Create a more structured JSON output
        let mut json_output_data = JsonOutput {
            matched_lines: Vec::new(),
            total_count: result.count,
            time_trends: None,
            stats: None,
            deduplicated: result.deduplicated,
        };

        // Convert matched lines with counts
        if result.deduplicated && !result.line_counts.is_empty() {
            for line in &result.matched_lines {
                let count = result.line_counts.get(line).unwrap_or(&1);
                json_output_data.matched_lines.push(LineWithCount {
                    line: line.clone(),
                    count: *count,
                });
            }
        } else {
            // Original behavior
            for line in &result.matched_lines {
                json_output_data.matched_lines.push(LineWithCount {
                    line: line.clone(),
                    count: 1,
                });
            }
        }

        // Add time trends if enabled
        if show_trends && !result.time_trends.is_empty() {
            let mut trend_entries: Vec<_> = result.time_trends.iter().collect();
            trend_entries.sort_by(|a, b| a.0.cmp(b.0));

            let trends = trend_entries
                .into_iter()
                .map(|(timestamp, count)| TimeTrend {
                    timestamp: timestamp.clone(),
                    count: *count,
                })
                .collect();

            json_output_data.time_trends = Some(trends);
        }

        // Add stats if enabled
        if show_stats {
            let repetition_ratio = if result.count > 0 {
                (1.0 - (result.unique_messages.len() as f64 / result.count as f64)) * 100.0
            } else {
                0.0
            };

            // Process log levels
            let mut level_entries: Vec<_> = result.levels_count.iter().collect();
            level_entries.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

            let log_levels = level_entries
                .into_iter()
                .map(|(level, count)| LevelCount {
                    level: level.clone(),
                    count: *count,
                })
                .collect();

            // Process error types
            let mut error_entries: Vec<_> = result.error_types.iter().collect();
            error_entries.sort_by_key(|&(_, count)| std::cmp::Reverse(*count));

            let error_types = error_entries
                .into_iter()
                .enumerate()
                .take(top_errors) // Use the configurable limit
                .map(|(idx, (error_type, count))| ErrorTypeCount {
                    error_type: error_type.clone(),
                    count: *count,
                    rank: idx + 1,
                })
                .collect();

            // Create stats object
            let unique_messages = if show_unique {
                let mut messages: Vec<String> = result.unique_messages.iter().cloned().collect();
                messages.sort(); // Sort alphabetically for consistent output
                Some(messages)
            } else {
                None
            };

            let stats = Stats {
                log_levels,
                error_types,
                unique_messages_count: result.unique_messages.len(),
                repetition_ratio,
                unique_messages,
            };

            json_output_data.stats = Some(stats);
        }

        // Serialize to JSON
        let json_str = serde_json::to_string_pretty(&json_output_data)
            .unwrap_or_else(|_| "Failed to serialize results".to_string());

        println!("{}", json_str);
        return;
    }

    // Print matching lines with counts if deduplicated
    if result.deduplicated && !result.line_counts.is_empty() {
        for line in &result.matched_lines {
            let count = result.line_counts.get(line).unwrap_or(&1);
            if *count > 1 {
                println!("{} [x{}]", line, count);
            } else {
                println!("{}", line);
            }
        }

        // If there are more lines than we stored, show a message
        if result.count > result.matched_lines.len() {
            println!(
                "... and {} more lines (total: {})",
                result.count - result.matched_lines.len(),
                result.count
            );
        }
    } else {
        // Original behavior - print individual lines
        for line in &result.matched_lines {
            println!("{}", line);
        }
    }

    // Print summary
    println!("\nFelled: {} logs", result.count);

    // Print time trends if enabled
    if show_trends && !result.time_trends.is_empty() {
        println!("\nTime trends:");

        // Sort by timestamp
        let mut trend_entries: Vec<_> = result.time_trends.iter().collect();
        trend_entries.sort_by(|a, b| a.0.cmp(b.0));

        for (hour, count) in trend_entries {
            // Format: "2025-03-21 14:00 - 3 logs occurred during this hour"
            println!(
                "  {} - {} log{} occurred during this hour",
                hour,
                count,
                if *count == 1 { "" } else { "s" } // Proper pluralization
            );
        }
    }

    // Print stats if enabled
    if show_stats {
        println!("\nStats summary:");

        // Print level distribution
        if !result.levels_count.is_empty() {
            println!("\n  Log levels:");
            let mut level_entries: Vec<_> = result.levels_count.iter().collect();
            level_entries.sort_by_key(|&(_, count)| Reverse(*count)); // Sort by count (descending)

            for (level, level_count) in level_entries {
                println!(
                    "    {}: {} log{}",
                    level,
                    level_count,
                    if *level_count == 1 { "" } else { "s" }
                );
            }
        }

        // Print top error types
        if !result.error_types.is_empty() {
            println!("\n  Top error types:");
            let mut error_entries: Vec<_> = result.error_types.iter().collect();
            error_entries.sort_by_key(|&(_, count)| Reverse(*count)); // Sort by count (descending)

            // Show top N or fewer based on user configuration
            for (idx, (error, error_count)) in error_entries.iter().take(top_errors).enumerate() {
                println!(
                    "    {}. {}: {} occurrence{}",
                    idx + 1,
                    error,
                    error_count,
                    if **error_count == 1 { "" } else { "s" }
                );
            }
        }

        // Print uniqueness stats
        println!("\n  Unique messages: {}", result.unique_messages.len());
        println!(
            "  Repetition ratio: {:.1}%",
            if result.count > 0 {
                (1.0 - (result.unique_messages.len() as f64 / result.count as f64)) * 100.0
            } else {
                0.0
            }
        );

        // Show unique messages if requested
        if show_unique && !result.unique_messages.is_empty() {
            println!("\n  Unique messages:");
            let mut unique_messages: Vec<_> = result.unique_messages.iter().collect();
            unique_messages.sort(); // Sort alphabetically

            for message in unique_messages {
                println!("    - {}", message);
            }
        }
    }

    println!("\nTimber finished chopping the log! 🪵");
}

// Test-friendly version that can write to any writer
// Make this public for integration tests
pub fn print_results_to_writer<W: Write>(
    result: &AnalysisResult,
    show_trends: bool,
    show_stats: bool,
    writer: &mut W,
    top_errors: usize,
    show_unique: bool,
) -> io::Result<()> {
    // Print matching lines with counts if deduplicated
    if result.deduplicated && !result.line_counts.is_empty() {
        for line in &result.matched_lines {
            let count = result.line_counts.get(line).unwrap_or(&1);
            if *count > 1 {
                writeln!(writer, "{} [x{}]", line, count)?;
            } else {
                writeln!(writer, "{}", line)?;
            }
        }

        // If there are more lines than we stored, show a message
        if result.count > result.matched_lines.len() {
            writeln!(
                writer,
                "... and {} more lines (total: {})",
                result.count - result.matched_lines.len(),
                result.count
            )?;
        }
    } else {
        // Original behavior - print individual lines
        for line in &result.matched_lines {
            writeln!(writer, "{}", line)?;
        }
    }

    // Print summary
    writeln!(writer, "\nFelled: {} logs", result.count)?;

    // Print time trends if enabled
    if show_trends && !result.time_trends.is_empty() {
        writeln!(writer, "\nTime trends:")?;

        // Sort by timestamp
        let mut trend_entries: Vec<_> = result.time_trends.iter().collect();
        trend_entries.sort_by(|a, b| a.0.cmp(b.0));

        for (hour, count) in trend_entries {
            // Format: "2025-03-21 14:00 - 3 logs occurred during this hour"
            writeln!(
                writer,
                "  {} - {} log{} occurred during this hour",
                hour,
                count,
                if *count == 1 { "" } else { "s" } // Proper pluralization
            )?;
        }
    }

    // Print stats if enabled
    if show_stats {
        writeln!(writer, "\nStats summary:")?;

        // Print level distribution
        if !result.levels_count.is_empty() {
            writeln!(writer, "\n  Log levels:")?;
            let mut level_entries: Vec<_> = result.levels_count.iter().collect();
            level_entries.sort_by_key(|&(_, count)| Reverse(*count)); // Sort by count (descending)

            for (level, level_count) in level_entries {
                writeln!(
                    writer,
                    "    {}: {} log{}",
                    level,
                    level_count,
                    if *level_count == 1 { "" } else { "s" }
                )?;
            }
        }

        // Print top error types
        if !result.error_types.is_empty() {
            writeln!(writer, "\n  Top error types:")?;
            let mut error_entries: Vec<_> = result.error_types.iter().collect();
            error_entries.sort_by_key(|&(_, count)| Reverse(*count)); // Sort by count (descending)

            // Show top N or fewer based on user configuration
            for (idx, (error, error_count)) in error_entries.iter().take(top_errors).enumerate() {
                writeln!(
                    writer,
                    "    {}. {}: {} occurrence{}",
                    idx + 1,
                    error,
                    error_count,
                    if **error_count == 1 { "" } else { "s" }
                )?;
            }
        }

        // Print uniqueness stats
        writeln!(
            writer,
            "\n  Unique messages: {}",
            result.unique_messages.len()
        )?;
        writeln!(
            writer,
            "  Repetition ratio: {:.1}%",
            if result.count > 0 {
                (1.0 - (result.unique_messages.len() as f64 / result.count as f64)) * 100.0
            } else {
                0.0
            }
        )?;

        // Show unique messages if requested
        if show_unique && !result.unique_messages.is_empty() {
            writeln!(writer, "\n  Unique messages:")?;
            let mut unique_messages: Vec<_> = result.unique_messages.iter().collect();
            unique_messages.sort(); // Sort alphabetically

            for message in unique_messages {
                writeln!(writer, "    - {}", message)?;
            }
        }
    }

    writeln!(writer, "\nTimber finished chopping the log! 🪵")?;
    Ok(())
}
