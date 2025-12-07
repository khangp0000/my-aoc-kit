//! Solver instance implementation

use crate::error::SolveError;
use crate::solver::{Solver, SolverExt};

/// A solver instance for a specific problem with shared data
///
/// Manages the state for solving a specific year-day problem, including:
/// - The shared data (parsed input and intermediate results)
pub struct SolverInstance<'a, S: Solver> {
    year: u16,
    day: u8,
    shared: S::SharedData<'a>,
}

impl<'b, 'a, S: Solver> SolverInstance<'a, S> {
    /// Create a new solver instance
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `shared` - The shared data (parsed input)
    pub fn new(year: u16, day: u8, shared: S::SharedData<'a>) -> Self {
        Self { year, day, shared }
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
///     let answer = solver.solve(1)?;
///     println!("Part 1: {}", answer);
///
///     // Solve part 2
///     let answer = solver.solve(2)?;
///     println!("Part 2: {}", answer);
///     
///     Ok(())
/// }
/// ```
pub trait DynSolver {
    /// Solve the specified part
    ///
    /// # Arguments
    /// * `part` - The part number to solve (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(String)` - The part was solved successfully and the answer
    /// * `Err(SolveError)` - The part is not implemented or solving failed
    fn solve(&mut self, part: u8) -> Result<String, SolveError>;

    /// Get the year for this solver
    fn year(&self) -> u16;

    /// Get the day for this solver
    fn day(&self) -> u8;

    /// Get the number of parts this solver supports
    fn parts(&self) -> u8;
}

impl<'a, 'b, S: SolverExt> DynSolver for SolverInstance<'a, S> {
    fn solve(&mut self, part: u8) -> Result<String, SolveError> {
        S::solve_part_checked_range(&mut self.shared, part)
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
