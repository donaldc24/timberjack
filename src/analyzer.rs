use regex::Regex;
use serde::{Deserialize, Serialize};
use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Serialize, Deserialize)]
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

    // Extract all needed data from a line at once to avoid repeated regex operations
    fn parse_line<'a>(&self, line: &'a str, need_timestamp: bool, need_stats: bool) -> ParsedLine<'a> {
        // Initialize with defaults
        let mut parsed = ParsedLine {
            level: "",
            timestamp: None,
            error_type: None,
            message: None,
        };

        // Extract log level if present
        if let Some(caps) = self.level_regex.captures(line) {
            parsed.level = caps
                .get(1)
                .map_or_else(|| caps.get(0).map_or("", |m| m.as_str()), |m| m.as_str());
        }

        // Extract timestamp only if needed
        if need_timestamp {
            if let Some(caps) = self.timestamp_regex.captures(line) {
                if let Some(timestamp) = caps.get(1) {
                    let timestamp_str = timestamp.as_str();
                    if timestamp_str.len() >= 13 {
                        parsed.timestamp = Some(&timestamp_str[0..13]);
                    } else {
                        parsed.timestamp = Some(timestamp_str);
                    }
                }
            }
        }

        // Extract message and error type only if collecting stats
        if need_stats {
            parsed.message = self.extract_message(line);

            // Error type extraction - still needs a String due to formatting in some cases
            if let Some(caps) = self.error_type_regex.captures(line) {
                if let Some(error_type) = caps.get(1) {
                    parsed.error_type = Some(error_type.as_str().to_string());
                }
            } else if line.contains("Failed to") {
                // Extract the specific failure reason
                let parts: Vec<&str> = line.split("Failed to ").collect();
                if parts.len() > 1 {
                    let action_parts: Vec<&str> = parts[1].split(':').collect();
                    if !action_parts.is_empty() {
                        let action = action_parts[0].trim();
                        parsed.error_type = Some(format!("Failed to {}", action));
                    }
                }
            }
        }

        parsed
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
        // Pre-allocate collections with capacity hints
        let estimated_lines = 1000; // Adjust based on expected input size

        let mut result = AnalysisResult {
            matched_lines: Vec::with_capacity(estimated_lines),
            count: 0,
            time_trends: FxHashMap::default(),
            levels_count: FxHashMap::default(),
            error_types: FxHashMap::default(),
            unique_messages: FxHashSet::default(),
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
}