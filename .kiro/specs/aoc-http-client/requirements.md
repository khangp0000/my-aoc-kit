# Requirements Document

## Introduction

This document specifies the requirements for an Advent of Code (AOC) HTTP client library that provides utilities for fetching puzzle inputs, submitting answers, and validating session cookies. The library will be implemented in Rust using blocking reqwest with rustls for TLS, and thiserror for error handling. It will handle AOC-specific response patterns including throttling and submission feedback. Caching will be the responsibility of library users.

## Glossary

- **AOC Client**: The HTTP client library that communicates with adventofcode.com
- **Session Cookie**: An authentication token provided by the caller to authenticate requests to adventofcode.com
- **Puzzle Input**: The personalized input data for a specific year and day problem
- **Answer Submission**: The process of submitting a solution answer to AOC for validation
- **Throttling**: Rate limiting imposed by AOC when submissions are made too frequently


## Requirements

### Requirement 1

**User Story:** As a developer, I want to verify my session cookie is valid and retrieve my user ID, so that I can detect authentication issues and identify which account is authenticated before attempting to fetch inputs or submit answers.

#### Acceptance Criteria

1. WHEN the AOC Client sends a GET request to https://adventofcode.com/settings with a session cookie THEN the system SHALL interpret any 2xx success status code as valid authentication
2. WHEN the AOC Client receives a 3xx redirect status code from the settings endpoint THEN the system SHALL interpret this as invalid authentication
3. WHEN the AOC Client receives an error response from the settings endpoint THEN the system SHALL interpret this as invalid authentication
4. WHEN the session verification completes with valid authentication THEN the AOC Client SHALL parse the HTML response to extract the user ID from the pattern "(anonymous user #[id])" where [id] is the numeric user identifier
5. WHEN the session verification completes THEN the AOC Client SHALL return a result containing both the validity status and the optional user ID
6. WHEN the session verification encounters a network error THEN the AOC Client SHALL return an error describing the failure
7. WHEN the AOC Client creates a cookie header THEN the system SHALL mark the header value as sensitive to prevent logging
8. WHEN the AOC Client finishes using temporary cookie strings THEN the system SHALL zeroize them from memory

### Requirement 2

**User Story:** As a developer, I want to fetch puzzle input for a specific year and day, so that I can solve AOC problems programmatically.

#### Acceptance Criteria

1. WHEN the AOC Client requests input for a valid year and day THEN the system SHALL send a GET request to https://adventofcode.com/{year}/day/{day}/input with the session cookie
2. WHEN the input request succeeds THEN the AOC Client SHALL return the puzzle input as a UTF-8 string
3. WHEN the input request fails with an HTTP error THEN the AOC Client SHALL return an error indicating the failure reason
4. WHEN the input request fails due to network issues THEN the AOC Client SHALL return an error describing the network failure
5. WHEN the response body cannot be decoded as UTF-8 THEN the AOC Client SHALL return an error indicating the encoding failure

### Requirement 3

**User Story:** As a developer, I want to submit answers for puzzle parts, so that I can validate my solutions and track progress.

#### Acceptance Criteria

1. WHEN the AOC Client submits an answer THEN the system SHALL send a POST request to https://adventofcode.com/{year}/day/{day}/answer with form data containing level (part number) and answer
2. WHEN the submission request is sent THEN the system SHALL include the session cookie in the request headers
3. WHEN the submission succeeds with a correct answer THEN the AOC Client SHALL return success
4. WHEN the response body cannot be decoded as UTF-8 THEN the AOC Client SHALL return an error indicating the encoding failure
5. WHEN the submission request fails due to network issues THEN the AOC Client SHALL return an error describing the network failure

### Requirement 5

**User Story:** As a developer, I want to receive detailed feedback on answer submissions, so that I can understand why a submission failed or was rejected.

#### Acceptance Criteria

1. WHEN the submission response contains "not the right answer" THEN the AOC Client SHALL return an IncorrectAnswer error with details about the year, day, part, and submitted answer
2. WHEN the submission response contains "already complete it" THEN the AOC Client SHALL return an AlreadySubmitted error with details about the year, day, and part
3. WHEN the submission response contains "gave an answer too recently" THEN the AOC Client SHALL parse the throttle duration from the response text
4. WHEN a throttle duration is successfully parsed THEN the AOC Client SHALL return a SubmissionThrottled error containing the parsed duration
5. WHEN a throttle duration cannot be parsed THEN the AOC Client SHALL return a SubmissionThrottled error without a duration value

### Requirement 6

**User Story:** As a developer, I want the HTTP client to use blocking reqwest with rustls, so that I have a synchronous API with secure TLS without OpenSSL dependencies.

#### Acceptance Criteria

