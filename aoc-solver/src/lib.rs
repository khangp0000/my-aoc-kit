//! Advent of Code Solver Library
//!
//! A flexible and type-safe framework for solving Advent of Code problems across multiple years and days.
//! Each problem is implemented as a solver with custom input parsing and can produce results
//! for multiple parts.
//!
//! # Overview
//!
//! This library provides:
//! - A trait-based interface for defining solvers
//! - Support for both independent and dependent parts
//! - Type-safe parsing and result handling
//! - A registry system for managing multiple solvers
//! - Result caching to avoid redundant computation
//!
//! # Quick Example
//!
//! ```
//! use aoc_solver::{Solver, ParseError, PartResult, SolveError, RegistryBuilder, register_solver};
//!
//! // Define a solver
//! pub struct MyDay1;
//!
//! impl Solver for MyDay1 {
//!     type Parsed = Vec<i32>;
//!     type PartialResult = ();
//!     
//!     fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
//!         input.lines()
//!             .map(|line| line.parse().map_err(|_| 
//!                 ParseError::InvalidFormat("Expected integer".to_string())))
//!             .collect()
//!     }
//!     
//!     fn solve_part(
//!         parsed: &Self::Parsed,
//!         part: usize,
//!         _previous_partial: Option<&Self::PartialResult>,
//!     ) -> Result<PartResult<Self::PartialResult>, SolveError> {
//!         match part {
//!             1 => Ok(PartResult {
//!                 answer: parsed.iter().sum::<i32>().to_string(),
//!                 partial: None,
//!             }),
//!             _ => Err(SolveError::PartNotImplemented(part)),
//!         }
//!     }
//! }
//!
//! // Use the solver with builder pattern
//! let mut builder = RegistryBuilder::new();
//! register_solver!(builder, MyDay1, 2023, 1);
//! let registry = builder.build();
//!
//! let mut solver = registry.create_solver(2023, 1, "1\n2\n3").unwrap();
//! let answer = solver.solve(1).unwrap();
//! assert_eq!(answer, "6");
//! ```
//!
//! # Key Concepts
//!
//! ## Solver Trait
//!
//! The [`Solver`] trait is the core interface. Implement it to define:
//! - How to parse input (`Parsed` type and `parse()` method)
//! - What data to share between parts (`PartialResult` type)
//! - How to solve each part (`solve_part()` method)
//!
//! ## DynSolver Trait
//!
//! The [`DynSolver`] trait provides type erasure for working with different solver types uniformly.
//! Key methods:
//! - `solve(part)`: Computes and caches the result
//! - `results()`: Returns cached results without recomputation
//!
//! ## Plugin System and Derive Macro
//!
//! Use `#[derive(AocSolver)]` to automatically register solvers:
//! ```ignore
//! #[derive(AocSolver)]
//! #[aoc(year = 2023, day = 1, tags = ["easy"])]
//! struct Day1Solver;
//! ```
//!
//! ## Part Dependencies
//!
//! Parts can be:
//! - **Independent**: Use `type PartialResult = ()` and return `partial: None`
//! - **Dependent**: Define a custom `PartialResult` type and pass data between parts
//!
//! See the examples directory for complete demonstrations.

mod error;
mod solver;
mod instance;
mod registry;

// Re-export public API
pub use error::{ParseError, RegistrationError, SolveError, SolverError};
pub use instance::{DynSolver, SolverInstance};
pub use registry::{RegisterableSolver, RegistryBuilder, SolverFactory, SolverPlugin, SolverRegistry};
pub use solver::{PartResult, Solver};

// Re-export inventory for use by the derive macro
pub use inventory;

// Re-export the derive macro
pub use aoc_solver_macros::AocSolver;
