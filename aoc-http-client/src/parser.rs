//! HTML response parsing utilities

use crate::{SubmissionResult, error::AocError};
use regex::Regex;
use scraper::{Html, Selector};
use std::cell::OnceCell;
use std::time::Duration;

/// Parser for AOC HTML responses with cached regex patterns and selectors
#[derive(Clone, Debug)]
pub(crate) struct ResponseParser {
    user_id_regex: OnceCell<Regex>,
    throttle_regex: OnceCell<Regex>,
    main_selector: OnceCell<Selector>,
}

impl ResponseParser {
    /// Create a new parser with uninitialized caches
    pub fn new() -> Self {
        Self {
            user_id_regex: OnceCell::new(),
            throttle_regex: OnceCell::new(),
            main_selector: OnceCell::new(),
        }
    }

    /// Get or compile the user ID regex
    fn user_id_regex(&self) -> &Regex {
        self.user_id_regex
            .get_or_init(|| Regex::new(r"\(anonymous user #(\d+)\)").unwrap())
    }

    /// Get or compile the throttle duration regex
    fn throttle_regex(&self) -> &Regex {
        self.throttle_regex
            .get_or_init(|| Regex::new(r"You have (.+?) left to wait\.").unwrap())
    }

    /// Get or compile the main element selector
    fn main_selector(&self) -> &Selector {
        self.main_selector
            .get_or_init(|| Selector::parse("main").unwrap())
    }

    /// Extract user ID from settings page HTML
    pub fn extract_user_id(&self, html: &str) -> Option<u64> {
        let regex = self.user_id_regex();
        let captures = regex.captures(html)?;
        let user_id_str = captures.get(1)?.as_str();
        user_id_str.parse::<u64>().ok()
    }

    /// Extract text content from the main element of an HTML document
    pub fn extract_main_text(&self, html: &str) -> Result<String, AocError> {
        let document = Html::parse_document(html);
        let selector = self.main_selector();

        let main_element = document
            .select(selector)
            .next()
            .ok_or(AocError::HtmlParse)?;

        Ok(main_element.text().collect::<String>())
    }

    /// Extract throttle duration from response text
    fn extract_throttle_duration(&self, text: &str) -> Option<Duration> {
        let regex = self.throttle_regex();
        let captures = regex.captures(text)?;
        let duration_str = captures.get(1)?.as_str();
        humantime::parse_duration(duration_str).ok()
    }

    /// Parse submission response and determine the result
    pub fn parse_submission_response(&self, html: &str) -> Result<SubmissionResult, AocError> {
        let text = self.extract_main_text(html)?;

        // Check for incorrect answer
        if text.contains("not the right answer") {
            return Ok(SubmissionResult::Incorrect);
        }

        // Check for already completed
        if text.contains("already complete it") {
            return Ok(SubmissionResult::AlreadyCompleted);
        }

        // Check for throttling
        if text.contains("gave an answer too recently") {
            let wait_time = self.extract_throttle_duration(&text);
            return Ok(SubmissionResult::Throttled { wait_time });
        }

        // If none of the above, assume correct
        Ok(SubmissionResult::Correct)
    }
}

