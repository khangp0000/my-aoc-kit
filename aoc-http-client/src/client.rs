//! AOC HTTP client implementation

use crate::error::AocError;
use crate::parser::ResponseParser;
use reqwest::header::HeaderValue;
use std::time::Duration;
use zeroize::Zeroize;

/// Result of session verification
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    /// User ID if session is valid, None otherwise
    pub user_id: Option<u64>,
}

/// Result of an answer submission
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmissionResult {
    /// Answer was correct
    Correct,
    /// Answer was incorrect
    Incorrect,
    /// Problem was already completed
    AlreadyCompleted,
    /// Submission was throttled
    Throttled {
        /// Optional wait time before next submission
        wait_time: Option<Duration>,
    },
}

/// The main AOC HTTP client
///
/// This client provides methods for interacting with the Advent of Code website,
/// including session validation, input fetching, and answer submission.
///
/// # Example
///
/// ```no_run
/// use aoc_http_client::AocClient;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = AocClient::new()?;
/// let session = "your_session_cookie";
///
/// // Verify session and get user ID
/// let session_info = client.verify_session(session)?;
/// if let Some(user_id) = session_info.user_id {
///     println!("Session valid! User ID: {}", user_id);
/// }
///
/// // Fetch input
/// let input = client.get_input(2024, 1, session)?;
/// println!("Input: {}", input);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct AocClient {
    client: reqwest::blocking::Client,
    base_url: reqwest::Url,
    parser: ResponseParser,
}

impl AocClient {
    /// Create a new AOC client with rustls-tls configuration and no redirect policy
    ///
    /// The client is configured to not follow redirects by default, which is necessary
    /// for session verification to work correctly.
    ///
    /// # Errors
    ///
    /// Returns `AocError::ClientInit` if the HTTP client cannot be initialized.
    ///
    /// # Example
    ///
    /// ```
    /// use aoc_http_client::AocClient;
    ///
    /// let client = AocClient::new().expect("Failed to create client");
    /// ```
    pub fn new() -> Result<Self, AocError> {
        Self::builder().build()
    }

    /// Create a builder for configuring the AOC client
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::builder()
    ///     .base_url("http://localhost:1234")?
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn builder() -> AocClientBuilder {
        AocClientBuilder::new()
    }

    /// Create a secure cookie header value from a session string
    ///
    /// This function creates a HeaderValue with the sensitive flag set to true
    /// and zeroizes the temporary string after use.
    fn create_cookie_header(session: &str) -> Result<HeaderValue, AocError> {
        let mut cookie_string = format!("session={}", session);
        let header_value = HeaderValue::from_bytes(cookie_string.as_bytes())
            .map_err(|_| AocError::ClientInit("Invalid session cookie format".to_string()))?;

        // Mark as sensitive and zeroize the temporary string
        let mut sensitive_header = header_value;
        sensitive_header.set_sensitive(true);
        cookie_string.zeroize();

        Ok(sensitive_header)
    }

    /// Verify if a session cookie is valid and retrieve user ID
    ///
    /// Sends a request to the AOC settings endpoint and checks the response status.
    /// A 200 OK status indicates a valid session, and the user ID is extracted from
    /// the HTML response. A redirect (3xx) indicates an invalid session.
    ///
    /// # Arguments
    ///
    /// * `session` - The session cookie value (without "session=" prefix)
    ///
    /// # Returns
    ///
    /// * `Ok(SessionInfo { user_id: Some(id) })` - Session is valid with user ID
    /// * `Ok(SessionInfo { user_id: None })` - Session is invalid
    /// * `Err` - Network error or URL construction error occurred
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::new()?;
    /// let session = "your_session_cookie";
    ///
    /// let info = client.verify_session(session)?;
    /// if let Some(user_id) = info.user_id {
    ///     println!("Session is valid! User ID: {}", user_id);
    /// } else {
    ///     println!("Session is invalid");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_session(&self, session: &str) -> Result<SessionInfo, AocError> {
        let cookie_header = Self::create_cookie_header(session)?;

