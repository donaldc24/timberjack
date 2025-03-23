use regex::Regex;
use serde::{Deserialize, Serialize};
use rustc_hash::{FxHashMap, FxHashSet};
use memmap2::Mmap;
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

const CHUNK_SIZE: usize = 1_048_576; // 1MB

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub matched_lines: Vec<String>,
    pub count: usize,
    pub time_trends: FxHashMap<String, usize>,
    pub levels_count: FxHashMap<String, usize>,
    pub error_types: FxHashMap<String, usize>,
    pub unique_messages: FxHashSet<String>,
}

// Add Default implementation for AnalysisResult
impl Default for AnalysisResult {
    fn default() -> Self {
        AnalysisResult {
            matched_lines: Vec::new(),
            count: 0,
            time_trends: FxHashMap::default(),
            levels_count: FxHashMap::default(),
            error_types: FxHashMap::default(),
            unique_messages: FxHashSet::default(),
        }
    }
}

pub struct LogAnalyzer {
    level_regex: Regex,
    timestamp_regex: Regex,
    error_type_regex: Regex,
}

// Structure to hold parsed data from a line
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

    // Parse line once to extract all needed data
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

    // For API compatibility - analyze a single line
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

    // Original method for iterative processing (keeps existing functionality)
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

    // Parallel processing for collections of lines
    pub fn analyze_lines_parallel(
        &self,
        lines: Vec<String>,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Use thread-safe containers for shared data
        let matched_lines = Arc::new(Mutex::new(Vec::with_capacity(lines.len() / 10)));
        let count = Arc::new(Mutex::new(0));

        // Each thread will have its own copy of these maps for performance
        let time_trends = Arc::new(Mutex::new(FxHashMap::default()));
        let levels_count = Arc::new(Mutex::new(FxHashMap::default()));
        let error_types = Arc::new(Mutex::new(FxHashMap::default()));
        let unique_messages = Arc::new(Mutex::new(FxHashSet::default()));

        // Process lines in parallel
        lines.par_iter().for_each(|line| {
            // Pre-check for pattern to avoid parsing unnecessarily
            if let Some(pat) = pattern {
                if !pat.is_match(line) {
                    return;
                }
            }

            // Parse the line once, extracting all needed data
            let parsed = self.parse_line(line, collect_trends, collect_stats);

            // Check level filter
            let level_matches = match level_filter {
                None => true,
                Some(filter_level) => !parsed.level.is_empty() &&
                    parsed.level.to_uppercase() == filter_level.to_uppercase()
            };

            if !level_matches {
                return;
            }

            // We have a match - add to results
            {
                let mut matched = matched_lines.lock().unwrap();
                matched.push(line.clone());
            }

            {
                let mut cnt = count.lock().unwrap();
                *cnt += 1;
            }

            // Add stats if requested
            if collect_stats {
                if !parsed.level.is_empty() {
                    let level_upper = parsed.level.to_uppercase();
                    let mut levels = levels_count.lock().unwrap();
                    *levels.entry(level_upper).or_insert(0) += 1;
                }

                if let Some(error_type) = parsed.error_type {
                    let mut errors = error_types.lock().unwrap();
                    *errors.entry(error_type).or_insert(0) += 1;
                }

                if let Some(message) = parsed.message {
                    let mut messages = unique_messages.lock().unwrap();
                    messages.insert(message.to_string());
                }
            }

            // Add to time trends if requested
            if collect_trends {
                if let Some(hour) = parsed.timestamp {
                    let mut trends = time_trends.lock().unwrap();
                    *trends.entry(hour.to_string()).or_insert(0) += 1;
                }
            }
        });

        // Combine results
        AnalysisResult {
            matched_lines: Arc::try_unwrap(matched_lines).unwrap().into_inner().unwrap(),
            count: Arc::try_unwrap(count).unwrap().into_inner().unwrap(),
            time_trends: Arc::try_unwrap(time_trends).unwrap().into_inner().unwrap(),
            levels_count: Arc::try_unwrap(levels_count).unwrap().into_inner().unwrap(),
            error_types: Arc::try_unwrap(error_types).unwrap().into_inner().unwrap(),
            unique_messages: Arc::try_unwrap(unique_messages).unwrap().into_inner().unwrap(),
        }
    }

    // Memory-mapped file processing (sequential)
    pub fn analyze_mmap(
        &self,
        mmap: &Mmap,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Initialize result
        let mut result = AnalysisResult {
            matched_lines: Vec::with_capacity(1000),
            count: 0,
            time_trends: FxHashMap::default(),
            levels_count: FxHashMap::default(),
            error_types: FxHashMap::default(),
            unique_messages: FxHashSet::default(),
        };

        // Buffer for handling partial lines between chunks
        let mut pending_line = Vec::with_capacity(4096);

        // Process file in chunks
        let mut position = 0;
        while position < mmap.len() {
            // Determine chunk boundaries
            let chunk_end = std::cmp::min(position + CHUNK_SIZE, mmap.len());
            let chunk = &mmap[position..chunk_end];

            // Find the last complete line in this chunk
            let last_newline = if chunk_end < mmap.len() {
                match chunk.iter().rposition(|&b| b == b'\n') {
                    Some(pos) => pos + 1, // Include the newline
                    None => 0, // No newline found in chunk
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
            self.process_chunk_data(&process_data, &mut result, pattern,
                                    level_filter, collect_trends, collect_stats);

            // Move to next chunk
            position += last_newline;
        }

        // Process any remaining data
        if !pending_line.is_empty() {
            self.process_chunk_data(&pending_line, &mut result, pattern,
                                    level_filter, collect_trends, collect_stats);
        }

        result
    }

    // Memory-mapped file processing (parallel)
    pub fn analyze_mmap_parallel(
        &self,
        mmap: &Mmap,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) -> AnalysisResult {
        // Split the file into chunks for parallel processing
        let mut chunks = Vec::new();
        let mut chunk_start = 0;

        // Identify chunk boundaries at newlines
        while chunk_start < mmap.len() {
            let chunk_end_approx = std::cmp::min(chunk_start + CHUNK_SIZE, mmap.len());

            // Find the next newline after the approximate chunk end
            let chunk_end = if chunk_end_approx < mmap.len() {
                let search_end = std::cmp::min(chunk_end_approx + 1000, mmap.len());
                match mmap[chunk_end_approx..search_end].iter().position(|&b| b == b'\n') {
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
        let results: Vec<AnalysisResult> = chunks.par_iter()
            .map(|&(start, end)| {
                let chunk = &mmap[start..end];
                let mut result = AnalysisResult::default();
                self.process_chunk_data(chunk, &mut result, pattern,
                                        level_filter, collect_trends, collect_stats);
                result
            })
            .collect();

        // Merge results
        let mut final_result = AnalysisResult::default();
        for result in results {
            final_result.matched_lines.extend(result.matched_lines);
            final_result.count += result.count;

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

    // Helper method to process chunk data
    fn process_chunk_data(
        &self,
        data: &[u8],
        result: &mut AnalysisResult,
        pattern: Option<&Regex>,
        level_filter: Option<&str>,
        collect_trends: bool,
        collect_stats: bool,
    ) {
        // Split data into lines
        for line in data.split(|&b| b == b'\n').filter(|l| !l.is_empty()) {
            // Convert line to string, skip if invalid UTF-8
            let line_str = match std::str::from_utf8(line) {
                Ok(s) => s,
                Err(_) => continue,
            };

            // Quick pattern check before more expensive operations
            if let Some(pat) = pattern {
                if !pat.is_match(line_str) {
                    continue;
                }
            }

            // Parse line and apply filters
            let parsed = self.parse_line(line_str, collect_trends, collect_stats);

            // Check level filter
            let level_matches = match level_filter {
                None => true,
                Some(filter) => !parsed.level.is_empty() &&
                    parsed.level.to_uppercase() == filter.to_uppercase()
            };

            if !level_matches {
                continue;
            }

            // We have a match - add to results
            result.matched_lines.push(line_str.to_string());
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

            // Add time trend if requested
            if collect_trends {
                if let Some(timestamp) = parsed.timestamp {
                    *result.time_trends.entry(timestamp.to_string()).or_insert(0) += 1;
                }
            }
        }
    }
}