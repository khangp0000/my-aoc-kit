//! Dynamic Programming Cache with Lazy Evaluation
//!
//! This module provides memoization caches for dynamic programming problems where values
//! depend on other values in a directed acyclic graph (DAG).
//!
//! # Cache Types
//!
//! - [`DpCache`]: Single-threaded cache with `RefCell` for interior mutability
//! - [`DashMapDpCache`]: Thread-safe cache with parallel dependency resolution using Rayon
//!
//! # Backend Types (for `DpCache`)
//!
//! - [`VecBackend`]: Efficient for dense, sequential `usize` indices
//! - [`HashMapBackend`]: Supports arbitrary hashable index types
//!
//! # Warning: Cycle Behavior
//!
//! **These caches do NOT support cycle detection.** If the dependency graph contains cycles:
//! - `DpCache`: Stack overflow or infinite loop
//! - `DashMapDpCache`: Deadlock or stack overflow
//!
//! **Users MUST ensure that dependencies form a DAG (Directed Acyclic Graph).**
//!
//! # Example: Trait-based API (recommended)
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, VecBackend};
//!
//! struct Fibonacci;
//!
//! impl DpProblem<usize, u64> for Fibonacci {
//!     fn deps(&self, n: &usize) -> Vec<usize> {
//!         if *n <= 1 { vec![] }
//!         else { vec![n - 1, n - 2] }
//!     }
//!     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
//!         if *n <= 1 { *n as u64 }
//!         else { deps[0] + deps[1] }
//!     }
//! }
//!
//! let cache = DpCache::with_problem(VecBackend::new(), Fibonacci);
//! assert_eq!(cache.get(10), 55);
//! ```
//!
//! # Example: Trait-based Parallel
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DashMapDpCache, DpProblem};
//!
//! struct Collatz;
//!
//! impl DpProblem<u64, u64> for Collatz {
//!     fn deps(&self, n: &u64) -> Vec<u64> {
//!         if *n <= 1 { vec![] }
//!         else if n % 2 == 0 { vec![n / 2] }
//!         else { vec![3 * n + 1] }
//!     }
//!     fn compute(&self, _n: &u64, deps: Vec<u64>) -> u64 {
//!         if deps.is_empty() { 0 } else { 1 + deps[0] }
//!     }
//! }
//!
//! let cache = DashMapDpCache::with_problem(Collatz);
//! assert_eq!(cache.get(27), 111);
//! ```
//!
//! # Example: Closure-based API
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DpCache, VecBackend};
//!
//! // Fibonacci using closures
//! let cache = DpCache::new(
//!     VecBackend::new(),
//!     |n: &usize| {
//!         if *n <= 1 { vec![] }
//!         else { vec![n - 1, n - 2] }
//!     },
//!     |n: &usize, deps: Vec<u64>| {
//!         if *n <= 1 { *n as u64 }
//!         else { deps[0] + deps[1] }
//!     },
//! );
//!
//! assert_eq!(cache.get(10), 55);
//! ```

mod backend;
mod cache;
mod parallel;
mod problem;

pub use backend::{Backend, HashMapBackend, VecBackend};
pub use cache::DpCache;
pub use parallel::DashMapDpCache;
pub use problem::{DpProblem, ParallelDpProblem};

#[cfg(test)]
mod tests;
