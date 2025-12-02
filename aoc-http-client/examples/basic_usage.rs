//! Basic usage example for the AOC HTTP client
//!
//! This example demonstrates how to:
//! - Create a client with default settings
//! - Create a client with custom base URL (for testing)
//! - Verify a session cookie
//! - Fetch puzzle input
//! - Submit an answer
//!
//! Note: This example requires a valid AOC session cookie to run.
//! You can get your session cookie from your browser's cookies after logging in to adventofcode.com

use aoc_http_client::{AocClient, SubmissionResult};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get session cookie from environment variable
    let session = std::env::var("AOC_SESSION")
        .expect("AOC_SESSION environment variable not set");

    // Example 1: Create a client with default settings
    println!("=== Example 1: Default Client ===");
    let client = AocClient::new()?;
    println!("✓ Client created with default base URL (https://adventofcode.com)");

    // Example 2: Create a client with custom base URL (useful for testing)
    println!("\n=== Example 2: Custom Base URL ===");
    let _custom_client = AocClient::builder()
        .base_url("https://adventofcode.com")?  // Could be a mock server URL for testing
        .build()?;
    println!("✓ Client created with custom base URL");

    // Example 3: Create a client with custom HTTP configuration
    println!("\n=== Example 3: Custom HTTP Configuration ===");
    let _configured_client = AocClient::builder()
        .client_builder(
            reqwest::blocking::Client::builder()
                .timeout(Duration::from_secs(30))
                .use_rustls_tls()
        )
        .build()?;
    println!("✓ Client created with custom timeout (30s)");

    // Use the default client for the rest of the example
    println!("\n=== Using Default Client ===");

    // Verify the session cookie
    println!("\nVerifying session cookie...");
    let session_info = client.verify_session(&session)?;
    if let Some(user_id) = session_info.user_id {
        println!("✓ Session is valid (User ID: {})", user_id);
    } else {
        println!("✗ Session is invalid");
        return Ok(());
    }

    // Fetch puzzle input for a specific year and day
    let year = 2024;
    let day = 1;
    println!("\nFetching input for year {} day {}...", year, day);
    match client.get_input(year, day, &session) {
        Ok(input) => {
            println!("✓ Input fetched successfully");
            println!("Input length: {} bytes", input.len());
            println!("First 100 chars: {}", &input.chars().take(100).collect::<String>());
        }
        Err(e) => {
            println!("✗ Failed to fetch input: {}", e);
        }
    }

    // Submit an answer (example - this will likely be incorrect)
    let part = 1;
    let answer = "12345";
    println!("\nSubmitting answer '{}' for part {}...", answer, part);
    match client.submit_answer(year, day, part, answer, &session) {
        Ok(result) => match result {
            SubmissionResult::Correct => {
                println!("✓ Answer is correct!");
            }
            SubmissionResult::Incorrect => {
                println!("✗ Answer is incorrect");
            }
            SubmissionResult::AlreadyCompleted => {
                println!("ℹ Problem already completed");
            }
            SubmissionResult::Throttled { wait_time } => {
                if let Some(duration) = wait_time {
                    println!("⏱ Throttled. Wait time: {:?}", duration);
                } else {
                    println!("⏱ Throttled. Wait time unknown");
                }
            }
        },
        Err(e) => {
            println!("✗ Failed to submit answer: {}", e);
        }
    }

    Ok(())
}
