//! Advent of Code puzzle solutions with automatic registration
//!
//! This crate contains actual puzzle solutions organized by year.
//! Each solution uses the `AutoRegisterSolver` derive macro for automatic
//! plugin registration with the solver framework.

#[cfg(feature = "stress_test")]
pub mod stress_test;

#[cfg(feature = "my-solutions")]
pub mod my_solutions;
