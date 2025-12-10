//! Dynamic Programming Cache with Lazy Evaluation
//!
//! This module provides memoization caches for dynamic programming problems where values
//! depend on other values in a directed acyclic graph (DAG).
//!
//! # Cache Types
//!
//! - [`DpCache`]: Single-threaded cache with `RefCell` for interior mutability
//! - [`ParallelDpCache`]: Thread-safe cache with parallel dependency resolution using Rayon
//!
//! # Backend Types
//!
//! Sequential backends (for `DpCache`):
//! - [`VecBackend`]: Efficient for dense, sequential `usize` indices
//! - [`HashMapBackend`]: Supports arbitrary hashable index types
//!
//! Parallel backends (for `ParallelDpCache`):
//! - [`DashMapBackend`]: Lock-free concurrent access using DashMap's sharded locking
//! - [`RwLockHashMapBackend`]: Simple RwLock around HashMap, good for read-heavy workloads
//!
//! # Warning: Cycle Behavior
//!
//! **These caches do NOT support cycle detection.** If the dependency graph contains cycles:
//! - `DpCache`: Stack overflow or infinite loop
//! - `ParallelDpCache`: Deadlock or stack overflow
//!
//! **Users MUST ensure that dependencies form a DAG (Directed Acyclic Graph).**
//!
//! # Example: Trait-based API with Builder (recommended)
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
//! let cache = DpCache::builder()
//!     .backend(VecBackend::new())
//!     .problem(Fibonacci)
//!     .build();
//! assert_eq!(cache.get(&10), 55);
//! ```
//!
//! # Example: Trait-based Parallel with Builder
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{ParallelDpCache, DashMapBackend, DpProblem};
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
//! let cache = ParallelDpCache::builder()
//!     .backend(DashMapBackend::new())
//!     .problem(Collatz)
//!     .build();
//! assert_eq!(cache.get(&27), 111);
//! ```
//!
//! # Example: Closure-based API with ClosureProblem
//!
//! For quick prototyping, you can use `ClosureProblem` instead of defining a struct:
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DpCache, ClosureProblem, VecBackend};
//!
//! let fib = ClosureProblem::new(
//!     |n: &usize| if *n <= 1 { vec![] } else { vec![n - 1, n - 2] },
//!     |n: &usize, deps: Vec<u64>| if *n <= 1 { *n as u64 } else { deps[0] + deps[1] },
//! );
//!
//! let cache = DpCache::builder()
//!     .backend(VecBackend::new())
//!     .problem(fib)
//!     .build();
//!
//! assert_eq!(cache.get(&10), 55);
//! ```

mod backend;
mod cache;
mod parallel;
mod problem;

pub use backend::{
    Backend, DashMapBackend, HashMapBackend, ParallelBackend, RwLockHashMapBackend, VecBackend,
};
pub use cache::DpCache;
pub use parallel::ParallelDpCache;
pub use problem::{ClosureProblem, DpProblem, ParallelDpProblem};

#[cfg(test)]
mod tests;
