//! Solver instance implementation

use crate::error::{ParseError, SolveError};
use crate::solver::{Solver, SolverExt};
use chrono::{DateTime, TimeDelta, Utc};

/// Result from solving a puzzle part, including timing information
#[derive(Debug, Clone)]
pub struct SolveResult {
    /// The answer string
    pub answer: String,
    /// When solving started (UTC)
    pub solve_start: DateTime<Utc>,
    /// When solving completed (UTC)
    pub solve_end: DateTime<Utc>,
}

impl SolveResult {
    /// Get the solve duration as TimeDelta
    pub fn duration(&self) -> TimeDelta {
        self.solve_end - self.solve_start
    }
}

/// A solver instance for a specific problem with shared data
///
/// Manages the state for solving a specific year-day problem, including:
/// - The shared data (parsed input and intermediate results)
/// - Parse timing information (start and end timestamps)
pub struct SolverInstance<'a, S: Solver> {
    year: u16,
    day: u8,
    shared: S::SharedData<'a>,
    parse_start: DateTime<Utc>,
    parse_end: DateTime<Utc>,
}

impl<'a, S: Solver> SolverInstance<'a, S> {
    /// Create a new solver instance by parsing input
    ///
    /// Records parse timing internally.
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `input` - The raw input string to parse
    ///
    /// # Returns
    /// * `Ok(SolverInstance)` - Successfully parsed and created instance with timing
    /// * `Err(ParseError)` - Parsing failed
    pub fn new(year: u16, day: u8, input: &'a str) -> Result<Self, ParseError> {
        let parse_start = Utc::now();
        let shared = S::parse(input)?;
        let parse_end = Utc::now();

        Ok(Self {
            year,
            day,
            shared,
            parse_start,
            parse_end,
        })
    }
}

/// Type-erased interface for working with any solver through dynamic dispatch
///
/// This trait provides a uniform interface for interacting with different solver types.
/// The concrete `SolverInstance<S>` implements this trait, allowing the registry to work
/// with different solver types uniformly.
///
/// # Example
///
/// ```no_run
/// use aoc_solver::DynSolver;
///
/// fn example(mut solver: Box<dyn DynSolver>) -> Result<(), Box<dyn std::error::Error>> {
///     // Solve part 1
///     let result = solver.solve(1)?;
///     println!("Part 1: {} (took {:?})", result.answer, result.duration());
///
///     // Solve part 2
///     let result = solver.solve(2)?;
///     println!("Part 2: {} (took {:?})", result.answer, result.duration());
///     
///     // Access parse timing
///     println!("Parse took {:?}", solver.parse_duration());
///     
///     Ok(())
/// }
/// ```
pub trait DynSolver {
    /// Solve the specified part with timing
    ///
    /// # Arguments
    /// * `part` - The part number to solve (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(SolveResult)` - The part was solved successfully with timing info
    /// * `Err(SolveError)` - The part is not implemented or solving failed
    fn solve(&mut self, part: u8) -> Result<SolveResult, SolveError>;

    /// Get the parse start time (UTC)
    fn parse_start(&self) -> DateTime<Utc>;

    /// Get the parse end time (UTC)
    fn parse_end(&self) -> DateTime<Utc>;

    /// Get the year for this solver
    fn year(&self) -> u16;

    /// Get the day for this solver
    fn day(&self) -> u8;

    /// Get the number of parts this solver supports
    fn parts(&self) -> u8;

    /// Convenience: get parse duration as TimeDelta
    fn parse_duration(&self) -> TimeDelta {
        self.parse_end() - self.parse_start()
    }
}

impl<'a, S: SolverExt> DynSolver for SolverInstance<'a, S> {
    fn solve(&mut self, part: u8) -> Result<SolveResult, SolveError> {
        let solve_start = Utc::now();
        let answer = S::solve_part_checked_range(&mut self.shared, part)?;
        let solve_end = Utc::now();

        Ok(SolveResult {
            answer,
            solve_start,
            solve_end,
        })
    }

    fn parse_start(&self) -> DateTime<Utc> {
        self.parse_start
    }

    fn parse_end(&self) -> DateTime<Utc> {
        self.parse_end
    }

    fn year(&self) -> u16 {
        self.year
    }

    fn day(&self) -> u8 {
        self.day
    }

    fn parts(&self) -> u8 {
        S::PARTS
    }
}
