//! Error types for the solver library

use thiserror::Error;

/// Error type for parsing input data
#[derive(Debug, Clone, Error)]
pub enum ParseError {
    /// Input format doesn't match expected structure
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    /// Required data is missing from input
    #[error("Missing data: {0}")]
    MissingData(String),
    /// Other parsing errors
    #[error("Parse error: {0}")]
    Other(String),
}

/// Error type for solving a specific part
#[derive(Debug, Error)]
pub enum SolveError {
    /// The requested part number is not implemented
    #[error("Part {0} is not implemented")]
    PartNotImplemented(u8),
    /// The requested part number is out of range (exceeds max_parts)
    #[error("Part {0} is out of range")]
    PartOutOfRange(u8),
    /// An error occurred while solving the part
    #[error("Solve failed: {0}")]
    SolveFailed(#[source] Box<dyn std::error::Error + Send + Sync>),
}

/// Error type for solver operations
#[derive(Debug, Error)]
pub enum SolverError {
    /// Solver not found for the given year and day
    #[error("Solver not found for year {0} day {1}")]
    NotFound(u16, u8),
    /// Error occurred during parsing
    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),
    /// Error occurred during solving
    #[error("Solve error: {0}")]
    SolveError(#[from] SolveError),
}

/// Error type for registration failures
#[derive(Debug, Clone, Error)]
pub enum RegistrationError {
    /// Attempted to register a solver for a year-day combination that already exists
    #[error("Duplicate solver registration for year {0} day {1}")]
    DuplicateSolver(u16, u8),
}
