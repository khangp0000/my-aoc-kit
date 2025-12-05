//! AOC HTTP Client Library
//!
//! This library provides utilities for interacting with the Advent of Code website,
//! including session validation, puzzle input fetching, and answer submission.
//!
//! # Features
//!
//! - Session validation to check if your AOC cookie is valid
//! - Puzzle input fetching for any year and day
//! - Answer submission with detailed feedback
//! - Secure TLS using rustls (no OpenSSL dependencies)
//! - Blocking synchronous API
//! - Well-typed errors using thiserror
//!
//! # Example
//!
//! ```no_run
//! use aoc_http_client::{AocClient, SubmissionResult};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client
//! let client = AocClient::new()?;
//!
//! // Your session cookie from adventofcode.com
//! let session = "your_session_cookie_here";
//!
//! // Verify session and get user ID
//! let session_info = client.verify_session(session)?;
//! if let Some(user_id) = session_info.user_id {
//!     println!("Session is valid! User ID: {}", user_id);
//! }
//!
//! // Fetch puzzle input
//! let input = client.get_input(2024, 1, session)?;
//!
//! // Submit an answer
//! let result = client.submit_answer(2024, 1, 1, "42", session)?;
//! match result {
//!     SubmissionResult::Correct => println!("Correct!"),
//!     SubmissionResult::Incorrect => println!("Incorrect"),
//!     SubmissionResult::AlreadyCompleted => println!("Already done"),
//!     SubmissionResult::Throttled { wait_time } => {
//!         println!("Throttled: {:?}", wait_time);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod error;
mod parser;

pub use client::{AocClient, AocClientBuilder, SessionInfo, SubmissionResult};
pub use error::AocError;
