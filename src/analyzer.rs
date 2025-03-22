use regex::Regex;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub matched_lines: Vec<String>,
    pub count: usize,
    pub time_trends: HashMap<String, usize>,
    pub levels_count: HashMap<String, usize>,
    pub error_types: HashMap<String, usize>,
    pub unique_messages: HashSet<String>,
}

pub struct LogAnalyzer {
    level_regex: Regex,
    timestamp_regex: Regex,
    error_type_regex: Regex,
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
            // Updated regex to better capture meaningful error types
            error_type_regex: Regex::new(r"([A-Za-z]+Exception|[A-Za-z]+Error|[A-Za-z]+\s+timeout|Connection timeout|500 Internal Server Error|401 Unauthorized|503 Service Unavailable)").expect("Failed to create error regex"),
        }
    }

    pub fn analyze_line(
        &self,
        line: &str,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        _collect_stats: bool,
    ) -> Option<(String, String, Option<String>)> {
        // Extract log level if present
        let mut found_level = String::new();
        if let Some(caps) = self.level_regex.captures(line) {
            found_level = caps
                .get(1)
                .map_or_else(|| caps.get(0).map_or("", |m| m.as_str()), |m| m.as_str())
                .to_uppercase();
        }

        // Check if we need to filter by level
        let level_matches = level_filter.is_none_or(|filter_level| {
            !found_level.is_empty() && found_level == filter_level.to_uppercase()
        });

        // Check if pattern matches (if we have a pattern)
        let pattern_matches = pattern.is_none_or(|re| re.is_match(line));

        // Only process if both conditions match
        if level_matches && pattern_matches {
            // Extract timestamp if needed for trends
            let timestamp = if collect_trends {
                self.extract_timestamp(line)
            } else {
                None
            };

            return Some((line.to_string(), found_level, timestamp));
        }

        None
    }

    fn extract_timestamp(&self, line: &str) -> Option<String> {
        if let Some(caps) = self.timestamp_regex.captures(line) {
            if let Some(timestamp) = caps.get(1) {
                // Get the hour part for grouping (first 13 chars: YYYY-MM-DD HH)
                let timestamp_str = timestamp.as_str();
                if timestamp_str.len() >= 13 {
                    return Some(timestamp_str[0..13].to_string());
                } else {
                    return Some(timestamp_str.to_string());
                }
            }
        }
        None
    }

    pub fn extract_error_type(&self, line: &str) -> Option<String> {
        // First try the regex pattern
        if let Some(caps) = self.error_type_regex.captures(line) {
            if let Some(error_type) = caps.get(1) {
                return Some(error_type.as_str().to_string());
            }
        }

        // If regex fails, try a more heuristic approach for "Failed to..." patterns
        if line.contains("Failed to") {
            // Extract the specific failure reason
            let parts: Vec<&str> = line.split("Failed to ").collect();
            if parts.len() > 1 {
                let action_parts: Vec<&str> = parts[1].split(":").collect();
                if !action_parts.is_empty() {
                    let action = action_parts[0].trim();
                    return Some(format!("Failed to {}", action));
                }
            }
        }

        None
    }

    // Extract the actual message content without timestamp and level
    fn extract_message(&self, line: &str) -> String {
        // Match pattern like: "YYYY-MM-DD HH:MM:SS,mmm [LEVEL] Service - Message"
        // We want to extract just the "Message" part

        let parts: Vec<&str> = line.splitn(3, " - ").collect();
        if parts.len() >= 3 {
            return parts[2].to_string();
        } else if parts.len() == 2 {
            return parts[1].to_string();
        }

        // Fallback to the original line if it doesn't match the expected format
        line.to_string()
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
        let mut result = AnalysisResult {
            matched_lines: Vec::new(),
            count: 0,
            time_trends: HashMap::new(),
            levels_count: HashMap::new(),
            error_types: HashMap::new(),
            unique_messages: HashSet::new(),
        };

        for line in lines {
            if let Some((matched_line, level, timestamp)) =
                self.analyze_line(&line, pattern, level_filter, collect_trends, collect_stats)
            {
                // Store matched line
                result.matched_lines.push(matched_line.clone());
                result.count += 1;

                // Stats collection
                if collect_stats {
                    // Count by level
                    if !level.is_empty() {
                        *result.levels_count.entry(level).or_insert(0) += 1;
                    }

                    // Extract error types
                    if let Some(error_type) = self.extract_error_type(&matched_line) {
                        *result.error_types.entry(error_type).or_insert(0) += 1;
                    }

                    // Store unique messages - FIXED: Use just the message content, not the whole line
                    let message_content = self.extract_message(&matched_line);
                    result.unique_messages.insert(message_content);
                }

                // Time trends collection
                if collect_trends {
                    if let Some(hour) = timestamp {
                        *result.time_trends.entry(hour).or_insert(0) += 1;
                    }
                }
            }
        }

        result
    }
}

// Extension trait to add is_none_or method
trait OptionExt<T> {
    fn is_none_or<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn is_none_or<F>(&self, f: F) -> bool
    where
        F: FnOnce(&T) -> bool,
    {
        match self {
            None => true,
            Some(value) => f(value),
        }
    }
}