use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::fs::File;
use std::io::{Read, Write};
use tempfile::NamedTempFile;
use timber_rs::accelerated::SimdLiteralMatcher;
use timber_rs::analyzer::{LiteralMatcher, PatternMatcher};

// Create sample log data for benchmarking
fn create_benchmark_log(size: usize, pattern_frequency: usize) -> NamedTempFile {
    let temp_file = NamedTempFile::new().unwrap();
    let mut file = File::create(temp_file.path()).unwrap();

    // Create log lines
    for i in 0..size {
        let level = match i % 5 {
            0 => "ERROR",
            1 => "WARN",
            2 => "INFO",
            3 => "DEBUG",
            _ => "TRACE",
        };

        // Insert the target pattern at controlled frequency
        let message = if i % pattern_frequency == 0 {
            "Exception occurred in process_data"
        } else {
            "Normal log message"
        };

        writeln!(
            file,
            "2025-03-21 {:02}:{:02}:00,{:03} [{}] {}",
            (i / 60) % 24,
            i % 60,
            i % 1000,
            level,
            message
        )
        .unwrap();
    }

    temp_file
}

fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    // Create sample log data with 10,000 lines and pattern every 100 lines
    let log_file = create_benchmark_log(10_000, 100);

    // Read the file into memory
    let mut file = File::open(log_file.path()).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    // Patterns to search for
    let patterns = [
        "Exception",          // Common error term
        "process_data",       // Function name
        "Normal log message", // Common message
        "NonExistentPattern", // No matches expected
    ];

    // Benchmark standard matcher
    for pattern in &patterns {
        group.bench_with_input(
            BenchmarkId::new("standard", pattern),
            pattern,
            |b, pattern| {
                let matcher = LiteralMatcher::new(pattern);
                b.iter(|| {
                    // Count matches in content
                    let mut matches = 0;
                    for line in content.lines() {
                        if matcher.is_match(black_box(line)) {
                            matches += 1;
                        }
                    }
                    black_box(matches)
                });
            },
        );
    }

    // Benchmark SIMD matcher
    for pattern in &patterns {
        group.bench_with_input(BenchmarkId::new("simd", pattern), pattern, |b, pattern| {
            let matcher = SimdLiteralMatcher::new(pattern);
            b.iter(|| {
                // Count matches in content
                let mut matches = 0;
                for line in content.lines() {
                    if matcher.is_match(black_box(line)) {
                        matches += 1;
                    }
                }
                black_box(matches)
            });
        });
    }

    group.finish();
}

fn bench_pattern_matching_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching_large");
    group.sample_size(10); // Reduce sample size for large files

    // Create larger sample log data with 100,000 lines and pattern every 500 lines
    let log_file = create_benchmark_log(100_000, 500);

    // Read the file into memory
    let mut file = File::open(log_file.path()).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    // Patterns to search for
    let patterns = [
        "Exception", // Common error term
    ];

    // Benchmark standard matcher vs SIMD matcher
    for pattern in &patterns {
        group.bench_with_input(
            BenchmarkId::new("standard_large", pattern),
            pattern,
            |b, pattern| {
                let matcher = LiteralMatcher::new(pattern);
                b.iter(|| {
                    // Count matches in content
                    let mut matches = 0;
                    for line in content.lines() {
                        if matcher.is_match(black_box(line)) {
                            matches += 1;
                        }
                    }
                    black_box(matches)
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("simd_large", pattern),
            pattern,
            |b, pattern| {
                let matcher = SimdLiteralMatcher::new(pattern);
                b.iter(|| {
                    // Count matches in content
                    let mut matches = 0;
                    for line in content.lines() {
                        if matcher.is_match(black_box(line)) {
                            matches += 1;
                        }
                    }
                    black_box(matches)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_pattern_matching,
    bench_pattern_matching_large
);
criterion_main!(benches);
