//! Core solver trait and related types

use crate::error::{ParseError, SolveError};

/// Trait for parsing AOC puzzle input into shared data
///
/// This trait defines the shared data type and parsing logic for a solver,
/// providing clean separation between parsing and solving concerns.
///
/// # Example
///
/// ```
/// use aoc_solver::{AocParser, ParseError};
///
/// struct Day1;
///
/// impl AocParser for Day1 {
///     type SharedData<'a> = Vec<i32>;
///     
///     fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
///         input
///             .lines()
///             .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
///             .collect()
///     }
/// }
/// ```
pub trait AocParser {
    /// The shared data structure that holds parsed input and intermediate results.
    ///
    /// Use any ownership strategy:
    /// - `Vec<T>` or custom structs for owned data (simplest, supports mutation)
    /// - `&'a str` for zero-copy borrowed data when no transformation is needed
    type SharedData<'a>;

    /// Parse the input string into the shared data structure.
    fn parse<'a>(input: &'a str) -> Result<Self::SharedData<'a>, ParseError>;
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
///
/// struct Day1;
///
/// impl AocParser for Day1 {
///     type SharedData<'a> = Vec<i32>;
///     
///     fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
///         input
///             .lines()
///             .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
///             .collect()
///     }
/// }
///
/// impl PartSolver<1> for Day1 {
///     fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
///         Ok(shared.iter().sum::<i32>().to_string())
///     }
/// }
/// ```
pub trait PartSolver<const N: u8>: AocParser {
    /// Solve this part of the puzzle.
    ///
    /// # Arguments
    /// * `shared` - Mutable reference to shared data
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError)` - An error occurred while solving
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError>;
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
///
/// struct Day1Solver;
///
/// #[derive(Debug, Clone)]
/// struct SharedData {
///     numbers: Vec<i32>,
/// }
///
/// impl AocParser for Day1Solver {
///     type SharedData<'a> = SharedData;
///
///     fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
///         let numbers = input.lines()
///             .map(|line| line.parse().map_err(|_|
///                 ParseError::InvalidFormat("Expected integer".to_string())))
///             .collect::<Result<Vec<_>, _>>()?;
///         Ok(SharedData { numbers })
///     }
/// }
///
/// impl Solver for Day1Solver {
///     const PARTS: u8 = 2;
///
///     fn solve_part(
///         shared: &mut Self::SharedData<'_>,
///         part: u8,
///     ) -> Result<String, SolveError> {
///         match part {
///             1 => {
///                 // Part 1: Sum all numbers
///                 let sum: i32 = shared.numbers.iter().sum();
///                 Ok(sum.to_string())
///             }
///             2 => {
///                 // Part 2: Product of all numbers
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
    /// * `shared` - Mutable reference to shared data (parsed input and intermediate results)
    /// * `part` - The part number (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError::PartNotImplemented)` - The part is not implemented
    /// * `Err(SolveError::SolveFailed)` - An error occurred while solving
    fn solve_part(shared: &mut Self::SharedData<'_>, part: u8) -> Result<String, SolveError>;
}

pub trait SolverExt: Solver {
    fn solve_part_checked_range(
        shared: &mut Self::SharedData<'_>,
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
