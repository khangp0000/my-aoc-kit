//! Core solver trait and related types

use crate::error::{ParseError, SolveError};

/// Result of solving a part, containing the answer and optional partial data
#[derive(Debug, Clone)]
pub struct PartResult<T> {
    /// The displayable answer for this part
    pub answer: String,
    /// Optional intermediate data to pass to subsequent parts
    pub partial: Option<T>,
}

/// Core trait that all Advent of Code solvers must implement
///
/// Each solver handles a specific year-day problem and defines:
/// - How to parse the input string into an intermediate representation
/// - How to solve each part of the problem
/// - What data (if any) to share between parts
///
/// # Example
///
/// ```
/// use aoc_solver::{Solver, ParseError, PartResult, SolveError};
///
/// struct Day1Solver;
///
/// impl Solver for Day1Solver {
///     type Parsed = Vec<i32>;
///     type PartialResult = ();  // No data shared between parts
///
///     fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
///         input.lines()
///             .map(|line| line.parse().map_err(|_| 
///                 ParseError::InvalidFormat("Expected integer".to_string())))
///             .collect()
///     }
///
///     fn solve_part(
///         parsed: &Self::Parsed,
///         part: usize,
///         _previous_partial: Option<&Self::PartialResult>,
///     ) -> Result<PartResult<Self::PartialResult>, SolveError> {
///         match part {
///             1 => Ok(PartResult {
///                 answer: parsed.iter().sum::<i32>().to_string(),
///                 partial: None,
///             }),
///             2 => Ok(PartResult {
///                 answer: parsed.iter().product::<i32>().to_string(),
///                 partial: None,
///             }),
///             _ => Err(SolveError::PartNotImplemented(part)),
///         }
///     }
/// }
/// ```
pub trait Solver {
    /// The intermediate parsed representation of the input
    type Parsed;
    
    /// The type of data that can be shared between parts
    /// Use `()` if parts are independent
    type PartialResult;
    
    /// Parse the input string into the intermediate representation
    ///
    /// # Arguments
    /// * `input` - The raw input string for this problem
    ///
    /// # Returns
    /// * `Ok(Parsed)` - Successfully parsed data
    /// * `Err(ParseError)` - Parsing failed with details
    fn parse(input: &str) -> Result<Self::Parsed, ParseError>;
    
    /// Solve a specific part of the problem
    ///
    /// # Arguments
    /// * `parsed` - The parsed input data
    /// * `part` - The part number (1, 2, etc.)
    /// * `previous_partial` - Data from the previous part, if available
    ///
    /// # Returns
    /// * `Ok(PartResult)` - The part was solved successfully
    /// * `Err(SolveError::PartNotImplemented)` - The part is not implemented
    /// * `Err(SolveError::SolveFailed)` - An error occurred while solving
    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError>;
}
