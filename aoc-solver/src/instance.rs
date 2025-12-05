//! Solver instance implementation

use crate::error::SolveError;
use crate::solver::Solver;
use std::borrow::Cow;
use std::ops::Deref;

/// A solver instance for a specific problem with shared data
///
/// Manages the state for solving a specific year-day problem, including:
/// - The shared data (parsed input and intermediate results)
pub struct SolverInstanceCow<'a, S: Solver> {
    year: u32,
    day: u32,
    shared: Cow<'a, S::SharedData>,
}

impl<S: Solver> SolverInstanceCow<'_, S> {
    /// Convert a borrowed solver instance to an owned one
    ///
    /// This is useful when you need to extend the lifetime of a solver instance
    /// beyond the lifetime of the input data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let borrowed_solver = registry.create_solver(2023, 1, input)?;
    /// let owned_solver = borrowed_solver.into_owned();
    /// // owned_solver can now outlive the input string
    /// ```
    pub fn into_owned(&self) -> SolverInstance<S> {
        SolverInstanceCow {
            year: self.year,
            day: self.day,
            shared: Cow::Owned(self.shared.deref().to_owned()),
        }
    }
}

pub type SolverInstance<S> = SolverInstanceCow<'static, S>;

impl<'a, S: Solver> SolverInstanceCow<'a, S> {
    /// Create a new solver instance
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `shared` - The shared data (parsed input)
    pub fn new(year: u32, day: u32, shared: Cow<'a, S::SharedData>) -> Self {
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
    fn solve(&mut self, part: usize) -> Result<String, SolveError>;

    /// Get the year for this solver
    fn year(&self) -> u32;

    /// Get the day for this solver
    fn day(&self) -> u32;
}

impl<'a, S: Solver> DynSolver for SolverInstanceCow<'a, S> {
    fn solve(&mut self, part: usize) -> Result<String, SolveError> {
        S::solve_part(&mut self.shared, part)
    }

    fn year(&self) -> u32 {
        self.year
    }

    fn day(&self) -> u32 {
        self.day
    }
}