1. WHEN the AOC Client is initialized THEN the system SHALL create a reqwest blocking client with rustls-tls feature enabled
2. WHEN the AOC Client makes HTTP requests THEN the system SHALL use the blocking reqwest client for all operations
3. WHEN TLS connections are established THEN the system SHALL use rustls for cryptographic operations
4. WHEN the client is built THEN the system SHALL not depend on OpenSSL libraries
5. WHEN the client initialization fails THEN the system SHALL return an error describing the initialization failure

### Requirement 7

**User Story:** As a developer, I want all errors to be well-typed using thiserror, so that I can handle different error cases programmatically and get clear error messages.

#### Acceptance Criteria

1. WHEN any error occurs in the AOC Client THEN the system SHALL represent the error using a thiserror-derived enum
2. WHEN a reqwest error occurs THEN the system SHALL provide a ReqwestError variant with details from the underlying error
4. WHEN an HTTP response has a non-success status THEN the system SHALL provide a ResponseStatusError variant with the status information
5. WHEN response body decoding fails THEN the system SHALL provide a ResponseStringBodyError variant with encoding details
6. WHEN an incorrect answer is submitted THEN the system SHALL provide an IncorrectAnswer variant with submission details
7. WHEN an answer is already submitted THEN the system SHALL provide an AlreadySubmitted variant with problem details
8. WHEN submission is throttled THEN the system SHALL provide a SubmissionThrottled variant with optional duration information
9. WHEN duration parsing fails THEN the system SHALL provide a DurationParseError variant with the invalid duration string

### Requirement 8

**User Story:** As a developer, I want the library to use the latest stable versions of well-maintained dependencies, so that I benefit from bug fixes, security patches, and reliable functionality.

#### Acceptance Criteria

1. WHEN the library is built THEN the system SHALL use reqwest version 0.12 or later with blocking and rustls-tls features enabled
2. WHEN the library is built THEN the system SHALL use thiserror version 2.0 or later for error handling
3. WHEN the library is built THEN the system SHALL use scraper version 0.24 or later for robust HTML parsing of submission responses
4. WHEN the library is built THEN the system SHALL use regex version 1.12 or later for extracting throttle duration patterns from response text
5. WHEN the library is built THEN the system SHALL use humantime version 2.3 or later for parsing duration strings from AOC responses
6. WHEN the library is built THEN the system SHALL use zeroize version 1.8 or later for secure handling of sensitive session cookie data

### Requirement 9

**User Story:** As a developer, I want the library to parse HTML responses from submission endpoints, so that I can extract feedback messages and throttle information reliably.

#### Acceptance Criteria

1. WHEN the AOC Client receives a submission response THEN the system SHALL parse the HTML body using scraper to extract the main element content
2. WHEN the main element is found THEN the system SHALL extract all text content for analysis
3. WHEN checking for incorrect answers THEN the system SHALL search for the text "not the right answer" in the extracted text
4. WHEN checking for already completed problems THEN the system SHALL search for the text "already complete it" in the extracted text
5. WHEN checking for throttling THEN the system SHALL search for the text "gave an answer too recently" in the extracted text
6. WHEN throttling is detected THEN the system SHALL use regex to extract the wait time from the pattern "You have (.+) left to wait."
7. WHEN the extracted wait time string is found THEN the system SHALL parse it into a Duration value using humantime

### Requirement 10

**User Story:** As a developer, I want to configure a custom base URL for the AOC client, so that I can test the client with mock servers like mockito without modifying production code.

#### Acceptance Criteria

1. WHEN creating an AOC Client THEN the system SHALL provide a builder method that accepts an optional base URL parameter
2. WHEN no base URL is provided THEN the system SHALL default to "https://adventofcode.com"
3. WHEN a custom base URL is provided THEN the system SHALL use that URL for all HTTP requests
4. WHEN constructing request URLs THEN the system SHALL combine the base URL with the appropriate path segments
5. WHEN the base URL contains a trailing slash THEN the system SHALL handle it correctly to avoid double slashes in the final URL

### Requirement 11

**User Story:** As a developer, I want to provide a custom reqwest ClientBuilder for the AOC client, so that I can configure timeouts, proxies, and other HTTP client settings while maintaining correct redirect behavior.

#### Acceptance Criteria

1. WHEN creating an AOC Client THEN the system SHALL provide a builder method that accepts an optional reqwest ClientBuilder
2. WHEN no ClientBuilder is provided THEN the system SHALL create a default ClientBuilder with rustls-tls enabled
3. WHEN a custom ClientBuilder is provided THEN the system SHALL use it to build the HTTP client
4. WHEN building the HTTP client THEN the system SHALL always override the redirect policy to none regardless of the provided ClientBuilder configuration
5. WHEN the ClientBuilder fails to build THEN the system SHALL return an error describing the failure
