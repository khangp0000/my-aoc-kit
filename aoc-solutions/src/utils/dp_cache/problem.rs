//! Trait-based DP problem definition.

use std::marker::PhantomData;

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
/// let cache = DpCache::builder()
///     .backend(VecBackend::new())
///     .problem(Fibonacci)
///     .build();
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

/// A wrapper that adapts closure functions to the `DpProblem` trait.
///
/// Use this when you want to define a DP problem using closures instead of
/// implementing the `DpProblem` trait on a custom struct.
///
/// # Type Parameters
///
/// - `I`: Index type for the DP cache
/// - `K`: Value type stored in the cache
/// - `D`: Dependency function type `Fn(&I) -> Vec<I>`
/// - `C`: Compute function type `Fn(&I, Vec<K>) -> K`
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpCache, ClosureProblem, VecBackend};
///
/// let fib = ClosureProblem::new(
///     |n: &usize| if *n <= 1 { vec![] } else { vec![n - 1, n - 2] },
///     |n: &usize, deps: Vec<u64>| if *n <= 1 { *n as u64 } else { deps[0] + deps[1] },
/// );
///
/// let cache = DpCache::builder()
///     .backend(VecBackend::new())
///     .problem(fib)
///     .build();
///
/// assert_eq!(cache.get(&10), 55);
/// ```
pub struct ClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    dep_fn: D,
    compute_fn: C,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, D, C> ClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    /// Creates a new `ClosureProblem` from dependency and compute closures.
    ///
    /// # Arguments
    ///
    /// - `deps`: A function that returns the indices this index depends on
    /// - `compute`: A function that computes the value given the index and resolved dependencies
    pub fn new(deps: D, compute: C) -> Self {
        Self {
            dep_fn: deps,
            compute_fn: compute,
            _phantom: PhantomData,
        }
    }
}

impl<I, K, D, C> DpProblem<I, K> for ClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    fn deps(&self, index: &I) -> Vec<I> {
        (self.dep_fn)(index)
    }

    fn compute(&self, index: &I, deps: Vec<K>) -> K {
        (self.compute_fn)(index, deps)
    }
}
