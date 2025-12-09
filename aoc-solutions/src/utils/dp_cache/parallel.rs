//! Parallel DP cache implementation with pluggable backends.

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::ThreadPool;

use super::backend::{DashMapBackend, ParallelBackend, RwLockHashMapBackend};
use super::problem::ParallelDpProblem;

/// A parallel dynamic programming cache with pluggable backend storage.
///
/// This cache resolves dependencies in parallel using Rayon and supports
/// different backend storage implementations.
///
/// # Type Parameters
///
/// - `I`: Index type (must implement `Hash + Eq + Clone + Send + Sync`)
/// - `K`: Value type (must implement `Clone + Send + Sync`)
/// - `B`: Backend type (must implement `ParallelBackend<I, K>`)
/// - `P`: Problem type (must implement `ParallelDpProblem<I, K>`)
///
/// # Warning: No Cycle Detection
///
/// This cache does NOT detect cycles in the dependency graph. If cycles exist,
/// the behavior is undefined and may result in deadlock or stack overflow.
/// **Users MUST ensure dependencies form a DAG.**
///
/// # Example (with DashMapBackend)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelDpCache, DashMapBackend, DpProblem};
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
/// let cache = ParallelDpCache::with_problem(DashMapBackend::new(), Collatz);
/// assert_eq!(cache.get(&27), 111);
/// ```
pub struct ParallelDpCache<I, K, B, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
    P: ParallelDpProblem<I, K>,
{
    backend: B,
    problem: P,
    pool: Option<Arc<ThreadPool>>,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, P> ParallelDpCache<I, K, B, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
    P: ParallelDpProblem<I, K>,
{
    /// Creates a new ParallelDpCache with the given backend and problem definition.
    pub fn with_problem(backend: B, problem: P) -> Self {
        Self {
            backend,
            problem,
            pool: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new ParallelDpCache with a custom Rayon thread pool.
    pub fn with_problem_and_pool(backend: B, problem: P, pool: Arc<ThreadPool>) -> Self {
        Self {
            backend,
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
    pub fn get(&self, index: &I) -> K {
        // Fast path: check if already computed
        if let Some(value) = self.backend.get(index) {
            return value;
        }

        // Get dependencies (no locks held)
        let deps = self.problem.deps(index);

        // Resolve dependencies IN PARALLEL using Rayon (no locks held)
        let resolve_deps = || {
            deps.into_par_iter()
                .map(|dep| self.get(&dep))
                .collect::<Vec<K>>()
        };

        let dep_values = match &self.pool {
            Some(pool) => pool.install(resolve_deps),
            None => resolve_deps(),
        };

        // Insert using get_or_insert - only compute is inside the closure
        // dep_values is already resolved outside, so no recursive calls happen while holding the lock
        self.backend
            .get_or_insert(index.clone(), || self.problem.compute(index, dep_values))
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
type DummyParallelProblem<I, K> =
    ParallelClosureProblem<I, K, fn(&I) -> Vec<I>, fn(&I, Vec<K>) -> K>;

impl<I, K, B> ParallelDpCache<I, K, B, DummyParallelProblem<I, K>>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
{
    /// Creates a new ParallelDpCache with closure-based dependency and compute functions.
    ///
    /// This is a convenience constructor for the closure-based API.
    pub fn new<D, C>(
        backend: B,
        dep_fn: D,
        compute_fn: C,
    ) -> ParallelDpCache<I, K, B, ParallelClosureProblem<I, K, D, C>>
    where
        D: Fn(&I) -> Vec<I> + Send + Sync,
        C: Fn(&I, Vec<K>) -> K + Send + Sync,
    {
        ParallelDpCache {
            backend,
            problem: ParallelClosureProblem {
                dep_fn,
                compute_fn,
                _phantom: PhantomData,
            },
            pool: None,
            _phantom: PhantomData,
        }
    }

    /// Creates a new ParallelDpCache with closure-based functions and a custom thread pool.
    pub fn with_pool<D, C>(
        backend: B,
        dep_fn: D,
        compute_fn: C,
        pool: Arc<ThreadPool>,
    ) -> ParallelDpCache<I, K, B, ParallelClosureProblem<I, K, D, C>>
    where
        D: Fn(&I) -> Vec<I> + Send + Sync,
        C: Fn(&I, Vec<K>) -> K + Send + Sync,
    {
        ParallelDpCache {
            backend,
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

// =============================================================================
// Type Aliases for Convenience
// =============================================================================

/// A parallel DP cache using DashMap as the backend.
///
/// This is a convenience type alias for the common case of using DashMap.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DashMapDpCache, DashMapBackend, DpProblem, ParallelDpCache};
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
/// let cache: DashMapDpCache<_, _, _> = ParallelDpCache::with_problem(DashMapBackend::new(), Collatz);
/// assert_eq!(cache.get(&27), 111);
/// ```
pub type DashMapDpCache<I, K, P> = ParallelDpCache<I, K, DashMapBackend<I, K>, P>;

/// A parallel DP cache using RwLock<HashMap> as the backend.
///
/// This is a convenience type alias for using RwLock<HashMap>.
pub type RwLockDpCache<I, K, P> = ParallelDpCache<I, K, RwLockHashMapBackend<I, K>, P>;
