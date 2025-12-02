# AOC HTTP Client

A Rust library for interacting with the Advent of Code website. Provides utilities for session validation, puzzle input fetching, and answer submission.

## Features

- **Session Validation**: Verify if your AOC session cookie is valid and retrieve your user ID
- **Input Fetching**: Download puzzle inputs for any year and day
- **Answer Submission**: Submit answers and get detailed feedback
- **Secure**: Uses rustls for TLS (no OpenSSL dependencies)
- **Blocking API**: Simple synchronous interface using reqwest blocking client
- **Error Handling**: Well-typed errors using thiserror

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
aoc-http-client = { path = "../aoc-http-client" }
```

Or if using from within the workspace:

```toml
[dependencies]
aoc-http-client = { workspace = true }
```

## Usage

### Basic Example

```rust
use aoc_http_client::{AocClient, SubmissionResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default settings
    let client = AocClient::new()?;
    
    // Your session cookie from adventofcode.com
    let session = "your_session_cookie_here";
    
    // Verify session and get user ID
    let session_info = client.verify_session(session)?;
    if let Some(user_id) = session_info.user_id {
        println!("Session is valid! User ID: {}", user_id);
    }
    
    // Fetch puzzle input
    let input = client.get_input(2024, 1, session)?;
    println!("Input: {}", input);
    
    // Submit an answer
    let result = client.submit_answer(2024, 1, 1, "42", session)?;
    match result {
        SubmissionResult::Correct => println!("Correct!"),
        SubmissionResult::Incorrect => println!("Incorrect"),
        SubmissionResult::AlreadyCompleted => println!("Already done"),
        SubmissionResult::Throttled { wait_time } => {
            println!("Throttled: {:?}", wait_time);
        }
    }
    
    Ok(())
}
```

### Builder Pattern

The library supports a builder pattern for advanced configuration:

```rust
use aoc_http_client::AocClient;
use std::time::Duration;

// Default client
let client = AocClient::new()?;

// Custom base URL (useful for testing with mock servers)
let client = AocClient::builder()
    .base_url("http://localhost:1234")?
    .build()?;

// Custom HTTP client configuration
let client = AocClient::builder()
    .client_builder(
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(30))
            .use_rustls_tls()
    )
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

**Note:** The redirect policy is always set to `none` regardless of custom configuration, as this is required for session verification to work correctly.

### Getting Your Session Cookie

1. Log in to [adventofcode.com](https://adventofcode.com)
2. Open your browser's developer tools (F12)
3. Go to the Application/Storage tab
4. Find Cookies → https://adventofcode.com
5. Copy the value of the `session` cookie

### Error Handling

The library uses a well-typed error enum:

```rust
use aoc_http_client::{AocClient, AocError};

match client.get_input(2024, 1, session) {
    Ok(input) => println!("Got input: {}", input),
    Err(AocError::Request(e)) => eprintln!("Network error: {}", e),
    Err(AocError::InvalidStatus { status }) => eprintln!("HTTP error: {}", status),
    Err(AocError::Encoding) => eprintln!("UTF-8 decoding failed"),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Running Examples

Set your session cookie as an environment variable:

```bash
export AOC_SESSION="your_session_cookie_here"
cargo run --example basic_usage
```

## API Documentation

### `AocClient`

The main client struct for interacting with AOC.

#### Methods

- `new() -> Result<Self, AocError>` - Create a new client with default settings
- `builder() -> AocClientBuilder` - Create a builder for custom configuration
- `verify_session(&self, session: &str) -> Result<SessionInfo, AocError>` - Check if session is valid and get user ID
- `get_input(&self, year: u16, day: u8, session: &str) -> Result<String, AocError>` - Fetch puzzle input
- `submit_answer(&self, year: u16, day: u8, part: u8, answer: &str, session: &str) -> Result<SubmissionResult, AocError>` - Submit an answer

### `AocClientBuilder`

Builder for configuring an AOC HTTP client with custom settings.

#### Methods

- `new() -> Self` - Create a new builder with default settings
- `base_url(self, url: impl IntoUrl) -> Result<Self, AocError>` - Set a custom base URL (useful for testing)
- `client_builder(self, builder: ClientBuilder) -> Self` - Set a custom HTTP client builder (for timeouts, proxies, etc.)
- `build(self) -> Result<AocClient, AocError>` - Build the client with configured settings

### `SessionInfo`

Struct representing the result of session verification:

- `user_id: Option<u64>` - User ID if session is valid, `None` if invalid

### `SubmissionResult`

Enum representing the outcome of an answer submission:

- `Correct` - Answer was correct
- `Incorrect` - Answer was incorrect
- `AlreadyCompleted` - Problem was already solved
- `Throttled { wait_time: Option<Duration> }` - Submission was rate-limited

### `AocError`

Error types that can occur:

- `Request(reqwest::Error)` - Network/HTTP error
- `InvalidStatus { status: StatusCode }` - Non-success HTTP status
- `Encoding` - UTF-8 decoding failed
- `HtmlParse` - HTML parsing failed
- `DurationParse(String)` - Duration parsing failed
- `ClientInit(String)` - Client initialization failed

## Testing

The library supports custom base URLs, making it easy to test your code with mock servers. The unit tests in this library demonstrate this pattern using `mockito`:

```rust
use aoc_http_client::AocClient;

#[test]
fn test_with_mock_server() {
    let mut server = mockito::Server::new();
    
    // Mock the input endpoint
    let mock = server.mock("GET", "/2024/day/1/input")
        .with_status(200)
        .with_body("test input data")
        .create();
    
    // Create client with mock server URL
    let client = AocClient::builder()
        .base_url(&server.url())
        .unwrap()
        .build()
        .unwrap();
    
    // Test your code
    let input = client.get_input(2024, 1, "test_session").unwrap();
    assert_eq!(input, "test input data");
    
    mock.assert();
}
```

See the unit tests in `src/client.rs` and `src/parser.rs` for more examples.

## Caching

This library does NOT implement caching. You should implement your own caching layer if needed:

```rust
use std::collections::HashMap;

struct CachedClient {
    client: AocClient,
    cache: HashMap<(u16, u8), String>,
}

impl CachedClient {
    fn get_input(&mut self, year: u16, day: u8, session: &str) -> Result<String, AocError> {
        if let Some(cached) = self.cache.get(&(year, day)) {
            return Ok(cached.clone());
        }
        
        let input = self.client.get_input(year, day, session)?;
        self.cache.insert((year, day), input.clone());
        Ok(input)
    }
}
```

## License

This project is licensed under the MIT License.

## Disclaimer

This library is not affiliated with or endorsed by Advent of Code. Please be respectful of the AOC servers and implement appropriate rate limiting and caching in your applications.
