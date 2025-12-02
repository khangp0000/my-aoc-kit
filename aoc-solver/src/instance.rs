//! Solver instance implementation

use crate::error::SolveError;
use crate::solver::Solver;

/// A solver instance for a specific problem with parsed input
///
/// Manages the state for solving a specific year-day problem, including:
/// - The parsed input data
/// - Cached results for each part
/// - Partial results that can be shared between parts
pub struct SolverInstance<S: Solver> {
    year: u32,
    day: u32,
    parsed: S::Parsed,
    results: Vec<Option<String>>,
    partial_results: Vec<Option<S::PartialResult>>,
}

impl<S: Solver> SolverInstance<S> {
    /// Create a new solver instance
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `parsed` - The parsed input data
    pub fn new(year: u32, day: u32, parsed: S::Parsed) -> Self {
        Self {
            year,
            day,
            parsed,
            results: Vec::new(),
            partial_results: Vec::new(),
        }
    }
}

/// Type-erased interface for working with any solver through dynamic dispatch
///
/// This trait provides a uniform interface for interacting with different solver types.
/// The concrete `SolverInstance<S>` implements this trait, allowing the registry to work
/// with different solver types uniformly.
///
/// # Important Behavior
///
/// - `solve(part)`: **Recomputes** the solution each time it's called, updates the cache,
///   and returns the result. Use this to compute solutions.
/// - `results()`: Returns the **cached** results without any recomputation. Use this to
///   access previously computed answers without redundant computation.
///
/// # Example
///
/// ```no_run
/// # use aoc_solver::DynSolver;
/// # fn example(mut solver: Box<dyn DynSolver>) -> Result<(), Box<dyn std::error::Error>> {
/// // Solve part 1 (computes and caches)
/// let answer = solver.solve(1)?;
/// println!("Part 1: {}", answer);
///
/// // Solve part 2 (computes and caches)
/// let answer = solver.solve(2)?;
/// println!("Part 2: {}", answer);
///
/// // Access all cached results without recomputation
/// let all_results = solver.results();
/// println!("All results: {:?}", all_results);
/// # Ok(())
/// # }
/// ```
pub trait DynSolver {
    /// Solve the specified part, recomputing the result each time
    ///
    /// The result is cached in the results vector and returned.
    /// Use `results()` to access cached results without recomputation.
    ///
    /// # Arguments
    /// * `part` - The part number to solve (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(String)` - The part was solved successfully and the answer
    /// * `Err(SolveError)` - The part is not implemented or solving failed
    fn solve(&mut self, part: usize) -> Result<String, SolveError>;
    
    /// Returns a reference to all cached results without recomputation
    ///
    /// Index corresponds to part number (0-indexed: results[0] is Part 1).
    ///
    /// # Returns
    /// A slice of optional strings, where:
    /// - `Some(answer)` indicates a solved part
    /// - `None` indicates an unsolved or unimplemented part
    fn results(&self) -> &[Option<String>];
    
    /// Get the year for this solver
    fn year(&self) -> u32;
    
    /// Get the day for this solver
    fn day(&self) -> u32;
}

impl<S: Solver> DynSolver for SolverInstance<S> {
    fn solve(&mut self, part: usize) -> Result<String, SolveError> {
        // Get the previous part's partial result (if solving part 2, get part 1's data)
        let previous_partial = if part > 1 {
            self.partial_results
                .get(part - 2)
                .and_then(|opt| opt.as_ref())
        } else {
            None
        };
        
        // Call the solver's solve_part method
        let result = S::solve_part(&self.parsed, part, previous_partial)?;
        
        // Store the answer string
        let index = part - 1; // Convert to 0-indexed
        if index >= self.results.len() {
            self.results.resize_with(index + 1, || None);
        }
        if index >= self.partial_results.len() {
            self.partial_results.resize_with(index + 1, || None);
        }
        self.results[index] = Some(result.answer.clone());
        
        // Store the partial result for the next part
        self.partial_results[index] = result.partial;
        
        Ok(result.answer)
    }
    
    fn results(&self) -> &[Option<String>] {
        &self.results
    }
    
    fn year(&self) -> u32 {
        self.year
    }
    
    fn day(&self) -> u32 {
        self.day
    }
}
