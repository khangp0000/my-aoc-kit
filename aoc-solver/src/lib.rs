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
//! use aoc_solver::{AocParser, ParseError, RegistryBuilder, SolveError, Solver, SolverInstanceCow};
//! use std::borrow::Cow;
//!
//! // Define a solver
//! pub struct MyDay1;
//!
//! impl AocParser for MyDay1 {
//!     type SharedData = Vec<i32>;
//!     
//!     fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
//!         input.lines()
//!             .map(|line| line.parse().map_err(|_|
//!                 ParseError::InvalidFormat("Expected integer".to_string())))
//!             .collect::<Result<Vec<_>, _>>()
//!             .map(Cow::Owned)
//!     }
//! }
//!
//! impl Solver for MyDay1 {
//!     const PARTS: u8 = 1;
//!     
//!     fn solve_part(
//!         shared: &mut Cow<'_, Self::SharedData>,
//!         part: u8,
//!     ) -> Result<String, SolveError> {
//!         match part {
//!             1 => Ok(shared.iter().sum::<i32>().to_string()),
//!             _ => Err(SolveError::PartNotImplemented(part)),
//!         }
//!     }
//! }
//!
//! // Use the solver with builder pattern
//! let builder = RegistryBuilder::new();
//! let builder = builder.register(2023, 1, |input: &str| {
//!     let shared = MyDay1::parse(input)?;
//!     Ok(Box::new(SolverInstanceCow::<MyDay1>::new(2023, 1, shared)))
//! }).unwrap();
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
//! - How to parse input (`SharedData` type and `parse()` method)
//! - How to solve each part (`solve_part()` method with mutable access to shared data)
//!
//! ## DynSolver Trait
//!
//! The [`DynSolver`] trait provides type erasure for working with different solver types uniformly.
//! Key methods:
//! - `solve(part)`: Computes the result for a specific part
//!
//! ## Plugin System and Derive Macro
//!
//! Use `#[derive(AutoRegisterSolver)]` to automatically register solvers:
//! ```ignore
//! #[derive(AutoRegisterSolver)]
//! #[aoc(year = 2023, day = 1, tags = ["easy"])]
//! struct Day1Solver;
//! ```
//!
//! ## Part Dependencies
//!
//! Parts can share data through mutations to the `SharedData` structure:
//! - **Independent**: Parts don't modify shared data
//! - **Dependent**: Part 1 stores data in `SharedData`, Part 2 reads it
//!
//! See the examples directory for complete demonstrations.

mod error;
mod instance;
mod registry;
mod solver;

// Re-export public API
pub use error::{ParseError, RegistrationError, SolveError, SolverError};
pub use instance::{DynSolver, SolverInstance, SolverInstanceCow};
pub use registry::{
    BASE_YEAR, CAPACITY, DAYS_PER_YEAR, FactoryInfo, FactoryRegistryBuilder, MAX_YEARS,
    RegisterableFactory, RegisterableSolver, RegistryBuilder, SolverFactory, SolverFactoryRegistry,
    SolverFactoryStorage, SolverFactorySync, SolverPlugin, SolverRegistry,
};
pub use solver::{AocParser, PartSolver, Solver, SolverExt};

// Re-export inventory for use by the derive macro
pub use inventory;

// Re-export the derive macros
pub use aoc_solver_macros::{AocSolver, AutoRegisterSolver};
