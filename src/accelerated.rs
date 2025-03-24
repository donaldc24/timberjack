use crate::analyzer::PatternMatcher;
use memchr::memmem;
use std::sync::OnceLock;

// Feature detection flags
struct CpuFeatures {
    sse41_supported: bool,
    avx2_supported: bool,
}

static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

// Initialize CPU feature detection
fn get_cpu_features() -> &'static CpuFeatures {
    CPU_FEATURES.get_or_init(|| {
        let mut features = CpuFeatures {
            sse41_supported: false,
            avx2_supported: false,
        };

        #[cfg(target_arch = "x86_64")]
        {
            if cfg!(feature = "simd_acceleration") {
                features.sse41_supported = std::is_x86_feature_detected!("sse4.1");
                features.avx2_supported = std::is_x86_feature_detected!("avx2");
            }
        }

        features
    })
}

/// SIMD-accelerated literal matcher using memchr crate
pub struct SimdLiteralMatcher {
    // Store both string and bytes to avoid repeated conversions
    pattern_str: String,
    pattern_bytes: Vec<u8>,
}

impl SimdLiteralMatcher {
    pub fn new(pattern: &str) -> Self {
        // Ensure CPU features are detected
        get_cpu_features();

        Self {
            pattern_str: pattern.to_string(),
            pattern_bytes: pattern.as_bytes().to_vec(),
        }
    }

    // Determine if this instance can use SIMD acceleration
    fn can_use_simd(&self) -> bool {
        // Short patterns don't benefit much from SIMD, so use a threshold
        let min_pattern_length = 3;

        #[cfg(not(feature = "simd_acceleration"))]
        return false;

        #[cfg(feature = "simd_acceleration")]
        {
            // Patterns need to be long enough to benefit from SIMD
            if self.pattern_bytes.len() < min_pattern_length {
                return false;
            }

            // Return true if any SIMD features are available
            let features = get_cpu_features();
            features.sse41_supported || features.avx2_supported
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        false
    }
}

impl PatternMatcher for SimdLiteralMatcher {
    fn is_match(&self, text: &str) -> bool {
        if self.can_use_simd() {
            // SIMD path - use memchr for high-performance search
            memmem::find(text.as_bytes(), &self.pattern_bytes).is_some()
        } else {
            // Fallback path - use standard string search for short patterns
            // or when SIMD isn't available
            text.contains(&self.pattern_str)
        }
    }
}

/// Factory for creating the most appropriate pattern matcher based on the pattern
/// and available hardware capabilities.
pub struct PatternMatcherFactory;

impl PatternMatcherFactory {
    /// Creates the most optimized pattern matcher for the given pattern.
    ///
    /// This will automatically select between:
    /// - SIMD-accelerated literal matcher for simple patterns (when hardware supports it)
    /// - Standard literal matcher for simple patterns (fallback)
    /// - Regex matcher for complex patterns
    pub fn create(pattern: &str) -> Box<dyn PatternMatcher + Send + Sync> {
        if Self::is_complex_pattern(pattern) {
            // For complex patterns, use regex
            use crate::analyzer::RegexMatcher;
            Box::new(RegexMatcher::new(pattern))
        } else {
            // For simple patterns, try to use SIMD acceleration
            #[cfg(feature = "simd_acceleration")]
            {
                Box::new(SimdLiteralMatcher::new(pattern))
            }

            // If SIMD acceleration feature is disabled, use regular matcher
            #[cfg(not(feature = "simd_acceleration"))]
            {
                use crate::analyzer::LiteralMatcher;
                Box::new(LiteralMatcher::new(pattern))
            }
        }
    }

    /// Determines if a pattern is complex and requires regex capabilities
    fn is_complex_pattern(pattern: &str) -> bool {
        // Look for regex metacharacters
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
}

// SIMD-accelerated line processing utilities
pub mod line_processing {
    use memchr::memchr_iter;

    /// Split byte buffer into lines using SIMD-accelerated newline detection
    pub fn find_line_endings(buffer: &[u8]) -> Vec<usize> {
        memchr_iter(b'\n', buffer).collect()
    }

    /// Count lines in a buffer quickly
    pub fn count_lines(buffer: &[u8]) -> usize {
        memchr_iter(b'\n', buffer).count()
            + if buffer.is_empty() || buffer[buffer.len() - 1] == b'\n' {
                0
            } else {
                1
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simd_literal_matcher_basic() {
        let matcher = SimdLiteralMatcher::new("test");

        assert!(matcher.is_match("This is a test string"));
        assert!(!matcher.is_match("This does not match"));
    }

    #[test]
    fn test_factory_creates_appropriate_matchers() {
        // Simple pattern should use literal/SIMD matcher
        let simple_matcher = PatternMatcherFactory::create("simple");
        assert!(simple_matcher.is_match("simple pattern"));
        assert!(!simple_matcher.is_match("not matching"));

        // Complex pattern should use regex matcher
        let complex_matcher = PatternMatcherFactory::create("comp.ex|pattern");
        assert!(complex_matcher.is_match("complex"));
        assert!(complex_matcher.is_match("pattern"));
        assert!(!complex_matcher.is_match("not matching"));
    }

    #[test]
    fn test_line_processing() {
        use super::line_processing::*;

        let text = b"Line 1\nLine 2\nLine 3";
        let line_endings = find_line_endings(text);

        assert_eq!(line_endings, vec![6, 13]);
        assert_eq!(count_lines(text), 3);
    }
}
