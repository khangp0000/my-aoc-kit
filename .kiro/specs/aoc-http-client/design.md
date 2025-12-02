# Design Document

## Overview

The AOC HTTP Client library provides a simple, synchronous interface for interacting with the Advent of Code website. It handles session validation, puzzle input fetching, and answer submission with proper error handling and response parsing. The library is designed to be used as a building block for AOC solver applications, with caching and retry logic left to the consumer.

## Architecture

The library follows a simple layered architecture:

1. **Public API Layer**: Clean, ergonomic functions for session validation, input fetching, and answer submission
2. **HTTP Client Layer**: Manages the reqwest blocking client and request construction
3. **Response Parser Layer**: Extracts meaningful information from HTML responses
4. **Error Layer**: Strongly-typed errors using thiserror

The library will be stateless - each function takes the session cookie as a parameter, allowing users to manage their own client instances and caching strategies.

## Components and Interfaces

### AocClient

The main client struct that holds the reqwest client and base URL:

```rust
#[derive(Clone, Debug)]
pub struct AocClient {
    client: reqwest::blocking::Client,
    base_url: reqwest::Url,
}

impl AocClient {
    pub fn new() -> Result<Self, AocError>;
    pub fn builder() -> AocClientBuilder;
    fn create_cookie_header(session: &str) -> Result<HeaderValue, AocError>;
    pub fn verify_session(&self, session: &str) -> Result<SessionInfo, AocError>;
    pub fn get_input(&self, year: u16, day: u8, session: &str) -> Result<String, AocError>;
    pub fn submit_answer(&self, year: u16, day: u8, part: u8, answer: &str, session: &str) -> Result<SubmissionResult, AocError>;
}
```

Note: The `create_cookie_header` method is a private helper that creates secure cookie headers with the sensitive flag set and zeroizes temporary strings.

### SessionInfo

A struct representing the result of session verification:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionInfo {
    pub user_id: Option<u64>,
}
```

This provides information about the session validity through the presence of a user ID. A valid session will have `Some(user_id)`, while an invalid session will have `None`.

### AocClientBuilder

A builder for configuring the AOC client with custom settings:

```rust
pub struct AocClientBuilder {
    base_url: Option<reqwest::Url>,
    client_builder: Option<reqwest::blocking::ClientBuilder>,
}

