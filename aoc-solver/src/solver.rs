//! Core solver trait and related types

use crate::error::{ParseError, SolveError};
use std::borrow::Cow;

/// Trait for parsing AOC puzzle input into shared data
///
/// This trait defines the shared data type and parsing logic for a solver,
/// providing clean separation between parsing and solving concerns.
///
/// # Example
///
/// ```
/// use aoc_solver::{AocParser, ParseError};
/// use std::borrow::Cow;
///
/// struct Day1;
///
/// impl AocParser for Day1 {
///     type SharedData = Vec<i32>;
///     
///     fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         let numbers: Vec<i32> = input
///             .lines()
///             .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
///             .collect::<Result<_, _>>()?;
///         Ok(Cow::Owned(numbers))
///     }
/// }
/// ```
pub trait AocParser {
    /// The shared data structure that holds parsed input and intermediate results.
    /// Must implement ToOwned to support zero-copy via Cow.
    /// Use `?Sized` to allow unsized types like `str` for true zero-copy parsing.
    type SharedData: ToOwned + ?Sized;

    /// Parse the input string into the shared data structure.
    ///
    /// Returns `Cow::Owned` for transformed data, or `Cow::Borrowed` for zero-copy.
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError>;
}

/// Trait for solving a specific part of an AOC puzzle.
///
/// The const generic `N` represents the part number (1, 2, etc.).
/// This provides compile-time validation that the part is implemented.
///
/// # Example
///
/// ```
/// use aoc_solver::{AocParser, PartSolver, ParseError, SolveError};
/// use std::borrow::Cow;
///
/// struct Day1;
///
/// impl AocParser for Day1 {
///     type SharedData = Vec<i32>;
///     
///     fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         let numbers: Vec<i32> = input
///             .lines()
///             .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
///             .collect::<Result<_, _>>()?;
///         Ok(Cow::Owned(numbers))
///     }
/// }
///
/// impl PartSolver<1> for Day1 {
///     fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
///         Ok(shared.iter().sum::<i32>().to_string())
///     }
/// }
/// ```
pub trait PartSolver<const N: u8>: AocParser {
    /// Solve this part of the puzzle.
    ///
    /// # Arguments
    /// * `shared` - Mutable reference to Cow containing shared data.
    ///   - For read-only operations: just read from `shared` (zero-copy)
    ///   - For mutations: call `shared.to_mut()` to get owned data (triggers clone if borrowed)
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError)` - An error occurred while solving
    fn solve(shared: &mut Cow<'_, Self::SharedData>) -> Result<String, SolveError>;
}

/// Core trait that all Advent of Code solvers must implement.
///
/// Extends `AocParser` to inherit `SharedData` type and `parse()` function.
/// Each solver handles a specific year-day problem and defines:
/// - How to solve each part of the problem using mutable access to shared data
///
/// # Example
///
/// ```
/// use aoc_solver::{AocParser, ParseError, SolveError, Solver};
/// use std::borrow::Cow;
///
/// struct Day1Solver;
///
/// #[derive(Debug, Clone)]
/// struct SharedData {
///     numbers: Vec<i32>,
/// }
///
/// impl AocParser for Day1Solver {
///     type SharedData = SharedData;
///
///     fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         let numbers = input.lines()
///             .map(|line| line.parse().map_err(|_|
///                 ParseError::InvalidFormat("Expected integer".to_string())))
///             .collect::<Result<Vec<_>, _>>()?;
///         Ok(Cow::Owned(SharedData { numbers }))
///     }
/// }
///
/// impl Solver for Day1Solver {
///     const PARTS: u8 = 2;
///
///     fn solve_part(
///         shared: &mut Cow<'_, Self::SharedData>,
///         part: u8,
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
pub trait Solver: AocParser {
    /// Number of parts this solver implements
    const PARTS: u8;

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
        part: u8,
    ) -> Result<String, SolveError>;
}

pub trait SolverExt: Solver {
    fn solve_part_checked_range(
        shared: &mut Cow<'_, Self::SharedData>,
        part: u8,
    ) -> Result<String, SolveError> {
        if (1..=Self::PARTS).contains(&part) {
            Self::solve_part(shared, part)
        } else {
            Err(SolveError::PartOutOfRange(part))
        }
    }
}

impl<T: Solver + ?Sized> SolverExt for T {}