use crate::parser::LogParser;
use lazy_static::lazy_static;
use memchr;
use memmap2::Mmap;
use rayon::prelude::*;
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Constants
const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB for better SIMD efficiency
const MAX_UNIQUE_LINES: usize = 10000; // Maximum unique lines to store

// Pre-compiled common regex patterns
lazy_static! {
    static ref LEVEL_REGEX: Regex = Regex::new(
        r"\[((?i)ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE)]|(?i:ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE):"
    ).unwrap();

    static ref TIMESTAMP_REGEX: Regex = Regex::new(
        r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})"
    ).unwrap();

    static ref ERROR_TYPE_REGEX: Regex = Regex::new(
        r"([A-Za-z]+Exception|[A-Za-z]+Error|[A-Za-z]+\s+timeout|Connection timeout|500 Internal Server Error|401 Unauthorized|503 Service Unavailable)"
    ).unwrap();
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AnalysisResult {
    pub matched_lines: Vec<String>,
    pub line_counts: FxHashMap<String, usize>,
    pub count: usize,
    pub time_trends: FxHashMap<String, usize>,
    pub levels_count: FxHashMap<String, usize>,
    pub error_types: FxHashMap<String, usize>,
    pub unique_messages: FxHashSet<String>,
    pub deduplicated: bool,
}

// Pattern matcher trait for polymorphism
pub trait PatternMatcher: Send + Sync {
    fn is_match(&self, text: &str) -> bool;
}

// Fast literal matching
pub struct LiteralMatcher {
    pattern: String,
}

impl LiteralMatcher {
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
        }
    }
}

impl PatternMatcher for LiteralMatcher {
    fn is_match(&self, text: &str) -> bool {
        // Standard string contains method
        text.contains(&self.pattern)
    }
}

// Regex-based matching for complex patterns
pub struct RegexMatcher {
    regex: Regex,
}

impl RegexMatcher {
    pub fn new(pattern: &str) -> Self {
        Self {
            regex: Regex::new(pattern).unwrap(),
        }
    }
}

impl PatternMatcher for RegexMatcher {
    fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}