        // Construct URL using path segments
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| AocError::ClientInit("Cannot modify base URL path".to_string()))?
            .clear()
            .push("settings");

        let response = self
            .client
            .get(url)
            .header("Cookie", cookie_header)
            .send()?;

        // 2xx success means valid session (settings page loads)
        // 3xx redirect means invalid session (redirecting to homepage)
        if !response.status().is_success() {
            return Ok(SessionInfo { user_id: None });
        }

        // Extract user ID from HTML response
        let html = response.text().map_err(|_| AocError::Encoding)?;
        let user_id = self.parser.extract_user_id(&html);

        Ok(SessionInfo { user_id })
    }

    /// Fetch puzzle input for a specific year and day
    ///
    /// Downloads the personalized puzzle input for the given year and day.
    ///
    /// # Arguments
    ///
    /// * `year` - The AOC year (e.g., 2024)
    /// * `day` - The day number (1-25)
    /// * `session` - The session cookie value
    ///
    /// # Returns
    ///
    /// The puzzle input as a UTF-8 string.
    ///
    /// # Errors
    ///
    /// * `AocError::Request` - Network error
    /// * `AocError::InvalidStatus` - HTTP error (e.g., 404 if puzzle not available)
    /// * `AocError::Encoding` - Response is not valid UTF-8
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::new()?;
    /// let session = "your_session_cookie";
    ///
    /// let input = client.get_input(2024, 1, session)?;
    /// println!("Input length: {} bytes", input.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_input(&self, year: u16, day: u8, session: &str) -> Result<String, AocError> {
        let cookie_header = Self::create_cookie_header(session)?;

        // Construct URL using path segments
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| AocError::ClientInit("Cannot modify base URL path".to_string()))?
            .clear()
            .extend(&[&year.to_string(), "day", &day.to_string(), "input"]);

        let response = self
            .client
            .get(url)
            .header("Cookie", cookie_header)
            .send()?;

        if !response.status().is_success() {
            return Err(AocError::InvalidStatus {
                status: response.status(),
            });
        }

        response.text().map_err(|_| AocError::Encoding)
    }

    /// Submit an answer for a puzzle part
    ///
    /// Submits an answer to AOC and parses the response to determine the result.
    ///
    /// # Arguments
    ///
    /// * `year` - The AOC year (e.g., 2024)
    /// * `day` - The day number (1-25)
    /// * `part` - The part number (1 or 2)
    /// * `answer` - The answer to submit (as a string)
    /// * `session` - The session cookie value
    ///
    /// # Returns
    ///
    /// A `SubmissionResult` indicating the outcome:
    /// * `Correct` - Answer was correct
    /// * `Incorrect` - Answer was incorrect
    /// * `AlreadyCompleted` - Problem was already solved
    /// * `Throttled` - Submission was rate-limited (includes optional wait time)
    ///
    /// # Errors
    ///
    /// * `AocError::Request` - Network error
    /// * `AocError::InvalidStatus` - HTTP error
    /// * `AocError::Encoding` - Response is not valid UTF-8
    /// * `AocError::HtmlParse` - Failed to parse HTML response
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::{AocClient, SubmissionResult};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::new()?;
    /// let session = "your_session_cookie";
    ///
    /// let result = client.submit_answer(2024, 1, 1, "42", session)?;
    /// match result {
    ///     SubmissionResult::Correct => println!("Correct!"),
    ///     SubmissionResult::Incorrect => println!("Try again"),
    ///     SubmissionResult::AlreadyCompleted => println!("Already done"),
    ///     SubmissionResult::Throttled { wait_time } => {
    ///         println!("Wait: {:?}", wait_time);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn submit_answer(
        &self,
        year: u16,
        day: u8,
        part: u8,
        answer: &str,
        session: &str,
    ) -> Result<SubmissionResult, AocError> {
        let cookie_header = Self::create_cookie_header(session)?;

        // Construct URL using path segments
        let mut url = self.base_url.clone();
        url.path_segments_mut()
            .map_err(|_| AocError::ClientInit("Cannot modify base URL path".to_string()))?
            .clear()
            .extend(&[&year.to_string(), "day", &day.to_string(), "answer"]);

        let form = [("level", part.to_string()), ("answer", answer.to_string())];

        let response = self
            .client
            .post(url)
            .header("Cookie", cookie_header)
            .form(&form)
            .send()?;

        if !response.status().is_success() {
            return Err(AocError::InvalidStatus {
                status: response.status(),
            });
        }

        let html = response.text().map_err(|_| AocError::Encoding)?;
        self.parser.parse_submission_response(&html)
    }
}

