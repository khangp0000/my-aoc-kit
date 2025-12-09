//! Trait-based DP problem definition.

/// A trait for defining dynamic programming problems.
///
/// Implement this trait to define the dependency structure and computation
/// logic for a DP problem. This provides a cleaner API than passing separate
/// closure functions.
///
/// # Type Parameters
///
/// - `I`: Index type for the DP cache
/// - `K`: Value type stored in the cache
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpProblem, DpCache, VecBackend};
///
/// struct Fibonacci;
///
/// impl DpProblem<usize, u64> for Fibonacci {
///     fn deps(&self, n: &usize) -> Vec<usize> {
///         if *n <= 1 { vec![] }
///         else { vec![n - 1, n - 2] }
///     }
///
///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
///         if *n <= 1 { *n as u64 }
///         else { deps[0] + deps[1] }
///     }
/// }
///
/// let cache = DpCache::with_problem(VecBackend::new(), Fibonacci);
/// assert_eq!(cache.get(&10), 55);
/// ```
pub trait DpProblem<I, K> {
    /// Returns the indices that this index depends on.
    ///
    /// For base cases, return an empty vector.
    fn deps(&self, index: &I) -> Vec<I>;

    /// Computes the value for the given index using resolved dependency values.
    ///
    /// The `deps` vector contains the computed values for each dependency
    /// returned by `deps()`, in the same order.
    fn compute(&self, index: &I, deps: Vec<K>) -> K;
}

/// Marker trait for DP problems that are safe to use in parallel contexts.
///
/// Implement this trait (in addition to `DpProblem`) when your problem
/// implementation is thread-safe and can be used with `DashMapDpCache`.
pub trait ParallelDpProblem<I, K>: DpProblem<I, K> + Send + Sync {}

// Blanket implementation: any DpProblem that is Send + Sync is also ParallelDpProblem
impl<T, I, K> ParallelDpProblem<I, K> for T where T: DpProblem<I, K> + Send + Sync {}
