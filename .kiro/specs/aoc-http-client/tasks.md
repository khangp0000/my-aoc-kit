# Implementation Plan

- [x] 1. Set up project structure and dependencies
  - Create new library crate `aoc-http-client`
  - Add dependencies: reqwest (blocking + rustls-tls), thiserror, scraper, regex, humantime
  - Add dev dependencies: proptest for property-based testing
  - Set up lib.rs with module structure
  - _Requirements: 6.1, 8.1, 8.2, 8.3, 8.4, 8.5_

- [x] 2. Implement error types
  - Define AocError enum with thiserror derives
  - Add variants: Request, InvalidStatus, Encoding, HtmlParse, DurationParse, ClientInit
  - Implement From trait for reqwest::Error
  - _Requirements: 7.1, 7.2, 7.4, 7.5, 7.9_

- [x] 3. Implement AocClient struct and initialization
  - Define AocClient struct with reqwest blocking client field and base_url field (reqwest::Url type)
  - Derive Clone and Debug for AocClient
  - Implement new() method with rustls-tls configuration and default base URL
  - Implement builder() method that returns AocClientBuilder
  - Handle client initialization errors
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 10.1, 10.2_

- [x] 3.1 Implement AocClientBuilder
  - Define AocClientBuilder struct with Option<reqwest::Url> and Option<reqwest::blocking::ClientBuilder> fields
  - Implement new() method
  - Implement base_url() method that accepts impl IntoUrl and returns Result<Self, AocError>
  - Parse and validate URL at builder time using reqwest's IntoUrl trait
  - Implement client_builder() method that accepts reqwest::blocking::ClientBuilder
  - Implement build() method that creates AocClient with configured base URL and client
  - In build(), always override redirect policy to none regardless of provided ClientBuilder
  - Use provided ClientBuilder or create default with rustls-tls if none provided
  - Use Url::join() for proper path construction (handles trailing slashes automatically)
  - Default to "https://adventofcode.com" if no base URL provided
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 11.1, 11.2, 11.3, 11.4, 11.5_

- [x] 3.2 Write property test for base URL and client builder configuration
  - **Property 11: Base URL configuration**
  - **Property 12: Default base URL**
  - **Property 13: Custom ClientBuilder configuration**
  - **Property 14: Redirect policy enforcement**
  - **Validates: Requirements 10.2, 10.3, 11.3, 11.4**

- [x] 4. Update session verification to use configurable base URL
  - Update verify_session method to use self.base_url instead of hardcoded URL
  - Construct URL using self.base_url.join("/settings")?
  - Interpret 2xx success as valid (using .is_success()), 3xx redirect as invalid
  - Handle network errors and URL construction errors
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 10.3, 10.4_

- [x] 4.1 Write property test for session validation
  - **Property 1: Session validation interprets 2xx success as valid**
  - **Property 2: Session validation interprets redirects and errors as invalid**
  - **Validates: Requirements 1.1, 1.2, 1.3**

- [x] 4.2 Enhance session verification to return user ID
  - Create SessionInfo struct with user_id (Option<u64>) field
  - Update verify_session to return Result<SessionInfo, AocError> instead of Result<bool, AocError>
  - When response is successful (2xx), parse HTML to extract user ID from pattern "(anonymous user #[id])"
  - Add regex pattern to parser module: `\(anonymous user #(\d+)\)` and parse captured digits to u64
  - Return SessionInfo with Some(user_id) when user ID is found and successfully parsed
  - Return SessionInfo with None for invalid sessions (redirects, errors, or missing user ID)
  - _Requirements: 1.4, 1.5_

- [x]* 4.3 Write property test for user ID extraction
  - **Property 16: User ID extraction from HTML**
  - **Validates: Requirements 1.4**

- [x] 5. Update puzzle input fetching to use configurable base URL
  - Update get_input method to use self.base_url instead of hardcoded URL
  - Construct URL using self.base_url.join(&format!("/{}/day/{}/input", year, day))?
  - Add session cookie to request headers
  - Handle HTTP errors, encoding errors, and URL construction errors
  - Return puzzle input as String
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 10.3, 10.4_

- [x]* 5.1 Write property test for input URL construction
  - **Property 3: Input URL construction**
  - **Validates: Requirements 2.1**

- [x] 6. Implement SubmissionResult enum
  - Define SubmissionResult enum with variants: Correct, Incorrect, AlreadyCompleted, Throttled
  - Add Duration field to Throttled variant
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 7. Implement HTML response parser
  - Create response parsing module
  - Implement function to extract main element text using scraper
  - Implement function to detect "not the right answer" pattern
  - Implement function to detect "already complete it" pattern
  - Implement function to detect "gave an answer too recently" pattern
  - Implement regex-based duration extraction from "You have (.+) left to wait." pattern
  - Implement duration parsing using humantime
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_

- [x] 7.1 Write property test for HTML text extraction
  - **Property 9: HTML main element extraction**
  - **Validates: Requirements 9.1**

- [x] 7.2 Write property test for response pattern detection
  - **Property 5: Incorrect answer detection**
  - **Property 6: Already completed detection**
  - **Property 7: Throttling detection**
  - **Validates: Requirements 5.1, 5.2, 5.3**

- [x]* 7.3 Write property test for duration extraction
  - **Property 8: Throttle duration extraction and parsing**
  - **Validates: Requirements 5.4, 9.6, 9.7**

- [x] 8. Update answer submission to use configurable base URL
  - Update submit_answer method to use self.base_url instead of hardcoded URL
  - Construct URL using self.base_url.join(&format!("/{}/day/{}/answer", year, day))?
  - Build form data with level (part) and answer fields
  - Add session cookie to request headers
  - Send POST request
  - Parse response HTML and determine SubmissionResult
  - Handle network, encoding, and URL construction errors
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 5.1, 5.2, 5.3, 5.4, 5.5, 10.3, 10.4_

- [x]* 8.1 Write property test for submission request construction
  - **Property 4: Submission request construction**
  - **Validates: Requirements 3.1, 3.2**

- [x]* 8.2 Write property test for error status handling
  - **Property 10: Non-success status error handling**
  - **Validates: Requirements 7.4**

- [x] 9. Write unit tests for edge cases
  - Test invalid UTF-8 response handling
  - Test malformed HTML handling
  - Test missing main element handling
  - Test invalid duration string handling
  - Test network error propagation
  - _Requirements: 2.5, 3.4, 5.5, 7.5, 7.9_

- [x] 10. Update documentation and examples
  - Update rustdoc comments to document builder pattern and base_url configuration
  - Update examples/basic_usage.rs to show both default and custom base URL usage
  - Add example showing mockito integration for testing
  - Document testing patterns with custom base URLs
  - Update README with builder pattern usage

- [x] 11. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