impl AocClientBuilder {
    pub fn new() -> Self;
    pub fn base_url(self, url: impl IntoUrl) -> Result<Self, AocError>;
    pub fn client_builder(self, builder: reqwest::blocking::ClientBuilder) -> Self;
    pub fn build(self) -> Result<AocClient, AocError>;
}
```

Notes:
- `IntoUrl` is a trait from reqwest that's implemented for `&str`, `String`, and `Url`, providing flexible URL input
- The `client_builder` method allows full customization of the HTTP client (timeouts, proxies, etc.)
- The redirect policy will always be overridden to `Policy::none()` regardless of the provided builder configuration

### SubmissionResult

An enum representing the outcome of an answer submission:

```rust
pub enum SubmissionResult {
    Correct,
    Incorrect,
    AlreadyCompleted,
    Throttled { wait_time: Option<Duration> },
}
```

This provides a cleaner API than returning errors for expected outcomes like incorrect answers or throttling.

### AocError

The error type using thiserror:

```rust
#[derive(Error, Debug)]
pub enum AocError {
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),
    
    #[error("Invalid HTTP status: {status}")]
    InvalidStatus { status: reqwest::StatusCode },
    
    #[error("Failed to decode response as UTF-8")]
    Encoding,
    
    #[error("Failed to parse HTML response")]
    HtmlParse,
    
    #[error("Failed to parse duration: {0}")]
    DurationParse(String),
    
    #[error("Client initialization failed: {0}")]
    ClientInit(String),
}
```

Note: `reqwest::Error` already includes URL parsing errors, so they'll be automatically converted via the `From` implementation. URL construction errors within the client are wrapped in `ClientInit` errors with descriptive messages.

## Data Models

### Request Parameters

- **Session Cookie**: `&str` - The session token for authentication
- **Year**: `u16` - The AOC year (e.g., 2024)
- **Day**: `u8` - The day number (1-25)
- **Part**: `u8` - The part number (1 or 2)
- **Answer**: `&str` - The answer to submit as a string

### Response Models

The library parses HTML responses to extract:
- Main content text (using scraper with `<main>` selector)
- Throttle duration (using regex pattern `"You have (.+) left to wait."`)
- Submission feedback (string matching on extracted text)
- User ID from settings page (using regex pattern `\(anonymous user #(\d+)\)`)

### ResponseParser

An internal struct that caches compiled regex patterns and CSS selectors to avoid repeated compilation:

```rust
struct ResponseParser {
    user_id_regex: OnceCell<Regex>,
    throttle_regex: OnceCell<Regex>,
    main_selector: OnceCell<Selector>,
}
```

- Patterns are lazily initialized on first use via `OnceCell`
- Each `AocClient` has its own parser instance (no thread contention)
- Cloning `AocClient` clones the parser (cheap, uninitialized `OnceCell`s)
- After first use, accessing patterns is just a pointer dereference (zero overhead)


## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property Reflection

After analyzing the acceptance criteria, several properties can be consolidated:
- Properties 5.1, 5.2, 5.3 cover response parsing for different feedback types - these are distinct and should remain separate
- Properties 3.1 and 3.2 both relate to submission request construction - can be combined
- Network error handling (1.5, 2.4, 3.5) follows the same pattern but applies to different operations - keep as examples rather than properties
- Encoding errors (2.5, 3.4, 7.5) are the same edge case - consolidate to one example
- Duration parsing (5.4, 9.7) are testing the same behavior - combine

### Property 1: Session validation interprets 2xx success with user ID as valid

*For any* session cookie string, when the verify_session function receives any 2xx success status code from the settings endpoint and successfully extracts a user ID, it should return SessionInfo with Some(user_id).

**Validates: Requirements 1.1, 1.4**

### Property 2: Session validation interprets redirects and errors as invalid

*For any* session cookie string and any 3xx redirect or HTTP error status code (4xx, 5xx), when the verify_session function receives that status, it should return SessionInfo with user_id=None.

**Validates: Requirements 1.2, 1.3**

### Property 16: User ID extraction from HTML

*For any* HTML response containing the pattern "(anonymous user #[id])" where [id] is a numeric value, the parser should extract the user ID as a u64 value.

**Validates: Requirements 1.4**

### Property 3: Input URL construction

*For any* valid year (u16) and day (u8) values, the get_input function should construct a GET request to the URL `https://adventofcode.com/{year}/day/{day}/input`.

**Validates: Requirements 2.1**

### Property 4: Submission request construction

*For any* valid year, day, part, and answer values, the submit_answer function should construct a POST request to `https://adventofcode.com/{year}/day/{day}/answer` with form data containing the level (part) and answer, and include the session cookie in the request headers.

**Validates: Requirements 3.1, 3.2**

### Property 5: Incorrect answer detection

*For any* HTML response body containing the text "not the right answer", the response parser should return SubmissionResult::Incorrect.

**Validates: Requirements 5.1**

### Property 6: Already completed detection

*For any* HTML response body containing the text "already complete it", the response parser should return SubmissionResult::AlreadyCompleted.

**Validates: Requirements 5.2**

### Property 7: Throttling detection

*For any* HTML response body containing the text "gave an answer too recently", the response parser should return SubmissionResult::Throttled.

**Validates: Requirements 5.3**

### Property 8: Throttle duration extraction and parsing

*For any* HTML response containing the pattern "You have {duration} left to wait." where {duration} is a valid humantime duration string, the response parser should extract the duration string and parse it into a Duration value.

**Validates: Requirements 5.4, 9.6, 9.7**

### Property 9: HTML main element extraction

*For any* HTML document containing a `<main>` element, the response parser should extract all text content from within that element, with HTML tags removed.

**Validates: Requirements 9.1**

### Property 10: Non-success status error handling

*For any* HTTP response with a non-success status code (not 2xx or 3xx), the client should return an AocError::InvalidStatus error containing the status code.

**Validates: Requirements 7.4**

### Property 11: Base URL configuration

*For any* valid base URL string, when an AocClient is built with that base URL, all HTTP requests should use that base URL instead of the default "https://adventofcode.com".

**Validates: Requirements 10.3**

### Property 12: Default base URL

*For any* AocClient created without specifying a base URL, all HTTP requests should use "https://adventofcode.com" as the base URL.

**Validates: Requirements 10.2**

### Property 13: Custom ClientBuilder configuration

*For any* valid reqwest ClientBuilder, when an AocClient is built with that ClientBuilder, the resulting client should use the custom configuration while maintaining the redirect policy of none.

**Validates: Requirements 11.3, 11.4**

### Property 14: Redirect policy enforcement

*For any* AocClient built with or without a custom ClientBuilder, the HTTP client should always have redirect policy set to none to ensure session verification works correctly.

**Validates: Requirements 11.4**

### Property 15: Secure cookie header handling

*For any* session cookie string, when the client creates a cookie header, the header value should be marked as sensitive to prevent it from appearing in logs.

**Validates: Requirements 1.6**

## Error Handling

The library uses a single error type `AocError` that covers all failure modes:

1. **Network Errors**: Wrapped reqwest errors for connection failures, timeouts, DNS issues
2. **HTTP Errors**: Invalid status codes that don't match expected patterns
3. **Encoding Errors**: UTF-8 decoding failures
4. **Parse Errors**: HTML parsing failures or duration parsing failures
5. **Initialization Errors**: Client creation failures

All errors implement `std::error::Error` and provide descriptive messages. The library does not use panics - all failures are returned as `Result` types.

## Testing Strategy

### Unit Tests

Unit tests will cover:
- URL construction for various year/day combinations
- Form data encoding for submissions
- HTML parsing with various response formats
- Error conversion from reqwest errors
- Edge cases like invalid UTF-8, malformed HTML

### Property-Based Tests

Property-based tests are implemented using the `proptest` crate. Each correctness property has a corresponding property-based test:

1. **Session validation properties** - Generate random session strings and status codes
2. **URL construction** - Generate random valid year/day values and verify URL format
3. **Response parsing** - Generate HTML documents with various feedback patterns
4. **Duration extraction** - Generate HTML with various duration formats
5. **Error handling** - Generate various error conditions and verify proper error types
6. **Base URL configuration** - Generate random valid URLs and verify client configuration
7. **ClientBuilder configuration** - Generate random timeout values and verify custom configuration

Each property test runs 10 iterations by default (configured via `ProptestConfig::with_cases(10)`) to ensure coverage across the input space while keeping test execution fast. This can be increased for more thorough testing if needed.

### Integration Tests

Integration tests use the `mockito` crate to create mock HTTP servers that simulate AOC responses. The tests are integrated into the property-based test suite and cover:
- Valid session verification (200 status)
- Invalid session verification (3xx redirects and error status codes)
- Successful input fetching with various year/day combinations
- Successful answer submission with form data validation
- Incorrect answer responses with pattern detection
- Already completed responses
- Throttled submission responses with duration parsing
- Non-success status error handling
- Base URL configuration with mock servers
- Redirect policy enforcement verification

## Implementation Notes

### Client Initialization

The reqwest client will be created with:
```rust
reqwest::blocking::Client::builder()
    .use_rustls_tls()
    .redirect(reqwest::redirect::Policy::none())
    .build()
```

The builder pattern allows for flexible configuration:
```rust
// Default client
let client = AocClient::new()?;

// Client with custom base URL (for testing)
let client = AocClient::builder()
    .base_url("http://localhost:1234")?
    .build()?;

// Client with custom HTTP configuration
let custom_builder = reqwest::blocking::Client::builder()
    .timeout(Duration::from_secs(30))
    .use_rustls_tls();

let client = AocClient::builder()
    .client_builder(custom_builder)
    .build()?;

// Combine custom base URL and HTTP configuration
let client = AocClient::builder()
    .base_url("http://localhost:1234")?
    .client_builder(
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(10))
    )
    .build()?;
```

Note: The redirect policy will always be set to `Policy::none()` regardless of the provided `ClientBuilder` configuration, as this is required for session verification to work correctly.

### Security Considerations

Session cookies are handled securely:
- Cookie header values are marked as sensitive using `HeaderValue::set_sensitive(true)` to prevent them from appearing in logs
- Temporary cookie strings are zeroized from memory after use using the `zeroize` crate
- This prevents session tokens from lingering in memory where they could be exposed through memory dumps or debugging tools

### URL Construction

All URLs will be constructed using `reqwest::Url::join()` which properly handles:
- Trailing slashes in the base URL
- Path segment joining
- URL encoding

URL construction for each endpoint:
- Session verification: `base_url.join("/settings")?`
- Input fetching: `base_url.join(&format!("/{}/day/{}/input", year, day))?`
- Answer submission: `base_url.join(&format!("/{}/day/{}/answer", year, day))?`

Using `Url::join()` ensures proper URL construction and avoids common pitfalls like double slashes.

### Session Cookie Format

The session cookie will be sent as: `Cookie: session={value}`

The cookie header is created through a secure helper function that:
1. Formats the cookie string
2. Converts it to a `HeaderValue`
3. Marks the header as sensitive to prevent logging
4. Zeroizes the temporary string from memory

### Response Parsing Pipeline

1. Receive HTML response body as string
2. Parse HTML using scraper
3. Select `<main>` element
4. Extract text content
5. Check for feedback patterns
6. Extract and parse duration if throttled

### Caching Strategy

The library does NOT implement caching. Users should wrap the client with their own caching layer if desired. This keeps the library simple and allows users to choose their caching strategy (memory, disk, TTL, etc.).

## Dependencies

- `reqwest = { version = "0.12", features = ["blocking", "rustls-tls"] }` (includes `url` crate via re-export)
- `thiserror = "2.0"`
- `scraper = "0.24"`
- `regex = "1.12"`
- `humantime = "2.3"`
- `zeroize = "1.8"` (for secure handling of sensitive session cookie data)

Development dependencies:
- `proptest = "1.9"` (for property-based testing)
- `mockito = "1.6"` (for HTTP mocking in integration tests)

Note: `reqwest::Url` is re-exported from the `url` crate, so no additional dependency is needed.
