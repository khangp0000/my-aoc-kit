//! Parallel DP cache implementation using DashMap.

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use dashmap::DashMap;
use rayon::prelude::*;
use rayon::ThreadPool;

use super::problem::ParallelDpProblem;

/// A parallel dynamic programming cache using DashMap for storage.
///
/// This cache uses DashMap directly for storage, avoiding the lifetime issues
/// that arise from storing `OnceLock` references. It's designed for use cases
/// where the index type is hashable and you want lock-free concurrent access.
///
/// # Type Parameters
///
/// - `I`: Index type (must implement `Hash + Eq + Clone + Send + Sync`)
/// - `K`: Value type (must implement `Clone + Send + Sync`)
/// - `P`: Problem type (must implement `ParallelDpProblem<I, K>`)
///
/// # Warning: No Cycle Detection
///
/// This cache does NOT detect cycles in the dependency graph. If cycles exist,
/// the behavior is undefined and may result in deadlock or stack overflow.
/// **Users MUST ensure dependencies form a DAG.**
///
/// # Example (trait-based)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DashMapDpCache, DpProblem};
///
/// struct Collatz;
///
/// impl DpProblem<u64, u64> for Collatz {
///     fn deps(&self, n: &u64) -> Vec<u64> {
///         if *n <= 1 { vec![] }
///         else if n % 2 == 0 { vec![n / 2] }
///         else { vec![3 * n + 1] }
///     }
///     fn compute(&self, _n: &u64, deps: Vec<u64>) -> u64 {
///         if deps.is_empty() { 0 } else { 1 + deps[0] }
///     }
/// }
///
/// let cache = DashMapDpCache::with_problem(Collatz);
/// assert_eq!(cache.get(27), 111);
/// ```
///
/// # Example (closure-based)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::DashMapDpCache;
///
/// let cache = DashMapDpCache::new(
///     |n: &u64| {
///         if *n <= 1 { vec![] }
///         else if n % 2 == 0 { vec![n / 2] }
///         else { vec![3 * n + 1] }
///     },
///     |_n: &u64, deps: Vec<u64>| {
///         if deps.is_empty() { 0 }
///         else { 1 + deps[0] }
///     },
/// );
///
/// assert_eq!(cache.get(27), 111);
/// ```
pub struct DashMapDpCache<I, K, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    P: ParallelDpProblem<I, K>,
{
    data: DashMap<I, K>,
    problem: P,
    pool: Option<Arc<ThreadPool>>,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, P> DashMapDpCache<I, K, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    P: ParallelDpProblem<I, K>,
{
    /// Creates a new DashMapDpCache with the given problem definition.
    pub fn with_problem(problem: P) -> Self {
        Self {
            data: DashMap::new(),
            problem,
            pool: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new DashMapDpCache with a custom Rayon thread pool.
    pub fn with_problem_and_pool(problem: P, pool: Arc<ThreadPool>) -> Self {
        Self {
            data: DashMap::new(),
            problem,
            pool: Some(pool),
            _phantom: PhantomData,
        }
    }

    /// Retrieves the value for the given index, computing it if necessary.
    ///
    /// If the value is already cached, returns a clone of the cached value.
    /// Otherwise, resolves all dependencies in parallel using Rayon, computes
    /// the value using the compute function, caches it, and returns a clone.
    ///
    /// Note: We cannot use `or_insert_with` here because it holds a write lock
    /// on the DashMap shard while the closure executes. If the closure calls
    /// `self.get()` recursively and hits the same shard, it would deadlock.
    /// Instead, we compute the value first (releasing any locks), then insert.
    pub fn get(&self, index: I) -> K {
        // Fast path: check if already computed
        if let Some(entry) = self.data.get(&index) {
            return entry.value().clone();
        }

        // Get dependencies (no locks held)
        let deps = self.problem.deps(&index);

        // Resolve dependencies IN PARALLEL using Rayon (no locks held)
        let resolve_deps = || {
            deps.into_par_iter()
                .map(|dep| self.get(dep))
                .collect::<Vec<K>>()
        };

        let dep_values = match &self.pool {
            Some(pool) => pool.install(resolve_deps),
            None => resolve_deps(),
        };

        // Insert using or_insert_with - only compute is inside the closure
        // dep_values is already resolved outside, so no recursive calls happen while holding the lock
        self.data
            .entry(index.clone())
            .or_insert_with(|| self.problem.compute(&index, dep_values))
            .value()
            .clone()
    }
}

/// Wrapper to adapt closure functions to the ParallelDpProblem trait.
pub struct ParallelClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I> + Send + Sync,
    C: Fn(&I, Vec<K>) -> K + Send + Sync,
{
    dep_fn: D,
    compute_fn: C,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, D, C> super::problem::DpProblem<I, K> for ParallelClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I> + Send + Sync,
    C: Fn(&I, Vec<K>) -> K + Send + Sync,
{
    fn deps(&self, index: &I) -> Vec<I> {
        (self.dep_fn)(index)
    }

    fn compute(&self, index: &I, deps: Vec<K>) -> K {
        (self.compute_fn)(index, deps)
    }
}

// Dummy type for the default generic parameter
type DummyParallelProblem<I, K> = ParallelClosureProblem<I, K, fn(&I) -> Vec<I>, fn(&I, Vec<K>) -> K>;

impl<I, K> DashMapDpCache<I, K, DummyParallelProblem<I, K>>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
{
    /// Creates a new DashMapDpCache with closure-based dependency and compute functions.
    ///
    /// This is a convenience constructor for the closure-based API.
    pub fn new<D, C>(dep_fn: D, compute_fn: C) -> DashMapDpCache<I, K, ParallelClosureProblem<I, K, D, C>>
    where
        D: Fn(&I) -> Vec<I> + Send + Sync,
        C: Fn(&I, Vec<K>) -> K + Send + Sync,
    {
        DashMapDpCache {
            data: DashMap::new(),
            problem: ParallelClosureProblem {
                dep_fn,
                compute_fn,
                _phantom: PhantomData,
            },
            pool: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new DashMapDpCache with closure-based functions and a custom thread pool.
    pub fn with_pool<D, C>(
        dep_fn: D,
        compute_fn: C,
        pool: Arc<ThreadPool>,
    ) -> DashMapDpCache<I, K, ParallelClosureProblem<I, K, D, C>>
    where
        D: Fn(&I) -> Vec<I> + Send + Sync,
        C: Fn(&I, Vec<K>) -> K + Send + Sync,
    {
        DashMapDpCache {
            data: DashMap::new(),
            problem: ParallelClosureProblem {
                dep_fn,
                compute_fn,
                _phantom: PhantomData,
            },
            pool: Some(pool),
            _phantom: PhantomData,
        }
    }
}
