//! Error types for the CLI

use thiserror::Error;
use thiserror_ext::Arc as ArcDerive;

/// Main CLI error type
#[derive(Error, Debug)]
pub enum CliError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Cache error
    #[error("Cache error: {0}")]
    Cache(#[from] CacheError),

    /// HTTP client error
    #[error("HTTP client error: {0}")]
    Http(#[from] aoc_http_client::AocError),

    /// Solver error
    #[error("Solver error: {0}")]
    Solver(#[from] aoc_solver::SolverError),

    /// Registration error
    #[error("Registration error: {0}")]
    Registration(#[from] aoc_solver::RegistrationError),

    /// User ID mismatch
    #[error("User ID mismatch: expected {expected}, got {actual}")]
    UserIdMismatch { expected: u64, actual: u64 },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Executor error (wraps Arc for cheap cloning)
    #[error("{0}")]
    Executor(#[from] ArcExecutorError),
}

/// Executor-specific errors
#[derive(Error, Debug, ArcDerive)]
#[thiserror_ext(newtype(name = ArcExecutorError))]
pub enum ExecutorError {
    /// Input fetch failed
    #[error("Input fetch failed for {year}/{day}: {source}")]
    InputFetch {
        year: u16,
        day: u8,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Solver execution failed
    #[error("Solver execution failed: {0}")]
    Solver(#[from] aoc_solver::SolverError),

    /// Channel send error
    #[error("Channel send error")]
    ChannelSend,

    /// Thread pool creation failed
    #[error("Thread pool creation failed: {0}")]
    ThreadPool(String),

    /// Cache write warning (non-fatal)
    #[error("Cache write failed for {year}/{day}: {message}")]
    CacheWrite { year: u16, day: u8, message: String },

    /// Multiple errors collected during parallel execution
    #[error("Multiple errors occurred ({} total)", .0.len())]
    Multiple(Vec<ArcExecutorError>),
}

impl ArcExecutorError {
    /// Combine two Arc-wrapped errors into one
    /// 1. If first is singular and second is Multiple: prepend first to second's vec
    /// 2. If second is singular and first is Multiple: append second to first's vec
    /// 3. If both are Multiple: concat them
    /// 4. If both are singular: create new Multiple with both
    pub fn combine(first: ArcExecutorError, second: ArcExecutorError) -> ArcExecutorError {
        let errors = match (first.inner(), second.inner()) {
            // Case 3: both are Multiple - concat
            (ExecutorError::Multiple(v1), ExecutorError::Multiple(v2)) => {
                let mut combined = v1.clone();
                combined.extend(v2.iter().cloned());
                combined
            }
            // Case 1: first is singular, second is Multiple - prepend first to second's vec
            (_, ExecutorError::Multiple(v)) => {
                let mut combined = vec![first];
                combined.extend(v.iter().cloned());
                combined
            }
            // Case 2: first is Multiple, second is singular - append second to first's vec
            (ExecutorError::Multiple(v), _) => {
                let mut combined = v.clone();
                combined.push(second);
                combined
            }
            // Case 4: both singular - create new vec
            _ => vec![first, second],
        };
        ExecutorError::Multiple(errors).into()
    }

    /// Combine an optional error with a new error
    pub fn combine_opt(
        existing: Option<ArcExecutorError>,
        new: ArcExecutorError,
    ) -> ArcExecutorError {
        match existing {
            Some(e) => Self::combine(e, new),
            None => new,
        }
    }
}

/// Cache-specific errors
#[derive(Error, Debug)]
pub enum CacheError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Cache directory creation failed
    #[error("Cache directory creation failed: {0}")]
    DirCreation(String),
}
