//! Core solver trait and related types

use crate::error::{ParseError, SolveError};
use std::borrow::Cow;

/// Core trait that all Advent of Code solvers must implement
///
/// Each solver handles a specific year-day problem and defines:
/// - How to parse the input string into shared data
/// - How to solve each part of the problem using mutable access to shared data
///
/// # Example
///
/// ```
/// use aoc_solver::{ParseError, SolveError, Solver};
/// use std::borrow::Cow;
///
/// struct Day1Solver;
///
/// #[derive(Debug, Clone)]
/// struct SharedData {
///     numbers: Vec<i32>,
/// }
///
/// impl Solver for Day1Solver {
///     type SharedData = SharedData;
///
///     fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         let numbers = input.lines()
///             .map(|line| line.parse().map_err(|_|
///                 ParseError::InvalidFormat("Expected integer".to_string())))
///             .collect::<Result<Vec<_>, _>>()?;
///         Ok(Cow::Owned(SharedData { numbers }))
///     }
///
///     fn solve_part(
///         shared: &mut Cow<'_, Self::SharedData>,
///         part: usize,
///     ) -> Result<String, SolveError> {
///         match part {
///             1 => {
///                 // Part 1: Sum all numbers (read-only, zero-copy)
///                 let sum: i32 = shared.numbers.iter().sum();
///                 Ok(sum.to_string())
///             }
///             2 => {
///                 // Part 2: Product of all numbers (read-only, zero-copy)
///                 let product: i32 = shared.numbers.iter().product();
///                 Ok(product.to_string())
///             }
///             _ => Err(SolveError::PartNotImplemented(part)),
///         }
///     }
/// }
/// ```
pub trait Solver {
    /// The shared data structure that holds parsed input and intermediate results
    type SharedData: ToOwned;

    /// Parse the input string into the shared data structure
    ///
    /// # Arguments
    /// * `input` - The raw input string for this problem
    ///
    /// # Returns
    /// * `Ok(SharedData)` - Successfully parsed data
    /// * `Err(ParseError)` - Parsing failed with details
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError>;

    /// Solve a specific part of the problem
    ///
    /// # Arguments
    /// * `shared` - Mutable reference to Cow containing shared data (parsed input and intermediate results).
    ///   Solvers can work with borrowed data or call `.to_mut()` to get owned data when mutation is needed
    /// * `part` - The part number (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError::PartNotImplemented)` - The part is not implemented
    /// * `Err(SolveError::SolveFailed)` - An error occurred while solving
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError>;
}
