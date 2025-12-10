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
//! - [`VecBackend`]: Efficient for dense, sequential `usize` indices (auto-growing)
//! - [`ArrayBackend`]: Fixed-size 1D array with const generic size (zero-allocation)
//! - [`Array2DBackend`]: Fixed-size 2D array with const generic dimensions (zero-allocation)
//! - [`Vec2DBackend`]: Runtime-sized 2D Vec for grid problems
//! - [`HashMapBackend`]: Supports arbitrary hashable index types
//!
//! Parallel backends (for `ParallelDpCache`):
//! - [`DashMapBackend`]: Lock-free concurrent access using DashMap's sharded locking
//! - [`RwLockHashMapBackend`]: Simple RwLock around HashMap, good for read-heavy workloads
//! - [`ParallelArrayBackend`]: Thread-safe fixed-size 1D array (zero-allocation, lock-free reads)
//! - [`ParallelArray2DBackend`]: Thread-safe fixed-size 2D array (zero-allocation, lock-free reads)
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
//! assert_eq!(cache.get(&10).unwrap(), 55);
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
//! assert_eq!(cache.get(&27).unwrap(), 111);
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
//! assert_eq!(cache.get(&10).unwrap(), 55);
//! ```
//!
//! # Example: Fixed-size Array Backend
//!
//! For problems with known bounds, use `ArrayBackend` for zero-allocation caching:
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, ArrayBackend};
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
//! // Use ArrayBackend with const generic size
//! let cache = DpCache::builder()
//!     .backend(ArrayBackend::<u64, 21>::new())
//!     .problem(Fibonacci)
//!     .build();
//! assert_eq!(cache.get(&20).unwrap(), 6765);
//! ```
//!
//! # Example: 2D Grid Backend
//!
//! For 2D grid problems, use `Array2DBackend` or `Vec2DBackend`:
//!
//! ```rust
//! use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, Array2DBackend};
//!
//! struct GridPaths;
//!
//! impl DpProblem<(usize, usize), u64> for GridPaths {
//!     fn deps(&self, pos: &(usize, usize)) -> Vec<(usize, usize)> {
//!         let (r, c) = *pos;
//!         if r == 0 && c == 0 { vec![] }
//!         else if r == 0 { vec![(0, c - 1)] }
//!         else if c == 0 { vec![(r - 1, 0)] }
//!         else { vec![(r - 1, c), (r, c - 1)] }
//!     }
//!     fn compute(&self, _pos: &(usize, usize), deps: Vec<u64>) -> u64 {
//!         if deps.is_empty() { 1 } else { deps.iter().sum() }
//!     }
//! }
//!
//! let cache = DpCache::builder()
//!     .backend(Array2DBackend::<u64, 5, 5>::new())
//!     .problem(GridPaths)
//!     .build();
//! assert_eq!(cache.get(&(4, 4)).unwrap(), 70); // C(8,4) = 70 paths
//! ```

mod backend;
mod cache;
mod parallel;
mod problem;

pub use backend::{
    Array2DBackend, ArrayBackend, Backend, DashMapBackend, HashMapBackend, NoCacheBackend,
    ParallelArray2DBackend, ParallelArrayBackend, ParallelBackend, ParallelNoCacheBackend,
    RwLockHashMapBackend, Vec2DBackend, VecBackend,
};
pub use cache::DpCache;
pub use parallel::ParallelDpCache;
pub use problem::{ClosureProblem, DpProblem, ParallelDpProblem};

#[cfg(test)]
mod tests;
