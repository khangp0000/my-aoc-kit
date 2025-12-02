//! Error types for the AOC HTTP client

use thiserror::Error;

/// Errors that can occur when using the AOC HTTP client
#[derive(Error, Debug)]
pub enum AocError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Request(#[from] reqwest::Error),

    /// Invalid HTTP status code received
    #[error("Invalid HTTP status: {status}")]
    InvalidStatus {
        /// The status code that was received
        status: reqwest::StatusCode,
    },

    /// Failed to decode response as UTF-8
    #[error("Failed to decode response as UTF-8")]
    Encoding,

    /// Failed to parse HTML response
    #[error("Failed to parse HTML response")]
    HtmlParse,

    /// Failed to parse duration string
    #[error("Failed to parse duration: {0}")]
    DurationParse(String),

    /// Client initialization failed
    #[error("Client initialization failed: {0}")]
    ClientInit(String),
}
