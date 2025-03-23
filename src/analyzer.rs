use regex::Regex;
use serde::{Deserialize, Serialize};
use rustc_hash::{FxHashMap, FxHashSet};
use rayon::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub matched_lines: Vec<String>,
    pub count: usize,
    pub time_trends: FxHashMap<String, usize>,
    pub levels_count: FxHashMap<String, usize>,
    pub error_types: FxHashMap<String, usize>,
    pub unique_messages: FxHashSet<String>,
}

pub struct LogAnalyzer {
    level_regex: Regex,
    timestamp_regex: Regex,
    error_type_regex: Regex,
}

// Structure to hold parsed data from a line
// This avoids repeated regex parsing and string allocations
struct ParsedLine<'a> {
    level: &'a str,
    timestamp: Option<&'a str>,
    error_type: Option<String>,
    message: Option<&'a str>,
}

impl Default for LogAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAnalyzer {
    pub fn new() -> Self {
        LogAnalyzer {
            level_regex: Regex::new(r"\[((?i)ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE)]|(?i:ERROR|WARN|INFO|DEBUG|TRACE|SEVERE|WARNING|FINE):").expect("Failed to create level regex"),
            timestamp_regex: Regex::new(r"(\d{4}-\d{2}-\d{2}\s+\d{2}:\d{2}:\d{2})").expect("Failed to create timestamp regex"),
            error_type_regex: Regex::new(r"([A-Za-z]+Exception|[A-Za-z]+Error|[A-Za-z]+\s+timeout|Connection timeout|500 Internal Server Error|401 Unauthorized|503 Service Unavailable)").expect("Failed to create error regex"),
        }
    }