/// Builder for configuring an AOC HTTP client
///
/// This builder allows customization of the base URL and HTTP client configuration
/// while ensuring the redirect policy is always set correctly for session verification.
///
/// # Example
///
/// ```no_run
/// use aoc_http_client::AocClient;
/// use std::time::Duration;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Default client
/// let client = AocClient::builder().build()?;
///
/// // Custom base URL for testing
/// let client = AocClient::builder()
///     .base_url("http://localhost:1234")?
///     .build()?;
///
/// // Custom timeout
/// let client = AocClient::builder()
///     .client_builder(
///         reqwest::blocking::Client::builder()
///             .timeout(Duration::from_secs(30))
///     )
///     .build()?;
///
/// // Combine custom base URL and timeout
/// let client = AocClient::builder()
///     .base_url("http://localhost:1234")?
///     .client_builder(
///         reqwest::blocking::Client::builder()
///             .timeout(Duration::from_secs(10))
///     )
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct AocClientBuilder {
    base_url: Option<reqwest::Url>,
    client_builder: Option<reqwest::blocking::ClientBuilder>,
}

impl AocClientBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            base_url: None,
            client_builder: None,
        }
    }

    /// Set a custom base URL for the client
    ///
    /// This is useful for testing with mock servers. The URL is parsed and validated
    /// at builder time, catching errors early.
    ///
    /// # Arguments
    ///
    /// * `url` - The base URL (can be `&str`, `String`, or `reqwest::Url`)
    ///
    /// # Errors
    ///
    /// Returns an error if the URL cannot be parsed.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::builder()
    ///     .base_url("http://localhost:1234")?
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn base_url(mut self, url: impl reqwest::IntoUrl) -> Result<Self, AocError> {
        self.base_url = Some(url.into_url()?);
        Ok(self)
    }

    /// Set a custom HTTP client builder
    ///
    /// This allows full customization of the HTTP client (timeouts, proxies, etc.).
    /// The redirect policy will always be overridden to `Policy::none()` regardless
    /// of the provided builder configuration.
    ///
    /// # Arguments
    ///
    /// * `builder` - A reqwest ClientBuilder with custom configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    /// use std::time::Duration;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::builder()
    ///     .client_builder(
    ///         reqwest::blocking::Client::builder()
    ///             .timeout(Duration::from_secs(30))
    ///             .use_rustls_tls()
    ///     )
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn client_builder(mut self, builder: reqwest::blocking::ClientBuilder) -> Self {
        self.client_builder = Some(builder);
        self
    }

    /// Build the AOC client with the configured settings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The HTTP client cannot be initialized
    /// - The default base URL cannot be parsed (should never happen)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use aoc_http_client::AocClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = AocClient::builder().build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn build(self) -> Result<AocClient, AocError> {
        // Use provided base URL or default to adventofcode.com
        let base_url = self.base_url.unwrap_or_else(|| {
            reqwest::Url::parse("https://adventofcode.com")
                .expect("Default base URL should always be valid")
        });

        // Use provided client builder or create default with rustls-tls
        let builder = self
            .client_builder
            .unwrap_or_else(|| reqwest::blocking::Client::builder().use_rustls_tls());

        // Always override redirect policy to none for session verification
        let client = builder
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AocError::ClientInit(e.to_string()))?;

        Ok(AocClient {
            client,
            base_url,
            parser: ResponseParser::new(),
        })
    }
}