pub struct LogAnalyzer {
    pub(crate) pattern_matcher: Option<Box<dyn PatternMatcher + Send + Sync>>,
    pub(crate) level_filter_lowercase: Option<String>,
    pub(crate) parser: Option<Arc<dyn LogParser>>,
    pub(crate) field_filters: FxHashMap<String, String>,
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            pattern_matcher: None,
            level_filter_lowercase: None,
            parser: None,
            field_filters: FxHashMap::default(),
        }
    }

    /// Set field filters from command line arguments in format "field=value"
    pub fn set_field_filters(&mut self, field_filters: Vec<String>) {
        for filter in field_filters {
            if let Some(separator_pos) = filter.find('=') {
                let field = filter[..separator_pos].trim().to_string();
                let value = filter[separator_pos + 1..].trim().to_string();
                self.field_filters.insert(field, value);
            }
        }
    }

    /// Helper function to check if a line contains a specific field filter
    fn line_contains_filter(&self, line: &str, key: &str, value: &str) -> bool {
        // Check if the line contains both key and value, case-insensitive
        let line_lower = line.to_lowercase();
        let key_lower = key.to_lowercase();
        let value_lower = value.to_lowercase();

        line_lower.contains(&key_lower) && line_lower.contains(&value_lower)
    }

    /// Check if parsed log line matches all field filters
    fn matches_field_filters(&self, line: &str, parsed: &crate::parser::ParsedLogLine) -> bool {
        if self.field_filters.is_empty() {
            return true; // No field filters, so it matches
        }

        // Check each field filter against the parsed line's fields
        for (filter_key, filter_value) in &self.field_filters {
            // Trim whitespace and normalize the comparison
            let normalized_filter_key = filter_key.trim();
            let normalized_filter_value = filter_value.trim();

            // Check the parsed fields first (works for JSON)
            if let Some(field_value) = parsed.fields.get(normalized_filter_key) {
                // Compare trimmed and lowercased values
                if field_value.trim().to_lowercase() != normalized_filter_value.to_lowercase() {
                    return false;
                }
            } else {
                // If not found in parsed fields, try a fallback line search
                if !self.line_contains_filter(line, normalized_filter_key, normalized_filter_value)
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn set_parser(&mut self, parser: Arc<dyn LogParser>) {
        self.parser = Some(parser);
    }

    // Configure analyzer with patterns - always using optimized approach
    pub fn configure(&mut self, pattern: Option<&str>, level_filter: Option<&str>) {
        // Create appropriate pattern matcher with SIMD acceleration when possible
        self.pattern_matcher = pattern.map(|p| {
            if Self::is_complex_pattern(p) {
                Box::new(RegexMatcher::new(p)) as Box<dyn PatternMatcher + Send + Sync>
            } else {
                // Use SIMD-accelerated matcher when supported
                #[cfg(feature = "simd_acceleration")]
                {
                    use crate::accelerated::PatternMatcherFactory;
                    PatternMatcherFactory::create(p)
                }

                #[cfg(not(feature = "simd_acceleration"))]
                {
                    Box::new(LiteralMatcher::new(p)) as Box<dyn PatternMatcher + Send + Sync>
                }
            }
        });

        // Store level filter in lowercase for fast comparison
        self.level_filter_lowercase = level_filter.map(|l| l.to_lowercase());
    }

    // Check if pattern is complex and needs regex
    fn is_complex_pattern(pattern: &str) -> bool {
        pattern.contains(|c: char| {
            c == '*'
                || c == '?'
                || c == '['
                || c == '('
                || c == '|'
                || c == '+'
                || c == '.'
                || c == '^'
                || c == '$'
                || c == '\\'
        })
    }

    // Core line analysis
    pub fn analyze_line(
        &self,
        line: &str,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        _collect_stats: bool,
    ) -> Option<(String, String, Option<String>)> {
        // If a parser is set, use it first to parse the line
        let parsed_line = if let Some(parser) = &self.parser {
            parser.parse_line(line)
        } else {
            crate::parser::ParsedLogLine::default()
        };

        // Apply field filters
        if !self.matches_field_filters(line, &parsed_line) {
            return None;
        }

        // Determine the level
        let level = parsed_line
            .level
            .clone() // Add .clone() here to avoid moving
            .unwrap_or_else(|| {
                LEVEL_REGEX
                    .captures(line)
                    .and_then(|caps| {
                        caps.get(1).map_or_else(
                            || caps.get(0).map(|m| m.as_str().to_uppercase()),
                            |m| Some(m.as_str().to_uppercase()),
                        )
                    })
                    .unwrap_or_default()
            });

        // Apply level filter
        let level_matches = match level_filter {
            None => true,
            Some(filter_level) => {
                !level.is_empty() && level.to_uppercase() == filter_level.to_uppercase()
            }
        };

        // Apply pattern matching
        let pattern_matches = match pattern {
            None => {
                if let Some(matcher) = &self.pattern_matcher {
                    matcher.is_match(line)
                } else {
                    true
                }
            }
            Some(re) => re.is_match(line),
        };

        // Check field filters if set
        let field_matches = self.matches_field_filters(line, &parsed_line);

        // Combine all filters
        if level_matches && pattern_matches && field_matches {
            // Determine timestamp
            let timestamp = if collect_trends {
                // Try to use parsed timestamp from JSON first
                parsed_line.timestamp.or_else(|| {
                    // Otherwise extract from line
                    TIMESTAMP_REGEX
                        .captures(line)
                        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
                })
            } else {
                None
            };

            Some((line.to_string(), level, timestamp))
        } else {
            None
        }
    }

    // Process chunk data efficiently
    pub fn process_chunk_data(
        &self,
        data: &[u8],
        result: &mut AnalysisResult,
        collect_trends: bool,
        collect_stats: bool,
    ) {
        for line in data.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
            // Convert line to string, skip if invalid UTF-8
            let line_str = match std::str::from_utf8(line) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Match line against all filters and conditions
            if let Some((matched_line, level, timestamp)) = self.analyze_line(
                line_str,
                None, // No specific regex pattern
                self.level_filter_lowercase.as_deref(),
                collect_trends,
                collect_stats,
            ) {
                // Increment count
                result.count += 1;

                // Manage line counts for deduplication
                let line_count_entry = result.line_counts.entry(matched_line.clone()).or_insert(0);
                *line_count_entry += 1;

                // Add to matched lines if within unique lines limit
                if result.matched_lines.len() < MAX_UNIQUE_LINES {
                    result.matched_lines.push(matched_line.clone());
                }

                // Collect time trends if requested
                if collect_trends {
                    if let Some(ts) = timestamp {
                        let hour = if ts.len() >= 13 {
                            ts[0..13].to_string()
                        } else {
                            ts
                        };
                        *result.time_trends.entry(hour).or_insert(0) += 1;
                    }
                }

                // Collect stats if requested
                if collect_stats {
                    // Collect level counts
                    *result.levels_count.entry(level).or_insert(0) += 1;

                    // Collect error types
                    if let Some(error_type) = self.extract_error_type(line_str) {
                        *result.error_types.entry(error_type).or_insert(0) += 1;
                    }

                    // Collect unique messages
                    if let Some(message) =
                        matched_line.split(']').nth(1).map(|s| s.trim().to_string())
                    {
                        result.unique_messages.insert(message);
                    } else {
                        result.unique_messages.insert(matched_line);
                    }
                }
            }
        }
    }

    // Helper method to extract error type - made public for testing
    pub fn extract_error_type(&self, line: &str) -> Option<String> {
        ERROR_TYPE_REGEX
            .captures(line)
            .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
    }

    // Unified memory-mapped file processing that automatically uses parallel processing for large files
    pub fn analyze_mmap(
        &mut self,
        mmap: &Mmap,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
        force_parallel: bool,
    ) -> AnalysisResult {
        // Configure with pattern if provided
        if let Some(pat) = pattern {
            self.configure(Some(&pat.to_string()), level_filter);
        } else if level_filter.is_some() {
            self.configure(None, level_filter);
        }

        if force_parallel {
            // Parallel processing for large files
            self.analyze_mmap_parallel(mmap, collect_trends, collect_stats)
        } else {
            // Sequential processing for small files
            self.analyze_mmap_sequential(mmap, collect_trends, collect_stats)
        }
    }

    // Private method for sequential processing
    fn analyze_mmap_sequential(
        &self,
        mmap: &Mmap,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Initialize result
        let mut result = AnalysisResult {
            matched_lines: Vec::with_capacity(1000),
            line_counts: FxHashMap::default(),
            count: 0,
            time_trends: FxHashMap::default(),
            levels_count: FxHashMap::default(),
            error_types: FxHashMap::default(),
            unique_messages: FxHashSet::default(),
            deduplicated: true,
        };

        // Buffer for handling partial lines between chunks
        let mut pending_line = Vec::with_capacity(8192);

        // Process file in chunks - use larger chunk size for SIMD efficiency
        let mut position = 0;

        while position < mmap.len() {
            // Determine chunk boundaries
            let chunk_end = std::cmp::min(position + CHUNK_SIZE, mmap.len());
            let chunk = &mmap[position..chunk_end];

            // Use memchr for fast newline search (SIMD-accelerated)
            let last_newline = if chunk_end < mmap.len() {
                match memchr::memchr_iter(b'\n', chunk).last() {
                    Some(pos) => pos + 1, // Include the newline
                    None => 0,            // No newline found in chunk
                }
            } else {
                chunk.len() // Last chunk, process everything
            };

            // Prepare data to process (pending line + complete lines)
            let mut process_data = Vec::with_capacity(pending_line.len() + last_newline);
            process_data.extend_from_slice(&pending_line);
            process_data.extend_from_slice(&chunk[..last_newline]);

            // Save incomplete line for next chunk
            pending_line.clear();
            if last_newline < chunk.len() {
                pending_line.extend_from_slice(&chunk[last_newline..]);
            }

            // Process the lines in this chunk
            self.process_chunk_data(&process_data, &mut result, collect_trends, collect_stats);

            // Move to next chunk
            position += last_newline;
        }

        // Process any remaining data
        if !pending_line.is_empty() {
            self.process_chunk_data(&pending_line, &mut result, collect_trends, collect_stats);
        }

        result
    }

    // Public method for parallel processing
    pub fn analyze_mmap_parallel(
        &mut self,
        mmap: &Mmap,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Create thread-safe shared analyzer
        let analyzer = Arc::new(self);

        // Split the file into chunks for parallel processing
        // Use SIMD to efficiently find newlines for chunk boundaries
        let mut chunks = Vec::new();
        let mut chunk_start = 0;

        // Identify chunk boundaries at newlines
        while chunk_start < mmap.len() {
            let chunk_end_approx = std::cmp::min(chunk_start + CHUNK_SIZE, mmap.len());

            // Find the next newline after the approximate chunk end
            let chunk_end = if chunk_end_approx < mmap.len() {
                let search_end = std::cmp::min(chunk_end_approx + 2000, mmap.len());
                match memchr::memchr(b'\n', &mmap[chunk_end_approx..search_end]) {
                    Some(pos) => chunk_end_approx + pos + 1, // Include the newline
                    None => chunk_end_approx, // No newline found, use approximate end
                }
            } else {
                mmap.len() // Last chunk goes to the end
            };

            // Add chunk to list
            chunks.push((chunk_start, chunk_end));
            chunk_start = chunk_end;
        }

        // Process chunks in parallel
        let results: Vec<AnalysisResult> = chunks
            .par_iter()
            .map(|&(start, end)| {
                let analyzer = Arc::clone(&analyzer);
                let chunk = &mmap[start..end];
                let mut result = AnalysisResult {
                    deduplicated: true,
                    ..Default::default()
                };
                analyzer.process_chunk_data(chunk, &mut result, collect_trends, collect_stats);
                result
            })
            .collect();

        // Merge results
        let mut final_result = AnalysisResult {
            deduplicated: true,
            ..Default::default()
        };

        for result in results {
            final_result.count += result.count;

            // Merge line counts for deduplication
            for (line, count) in result.line_counts {
                let current_count = final_result.line_counts.entry(line.clone()).or_insert(0);
                *current_count += count;

                // Only add to matched_lines if we haven't exceeded our limit
                if final_result.matched_lines.len() < MAX_UNIQUE_LINES
                    && !final_result.matched_lines.contains(&line)
                {
                    final_result.matched_lines.push(line);
                }
            }

            // Merge time trends
            for (timestamp, count) in result.time_trends {
                *final_result.time_trends.entry(timestamp).or_insert(0) += count;
            }

            // Merge level counts
            for (level, count) in result.levels_count {
                *final_result.levels_count.entry(level).or_insert(0) += count;
            }

            // Merge error types
            for (error_type, count) in result.error_types {
                *final_result.error_types.entry(error_type).or_insert(0) += count;
            }

            // Merge unique messages
            final_result.unique_messages.extend(result.unique_messages);
        }

        final_result
    }
}