    // Optimized parsing: Extract all needed data in a single pass
    fn parse_line<'a>(&self, line: &'a str, need_timestamp: bool, need_stats: bool) -> ParsedLine<'a> {
        // Cache regex captures to avoid multiple regex runs on the same line
        let level_caps = self.level_regex.captures(line);

        // Only run timestamp regex if needed
        let timestamp_caps = if need_timestamp {
            self.timestamp_regex.captures(line)
        } else {
            None
        };

        // Only run error_type regex if needed for stats
        let error_type_caps = if need_stats {
            self.error_type_regex.captures(line)
        } else {
            None
        };

        // Extract log level
        let level = if let Some(caps) = &level_caps {
            caps.get(1)
                .map_or_else(|| caps.get(0).map_or("", |m| m.as_str()), |m| m.as_str())
        } else {
            ""
        };

        // Extract timestamp
        let timestamp = if let Some(caps) = timestamp_caps {
            if let Some(timestamp) = caps.get(1) {
                let timestamp_str = timestamp.as_str();
                if timestamp_str.len() >= 13 {
                    Some(&timestamp_str[0..13])
                } else {
                    Some(timestamp_str)
                }
            } else {
                None
            }
        } else {
            None
        };

        // Extract error type and message for stats
        let (error_type, message) = if need_stats {
            // Process error type
            let err_type = if let Some(caps) = error_type_caps {
                if let Some(error_type) = caps.get(1) {
                    Some(error_type.as_str().to_string())
                } else {
                    None
                }
            } else if line.contains("Failed to") {
                // Extract the specific failure reason
                let parts: Vec<&str> = line.split("Failed to ").collect();
                if parts.len() > 1 {
                    let action_parts: Vec<&str> = parts[1].split(':').collect();
                    if !action_parts.is_empty() {
                        let action = action_parts[0].trim();
                        Some(format!("Failed to {}", action))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Extract message
            let msg = self.extract_message(line);

            (err_type, msg)
        } else {
            (None, None)
        };

        ParsedLine {
            level,
            timestamp,
            error_type,
            message,
        }
    }

    // Extract message with string slices instead of new Strings
    fn extract_message<'a>(&self, line: &'a str) -> Option<&'a str> {
        let parts: Vec<&str> = line.splitn(3, " - ").collect();
        if parts.len() >= 3 {
            Some(parts[2])
        } else if parts.len() == 2 {
            Some(parts[1])
        } else {
            Some(line)
        }
    }

    // For API compatibility
    pub fn analyze_line(
        &self,
        line: &str,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> Option<(String, String, Option<String>)> {
        // Parse line once to extract all needed data
        let parsed = self.parse_line(line, collect_trends, collect_stats);

        // Apply filters
        let level_matches = match level_filter {
            None => true,
            Some(filter_level) => !parsed.level.is_empty() &&
                parsed.level.to_uppercase() == filter_level.to_uppercase()
        };

        let pattern_matches = match pattern {
            None => true,
            Some(re) => re.is_match(line)
        };

        if level_matches && pattern_matches {
            // Format timestamp for return value
            let timestamp = parsed.timestamp.map(String::from);

            return Some((line.to_string(), parsed.level.to_uppercase(), timestamp));
        }

        None
    }

    pub fn extract_error_type(&self, line: &str) -> Option<String> {
        let parsed = self.parse_line(line, false, true);
        parsed.error_type
    }

    pub fn analyze_lines<I>(
        &self,
        lines: I,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult
    where
        I: Iterator<Item = String>,
    {
        // More intelligent capacity hints based on filtering
        // Start with moderately large number, can be adjusted based on benchmarks
        let lines_hint = 10_000;

        // If we have filtering, expect fewer matches
        let estimated_matches = if pattern.is_some() || level_filter.is_some() {
            lines_hint / 5  // Estimate 20% match rate when filters are applied
        } else {
            lines_hint  // No filters means all lines match
        };

        // Initialize hash maps with appropriate capacities
        let mut result = AnalysisResult {
            matched_lines: Vec::with_capacity(estimated_matches),
            count: 0,
            time_trends: FxHashMap::with_capacity_and_hasher(24, Default::default()), // Most logs span hours (24 max)
            levels_count: FxHashMap::with_capacity_and_hasher(5, Default::default()), // Most logs have ~5 levels
            error_types: FxHashMap::with_capacity_and_hasher(50, Default::default()), // Reasonable number of error types
            unique_messages: FxHashSet::with_capacity_and_hasher(estimated_matches, Default::default()),
        };

        // Process each line efficiently
        for line in lines {
            // Pre-check for pattern to avoid parsing unnecessarily
            if let Some(pat) = pattern {
                if !pat.is_match(&line) {
                    continue;
                }
            }

            // Parse the line once, extracting all needed data
            let parsed = self.parse_line(&line, collect_trends, collect_stats);

            // Check level filter
            let level_matches = match level_filter {
                None => true,
                Some(filter_level) => !parsed.level.is_empty() &&
                    parsed.level.to_uppercase() == filter_level.to_uppercase()
            };

            if !level_matches {
                continue;
            }

            // We have a match - add to results
            result.matched_lines.push(line.clone());
            result.count += 1;

            // Add stats if requested
            if collect_stats {
                if !parsed.level.is_empty() {
                    let level_upper = parsed.level.to_uppercase();
                    *result.levels_count.entry(level_upper).or_insert(0) += 1;
                }

                if let Some(error_type) = parsed.error_type {
                    *result.error_types.entry(error_type).or_insert(0) += 1;
                }

                if let Some(message) = parsed.message {
                    result.unique_messages.insert(message.to_string());
                }
            }

            // Add to time trends if requested
            if collect_trends {
                if let Some(hour) = parsed.timestamp {
                    *result.time_trends.entry(hour.to_string()).or_insert(0) += 1;
                }
            }
        }

        result
    }

    /// Analyze lines in parallel for large files
    /// This method processes chunks of lines in parallel to improve performance on multi-core systems
    pub fn analyze_lines_parallel(
        &self,
        lines: Vec<String>,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Determine optimal chunk size based on number of lines
        // Smaller chunks for more cores, but not too small to avoid overhead
        let num_cpus = rayon::current_num_threads();
        let chunk_size = std::cmp::max(1000, lines.len() / (num_cpus * 2));

        // Process chunks in parallel
        let chunk_results: Vec<AnalysisResult> = lines
            .par_chunks(chunk_size)
            .map(|chunk| {
                let analyzer = LogAnalyzer::new();
                analyzer.analyze_lines(chunk.iter().cloned(), pattern, level_filter, collect_trends, collect_stats)
            })
            .collect();

        // Merge results from all chunks
        self.merge_results(chunk_results)
    }

    /// Merge multiple AnalysisResult objects into a single result
    pub fn merge_results(&self, results: Vec<AnalysisResult>) -> AnalysisResult {
        if results.is_empty() {
            return AnalysisResult {
                matched_lines: Vec::new(),
                count: 0,
                time_trends: FxHashMap::default(),
                levels_count: FxHashMap::default(),
                error_types: FxHashMap::default(),
                unique_messages: FxHashSet::default(),
            };
        }

        // Initialize with the first result
        let mut merged = results[0].clone();

        // Merge remaining results
        for result in results.iter().skip(1) {
            // Add matched lines and count
            merged.matched_lines.extend_from_slice(&result.matched_lines);
            merged.count += result.count;

            // Merge time trends
            for (timestamp, count) in &result.time_trends {
                *merged.time_trends.entry(timestamp.clone()).or_insert(0) += count;
            }

            // Merge level counts
            for (level, count) in &result.levels_count {
                *merged.levels_count.entry(level.clone()).or_insert(0) += count;
            }

            // Merge error types
            for (error_type, count) in &result.error_types {
                *merged.error_types.entry(error_type.clone()).or_insert(0) += count;
            }

            // Merge unique messages
            for message in &result.unique_messages {
                merged.unique_messages.insert(message.clone());
            }
        }

        merged
    }
}