impl Default for AocClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // **Feature: aoc-http-client, Property 11: Base URL configuration**
    // **Validates: Requirements 10.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_base_url_configuration(
            scheme in prop::sample::select(vec!["http", "https"]),
            host in "[a-z]{3,10}",
            port in 1000u16..10000u16,
        ) {
            // Construct a valid base URL
            let base_url = format!("{}://{}:{}", scheme, host, port);

            // Build client with custom base URL
            let client = AocClient::builder()
                .base_url(&base_url)
                .unwrap()
                .build()
                .unwrap();

            // Verify the base URL is set correctly
            prop_assert_eq!(client.base_url.scheme(), scheme);
            prop_assert_eq!(client.base_url.host_str(), Some(host.as_str()));
            prop_assert_eq!(client.base_url.port(), Some(port));
        }
    }

    // **Feature: aoc-http-client, Property 12: Default base URL**
    // **Validates: Requirements 10.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_default_base_url(_dummy in 0u8..10u8) {
            // Create client without specifying base URL
            let client = AocClient::builder().build().unwrap();

            // Verify default base URL is used
            prop_assert_eq!(client.base_url.as_str(), "https://adventofcode.com/");
        }
    }

    // **Feature: aoc-http-client, Property 13: Custom ClientBuilder configuration**
    // **Validates: Requirements 11.3, 11.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_custom_client_builder_configuration(
            timeout_secs in 1u64..120u64,
        ) {
            // Create a custom ClientBuilder with timeout
            let custom_builder = reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(timeout_secs))
                .use_rustls_tls();

            // Build client with custom ClientBuilder
            let result = AocClient::builder()
                .client_builder(custom_builder)
                .build();

            // Verify client builds successfully
            prop_assert!(result.is_ok());
        }
    }

    // **Feature: aoc-http-client, Property 14: Redirect policy enforcement**
    // **Validates: Requirements 11.4**
    #[test]
    fn test_redirect_policy_enforcement() {
        let mut server = mockito::Server::new();

        // Mock the base path (where redirect would go if followed)
        let base_mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_body("<html><body>Home page</body></html>")
            .expect(0) // Should NOT be called if redirects are disabled
            .create();

        // Mock a redirect response at /settings
        let settings_mock = server
            .mock("GET", "/settings")
            .with_status(303)
            .with_header("location", "/")
            .expect(1) // Should be called exactly once
            .create();

        // Build client with default settings (tests that redirect policy is enforced by default)
        let client = AocClient::builder()
            .base_url(server.url())
            .unwrap()
            .build()
            .unwrap();

        // Verify session - should get the 303 directly without following redirect
        let result = client.verify_session("test_session");
        assert!(result.is_ok());
        // 303 means invalid session (redirect to homepage)
        let info = result.unwrap();
        assert!(info.user_id.is_none());

        // Verify expectations
        base_mock.assert();
        settings_mock.assert();
    }

    #[test]
    fn test_invalid_base_url() {
        let result = AocClient::builder().base_url("not a valid url");

        assert!(result.is_err());
    }

    // **Feature: aoc-http-client, Property 1: Session validation interprets 200 as valid**
    // **Validates: Requirements 1.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_session_validation_200_is_valid(
            session in "[a-f0-9]{32,128}",
            user_id in 100000u64..999999u64,
        ) {
            let mut server = mockito::Server::new();

            // Mock a 200 OK response with user ID in HTML
            let body = format!(r#"<html><body>Settings page (anonymous user #{})</body></html>"#, user_id);
            let mock = server.mock("GET", "/settings")
                .with_status(200)
                .with_body(&body)
                .expect(1)
                .create();

            // Build client with mock server URL
            let client = AocClient::builder()
                .base_url(server.url())
                .unwrap()
                .build()
                .unwrap();

            // Verify session - 200 should mean valid authentication
            let result = client.verify_session(&session);
            prop_assert!(result.is_ok(), "verify_session should not return an error");

            let info = result.unwrap();
            prop_assert!(info.user_id.is_some(), "200 status should indicate valid session with user ID");
            prop_assert_eq!(info.user_id.unwrap(), user_id, "User ID should match");

            // Verify mock was called
            mock.assert();
        }
    }

    // **Feature: aoc-http-client, Property 2: Session validation interprets redirects and errors as invalid**
    // **Validates: Requirements 1.2, 1.3**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_session_validation_redirects_and_errors_are_invalid(
            session in "[a-f0-9]{32,128}",
            status_code in prop::sample::select(vec![301, 302, 303, 307, 308, 400, 401, 403, 404, 500, 502, 503]),
        ) {
            let mut server = mockito::Server::new();

            // Mock a redirect or error response (should indicate invalid session per spec)
            let mock_builder = server.mock("GET", "/settings")
                .with_status(status_code)
                .expect(1);

            // Add location header for redirects
            let mock = if (300..400).contains(&status_code) {
                mock_builder.with_header("location", "/").create()
            } else {
                mock_builder.create()
            };

            // Build client with mock server URL
            let client = AocClient::builder()
                .base_url(server.url())
                .unwrap()
                .build()
                .unwrap();

            // Verify session - redirects and error status codes should mean invalid authentication
            let result = client.verify_session(&session);
            prop_assert!(result.is_ok(), "verify_session should not return an error for HTTP redirects/errors");

            let info = result.unwrap();
            prop_assert!(info.user_id.is_none(), "Redirect and error status codes should indicate invalid session");

            // Verify mock was called
            mock.assert();
        }
    }

    // **Feature: aoc-http-client, Property 3: Input URL construction**
    // **Validates: Requirements 2.1**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_input_url_construction(
            year in 2015u16..2030u16,
            day in 1u8..=25u8,
            session in "[a-f0-9]{32,128}",
        ) {
            let mut server = mockito::Server::new();

            // Construct the expected path
            let expected_path = format!("/{}/day/{}/input", year, day);

            // Mock the input endpoint
            let mock = server.mock("GET", expected_path.as_str())
                .with_status(200)
                .with_body("test input data")
                .expect(1)
                .create();

            // Build client with mock server URL
            let client = AocClient::builder()
                .base_url(server.url())
                .unwrap()
                .build()
                .unwrap();

            // Fetch input - this should construct the correct URL
            let result = client.get_input(year, day, &session);

            // Property: URL construction should succeed for valid year/day values
            prop_assert!(
                result.is_ok(),
                "get_input should succeed for valid year {} and day {}",
                year,
                day
            );

            // Property: the correct endpoint should be called
            mock.assert();

            // Property: the response body should be returned
            prop_assert_eq!(
                result.unwrap(),
                "test input data",
                "get_input should return the response body"
            );
        }
    }

    // **Feature: aoc-http-client, Property 4: Submission request construction**
    // **Validates: Requirements 3.1, 3.2**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_submission_request_construction(
            year in 2015u16..2030u16,
            day in 1u8..=25u8,
            part in 1u8..=2u8,
            answer in "[0-9]{1,10}",
            session in "[a-f0-9]{32,128}",
        ) {
            let mut server = mockito::Server::new();

            // Construct the expected path
            let expected_path = format!("/{}/day/{}/answer", year, day);

            // Mock the submission endpoint with form data matcher
            let mock = server.mock("POST", expected_path.as_str())
                .match_body(
                    mockito::Matcher::AllOf(vec![
                        mockito::Matcher::UrlEncoded("level".into(), part.to_string()),
                        mockito::Matcher::UrlEncoded("answer".into(), answer.clone()),
                    ])
                )
                .with_status(200)
                .with_body(r#"<html><body><main>That's the right answer!</main></body></html>"#)
                .expect(1)
                .create();

            // Build client with mock server URL
            let client = AocClient::builder()
                .base_url(server.url())
                .unwrap()
                .build()
                .unwrap();

            // Submit answer - this should construct the correct URL and form data
            let result = client.submit_answer(year, day, part, &answer, &session);

            // Property: submission should succeed for valid parameters
            prop_assert!(
                result.is_ok(),
                "submit_answer should succeed for valid year {}, day {}, part {}, answer {}",
                year,
                day,
                part,
                answer
            );

            // Property: the correct endpoint should be called with correct form data
            mock.assert();

            // Property: the response should be parsed correctly
            prop_assert_eq!(
                result.unwrap(),
                SubmissionResult::Correct,
                "submit_answer should return parsed result"
            );
        }
    }

    // **Feature: aoc-http-client, Property 10: Non-success status error handling**
    // **Validates: Requirements 7.4**
    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        #[test]
        fn prop_non_success_status_error_handling(
            year in 2015u16..2030u16,
            day in 1u8..=25u8,
            session in "[a-f0-9]{32,128}",
            // Test various non-success status codes (excluding 2xx and 3xx)
            status_code in prop::sample::select(vec![400, 401, 403, 404, 429, 500, 502, 503, 504]),
        ) {
            let mut server = mockito::Server::new();

            // Construct the expected path
            let expected_path = format!("/{}/day/{}/input", year, day);

            // Mock the input endpoint with non-success status
            let mock = server.mock("GET", expected_path.as_str())
                .with_status(status_code)
                .with_body("Error response")
                .expect(1)
                .create();

            // Build client with mock server URL
            let client = AocClient::builder()
                .base_url(server.url())
                .unwrap()
                .build()
                .unwrap();

            // Fetch input - this should fail with InvalidStatus error
            let result = client.get_input(year, day, &session);

            // Property: non-success status should result in an error
            prop_assert!(
                result.is_err(),
                "get_input should return an error for non-success status code {}",
                status_code
            );

            // Property: error should be InvalidStatus with the correct status code
            match result.unwrap_err() {
                AocError::InvalidStatus { status } => {
                    prop_assert_eq!(
                        status.as_u16(),
                        status_code as u16,
                        "Error should contain the correct status code"
                    );
                }
                other => {
                    prop_assert!(
                        false,
                        "Expected AocError::InvalidStatus, got {:?}",
                        other
                    );
                }
            }

            // Property: the endpoint should have been called
            mock.assert();
        }
    }
}