impl Default for ResponseParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_malformed_html() {
        let parser = ResponseParser::new();
        let html = r#"<html><body><main>Unclosed tag"#;
        // scraper is lenient and will still parse this
        let result = parser.extract_main_text(html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_throttled_without_duration() {
        let parser = ResponseParser::new();
        let html = r#"<html><body><main>You gave an answer too recently.</main></body></html>"#;
        let result = parser.parse_submission_response(html).unwrap();
        match result {
            SubmissionResult::Throttled { wait_time } => {
                assert!(wait_time.is_none());
            }
            _ => panic!("Expected Throttled result"),
        }
    }

    #[test]
    fn test_invalid_duration_string() {
        let parser = ResponseParser::new();
        let html = r#"<html><body><main>You gave an answer too recently. You have invalid duration left to wait.</main></body></html>"#;
        let result = parser.parse_submission_response(html).unwrap();
        match result {
            SubmissionResult::Throttled { wait_time } => {
                assert!(wait_time.is_none());
            }
            _ => panic!("Expected Throttled result"),
        }
    }

    #[test]
    fn test_empty_main_element() {
        let parser = ResponseParser::new();
        let html = r#"<html><body><main></main></body></html>"#;
        let result = parser.parse_submission_response(html).unwrap();
        // Empty main should default to Correct
        assert_eq!(result, SubmissionResult::Correct);
    }

    // **Feature: aoc-http-client, Property 9: HTML main element extraction**
    // **Validates: Requirements 9.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_html_main_element_extraction(
            // Generate text content without HTML special characters to avoid parsing issues
            text_content in "[a-zA-Z0-9 .,!?\\n]{1,200}",
            // Generate optional nested HTML tags
            has_nested_tags in prop::bool::ANY,
        ) {
            // Build HTML with main element containing text
            let html = if has_nested_tags {
                // Include nested tags that should be stripped
                format!(
                    r#"<html><body><main><p>{}</p><div><span>nested</span></div></main></body></html>"#,
                    text_content
                )
            } else {
                // Simple text content
                format!(
                    r#"<html><body><main>{}</main></body></html>"#,
                    text_content
                )
            };

            // Extract main text
            let parser = ResponseParser::new();
            let result = parser.extract_main_text(&html);

            // Property: extraction should succeed for valid HTML with main element
            prop_assert!(result.is_ok(), "extract_main_text should succeed for HTML with main element");

            let extracted = result.unwrap();

            // Property: extracted text should contain the original text content
            prop_assert!(
                extracted.contains(text_content.trim()),
                "Extracted text should contain original content. Expected substring: '{}', Got: '{}'",
                text_content.trim(),
                extracted
            );

            // Property: extracted text should not contain HTML tags
            prop_assert!(
                !extracted.contains('<') && !extracted.contains('>'),
                "Extracted text should not contain HTML tags. Got: '{}'",
                extracted
            );

            // Property: if nested tags were present, their text should also be extracted
            if has_nested_tags {
                prop_assert!(
                    extracted.contains("nested"),
                    "Extracted text should include nested element content. Got: '{}'",
                    extracted
                );
            }
        }
    }

    // Additional property test: HTML without main element should fail
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_html_without_main_element_fails(
            text_content in "[a-zA-Z0-9 .,!?\\n]{1,200}",
        ) {
            // Build HTML without main element
            let html = format!(
                r#"<html><body><div>{}</div></body></html>"#,
                text_content
            );

            // Extract main text
            let parser = ResponseParser::new();
            let result = parser.extract_main_text(&html);

            // Property: extraction should fail when main element is missing
            prop_assert!(
                result.is_err(),
                "extract_main_text should fail for HTML without main element"
            );

            // Property: error should be HtmlParse
            prop_assert!(
                matches!(result.unwrap_err(), AocError::HtmlParse),
                "Error should be AocError::HtmlParse"
            );
        }
    }

    // **Feature: aoc-http-client, Property 5: Incorrect answer detection**
    // **Validates: Requirements 5.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_incorrect_answer_detection(
            // Generate text before and after the pattern
            prefix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            suffix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            // Generate variations of the pattern
            pattern_variant in prop::sample::select(vec![
                "not the right answer",
                "That's not the right answer",
                "not the right answer.",
                "not the right answer!",
            ]),
        ) {
            // Build HTML with the incorrect answer pattern
            let text_content = format!("{} {} {}", prefix, pattern_variant, suffix);
            let html = format!(
                r#"<html><body><main>{}</main></body></html>"#,
                text_content
            );

            // Parse submission response
            let parser = ResponseParser::new();
            let result = parser.parse_submission_response(&html);

            // Property: any HTML containing "not the right answer" should be detected as Incorrect
            prop_assert!(
                result.is_ok(),
                "parse_submission_response should succeed for valid HTML"
            );

            prop_assert_eq!(
                result.unwrap(),
                SubmissionResult::Incorrect,
                "HTML containing 'not the right answer' should return SubmissionResult::Incorrect"
            );
        }
    }

    // **Feature: aoc-http-client, Property 6: Already completed detection**
    // **Validates: Requirements 5.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_already_completed_detection(
            // Generate text before and after the pattern
            prefix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            suffix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            // Generate variations of the pattern
            pattern_variant in prop::sample::select(vec![
                "already complete it",
                "You already complete it",
                "already complete it.",
                "already complete it!",
            ]),
        ) {
            // Build HTML with the already completed pattern
            let text_content = format!("{} {} {}", prefix, pattern_variant, suffix);
            let html = format!(
                r#"<html><body><main>{}</main></body></html>"#,
                text_content
            );

            // Parse submission response
            let parser = ResponseParser::new();
            let result = parser.parse_submission_response(&html);

            // Property: any HTML containing "already complete it" should be detected as AlreadyCompleted
            prop_assert!(
                result.is_ok(),
                "parse_submission_response should succeed for valid HTML"
            );

            prop_assert_eq!(
                result.unwrap(),
                SubmissionResult::AlreadyCompleted,
                "HTML containing 'already complete it' should return SubmissionResult::AlreadyCompleted"
            );
        }
    }

    // **Feature: aoc-http-client, Property 7: Throttling detection**
    // **Validates: Requirements 5.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_throttling_detection(
            // Generate text before and after the pattern
            prefix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            suffix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            // Generate variations of the pattern
            pattern_variant in prop::sample::select(vec![
                "gave an answer too recently",
                "You gave an answer too recently",
                "gave an answer too recently.",
                "gave an answer too recently!",
            ]),
        ) {
            // Build HTML with the throttling pattern
            let text_content = format!("{} {} {}", prefix, pattern_variant, suffix);
            let html = format!(
                r#"<html><body><main>{}</main></body></html>"#,
                text_content
            );

            // Parse submission response
            let parser = ResponseParser::new();
            let result = parser.parse_submission_response(&html);

            // Property: any HTML containing "gave an answer too recently" should be detected as Throttled
            prop_assert!(
                result.is_ok(),
                "parse_submission_response should succeed for valid HTML"
            );

            match result.unwrap() {
                SubmissionResult::Throttled { .. } => {
                    // Success - throttling was detected
                }
                other => {
                    prop_assert!(
                        false,
                        "HTML containing 'gave an answer too recently' should return SubmissionResult::Throttled, got {:?}",
                        other
                    );
                }
            }
        }
    }

    // **Feature: aoc-http-client, Property 8: Throttle duration extraction and parsing**
    // **Validates: Requirements 5.4, 9.6, 9.7**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_throttle_duration_extraction(
            // Generate random duration components
            minutes in 0u64..60u64,
            seconds in 0u64..60u64,
            // Generate text before and after
            prefix in "[a-zA-Z0-9 .,!?\\n]{0,50}",
            suffix in "[a-zA-Z0-9 .,!?\\n]{0,50}",
        ) {
            // Skip cases where both are zero (not a valid duration)
            prop_assume!(minutes > 0 || seconds > 0);

            // Build duration string in humantime format
            let duration_str = if minutes > 0 && seconds > 0 {
                format!("{}m {}s", minutes, seconds)
            } else if minutes > 0 {
                format!("{}m", minutes)
            } else {
                format!("{}s", seconds)
            };

            // Build HTML with throttling pattern and duration
            let text_content = format!(
                "{} You gave an answer too recently. You have {} left to wait. {}",
                prefix, duration_str, suffix
            );
            let html = format!(
                r#"<html><body><main>{}</main></body></html>"#,
                text_content
            );

            // Parse submission response
            let parser = ResponseParser::new();
            let result = parser.parse_submission_response(&html);

            // Property: parsing should succeed
            prop_assert!(
                result.is_ok(),
                "parse_submission_response should succeed for valid HTML with duration"
            );

            // Property: result should be Throttled with a duration
            match result.unwrap() {
                SubmissionResult::Throttled { wait_time } => {
                    prop_assert!(
                        wait_time.is_some(),
                        "Throttled result should contain a parsed duration for valid duration string '{}'",
                        duration_str
                    );

                    // Property: parsed duration should match expected value
                    let expected_secs = minutes * 60 + seconds;
                    let actual_secs = wait_time.unwrap().as_secs();
                    prop_assert_eq!(
                        actual_secs,
                        expected_secs,
                        "Parsed duration should match expected. Duration string: '{}'",
                        duration_str
                    );
                }
                other => {
                    prop_assert!(
                        false,
                        "Expected Throttled result, got {:?}",
                        other
                    );
                }
            }
        }
    }

    // **Feature: aoc-http-client, Property 16: User ID extraction from HTML**
    // **Validates: Requirements 1.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_user_id_extraction(
            // Generate random user IDs
            user_id in 100000u64..9999999u64,
            // Generate text before and after the pattern
            prefix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
            suffix in "[a-zA-Z0-9 .,!?\\n]{0,100}",
        ) {
            // Build HTML with user ID pattern
            let html = format!(
                r#"<html><body>{} (anonymous user #{}) {}</body></html>"#,
                prefix, user_id, suffix
            );

            // Extract user ID
            let parser = ResponseParser::new();
            let result = parser.extract_user_id(&html);

            // Property: extraction should succeed for HTML containing the pattern
            prop_assert!(
                result.is_some(),
                "extract_user_id should return Some for HTML containing '(anonymous user #{})'",
                user_id
            );

            // Property: extracted user ID should match the original
            prop_assert_eq!(
                result.unwrap(),
                user_id,
                "Extracted user ID should match the original"
            );
        }
    }

    // Additional test: HTML without user ID pattern should return None
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_user_id_extraction_missing_pattern(
            text_content in "[a-zA-Z0-9 .,!?\\n]{1,200}",
        ) {
            // Build HTML without user ID pattern
            let html = format!(
                r#"<html><body>{}</body></html>"#,
                text_content
            );

            // Extract user ID
            let parser = ResponseParser::new();
            let result = parser.extract_user_id(&html);

            // Property: extraction should return None when pattern is missing
            prop_assert!(
                result.is_none(),
                "extract_user_id should return None for HTML without user ID pattern"
            );
        }
    }
}
