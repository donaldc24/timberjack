use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

/// Creates benchmark log files with the specified number of lines.
/// These logs follow a standardized format for consistent comparisons.
fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <number-of-lines> <output-file>", args[0]);
        std::process::exit(1);
    }

    let num_lines: usize = args[1].parse().expect("Invalid number of lines");
    let file_path = &args[2];

    println!("Creating log file with {} lines at {}", num_lines, file_path);
    create_benchmark_logs(num_lines, file_path)?;
    println!("Log file created successfully.");

    Ok(())
}

/// Create a standardized log file with the specified number of lines.
/// This function creates logs with ERROR, WARN, INFO, DEBUG, and TRACE levels
/// and various message types to simulate real logs.
fn create_benchmark_logs(lines: usize, file_path: &str) -> io::Result<()> {
    // Create output directory if needed
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(file_path)?;

    for i in 0..lines {
        let level = match i % 5 {
            0 => "ERROR",
            1 => "WARN",
            2 => "INFO",
            3 => "DEBUG",
            _ => "TRACE",
        };

        let message = match i % 20 {
            0 => "NullPointerException in WebController.java:42",
            1 => "Connection timeout in NetworkClient.java:86",
            2 => "Database query took 2.3s in DatabaseService.java:128",
            3 => "Application started successfully",
            4 => "Session created for user_123",
            5 => "OutOfMemoryError in SearchIndexer.java:212",
            6 => "Failed to process request: invalid parameters",
            7 => "Cache miss for key: user_profile_123",
            8 => "Authentication successful for user_123",
            9 => "Request processed in 150ms",
            10 => "500 Internal Server Error: POST /api/orders",
            11 => "403 Forbidden: Access denied for user_456",
            12 => "Slow database operation detected (query took 3.5s)",
            13 => "Memory usage at 75% of allocated heap",
            14 => "Cache hit ratio: 65.4% (last hour)",
            15 => "API rate limit exceeded for client_789",
            16 => "Garbage collection cycle completed in 250ms",
            17 => "System backup started (estimated time: 15m)",
            18 => "Certificate expiring in 30 days (domain.com)",
            _ => "Configuration loaded from /etc/config.json",
        };

        // Generate a log line with timestamp, level, and message
        writeln!(
            file,
            "2025-03-{:02} {:02}:{:02}:{:02},{:03} [{}] {}",
            (i % 31) + 1,           // Day (1-31)
            (i / 3600) % 24,         // Hour (0-23)
            (i / 60) % 60,           // Minute (0-59)
            i % 60,                  // Second (0-59)
            i % 1000,                // Millisecond (0-999)
            level,
            message
        )?;
    }

    Ok(())
}