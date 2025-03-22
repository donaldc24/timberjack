use crate::analyzer::AnalysisResult;
use std::cmp::Reverse;
use std::io::{self, Write};

// Main production function - prints directly to stdout for maximum performance
pub fn print_results(result: &AnalysisResult, show_trends: bool, show_stats: bool) {
    // Print matching lines
    for line in &result.matched_lines {
        println!("{}", line);
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

            // Show top 5 or fewer
            for (idx, (error, error_count)) in error_entries.iter().take(5).enumerate() {
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
) -> io::Result<()> {
    // Print matching lines
    for line in &result.matched_lines {
        writeln!(writer, "{}", line)?;
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

            // Show top 5 or fewer
            for (idx, (error, error_count)) in error_entries.iter().take(5).enumerate() {
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
    }

    writeln!(writer, "\nTimber finished chopping the log! 🪵")?;
    Ok(())
}
