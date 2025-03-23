use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use tempfile::NamedTempFile;

use timber_rs::analyzer::LogAnalyzer;
use regex::Regex;

// Helper function to create a large log file for benchmarking
fn create_large_log_file(lines: usize) -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();
    let mut file = File::create(temp_file.path()).unwrap();

    for i in 0..lines {
        let level = match i % 5 {
            0 => "ERROR",
            1 => "WARN",
            2 => "INFO",
            3 => "DEBUG",
            _ => "TRACE",
        };

        let message = match i % 10 {
            0 => "NullPointerException in WebController.java:42",
            1 => "Connection timeout in NetworkClient.java:86",
            2 => "Database query took 2.3s in DatabaseService.java:128",
            3 => "Application started successfully",
            4 => "Session created for user_123",
            5 => "OutOfMemoryError in SearchIndexer.java:212",
            6 => "Failed to process request: invalid parameters",
            7 => "Cache miss for key: user_profile_123",
            8 => "Authentication successful for user_123",
            _ => "Request processed in 150ms",
        };

        writeln!(
            file,
            "2025-03-21 {:02}:{:02}:00,{:03} [{}] {}",
            (i / 60) % 24,
            i % 60,
            i % 1000,
            level,
            message
        ).unwrap();
    }

    temp_file
}

fn bench_process_large_file(c: &mut Criterion) {
    // Create a file with 10,000 lines
    let temp_file = create_large_log_file(10_000);
    let file_path = temp_file.path().to_str().unwrap();
    let pattern = Regex::new("Error").unwrap();

    let mut group = c.benchmark_group("log_processing");

    // No filtering
    group.bench_function("no_filter", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, None, None, false, false);
            black_box(result);
        });
    });

    // With pattern filtering
    group.bench_function("with_pattern", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, Some(&pattern), None, false, false);
            black_box(result);
        });
    });

    // With level filtering
    group.bench_function("with_level", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, None, Some("ERROR"), false, false);
            black_box(result);
        });
    });

    // With stats collection
    group.bench_function("with_stats", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, None, None, false, true);
            black_box(result);
        });
    });

    // With time trends
    group.bench_function("with_trends", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, None, None, true, false);
            black_box(result);
        });
    });

    // With all features enabled
    group.bench_function("all_features", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, Some(&pattern), Some("ERROR"), true, true);
            black_box(result);
        });
    });

    // Test parallel processing
    group.bench_function("parallel_processing", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines()
                .filter_map(Result::ok)
                .collect();

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines_parallel(lines, Some(&pattern), Some("ERROR"), true, true);
            black_box(result);
        });
    });

    group.finish();
}

fn bench_sequential_vs_parallel(c: &mut Criterion) {
    // Create a large file with 100,000 lines for a more realistic test
    let temp_file = create_large_log_file(100_000);
    let file_path = temp_file.path().to_str().unwrap();

    let mut group = c.benchmark_group("sequential_vs_parallel");

    // Sequential processing
    group.bench_function("sequential_100k", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines = reader.lines().map(|l| l.unwrap());

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines(lines, None, None, true, true);
            black_box(result);
        });
    });

    // Parallel processing
    group.bench_function("parallel_100k", |b| {
        b.iter(|| {
            let file = File::open(file_path).unwrap();
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines()
                .filter_map(Result::ok)
                .collect();

            let analyzer = LogAnalyzer::new();
            let result = analyzer.analyze_lines_parallel(lines, None, None, true, true);
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_process_large_file, bench_sequential_vs_parallel);
criterion_main!(benches);