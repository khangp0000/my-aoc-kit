//! Parallel DP cache implementation with pluggable backends.

use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::Arc;

use rayon::prelude::*;
use rayon::ThreadPool;

use super::backend::ParallelBackend;
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
/// let cache = ParallelDpCache::builder()
///     .backend(DashMapBackend::new())
///     .problem(Collatz)
///     .build();
/// assert_eq!(cache.get(&27).unwrap(), 111);
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
    /// Creates a new builder for ParallelDpCache.
    pub fn builder() -> ParallelDpCacheBuilder<I, K, B, P> {
        ParallelDpCacheBuilder::new()
    }

    /// Retrieves the value for the given index, computing it if necessary.
    ///
    /// If the value is already cached, returns a clone of the cached value.
    /// Otherwise, resolves all dependencies in parallel using Rayon, computes
    /// the value using the compute function, caches it, and returns a clone.
    pub fn get(&self, index: &I) -> Result<K, I> {
        // Fast path: check if already computed
        if let Some(value) = self.backend.get(index) {
            return Ok(value);
        }

        // Get dependencies (no locks held)
        let deps = self.problem.deps(index);

        // Resolve dependencies IN PARALLEL using Rayon (no locks held)
        let resolve_deps = || {
            deps.into_par_iter()
                .map(|dep| self.get(&dep))
                .collect::<Result<Vec<K>, I>>()
        };

        let dep_values = match &self.pool {
            Some(pool) => pool.install(resolve_deps),
            None => resolve_deps(),
        }?;

        // Insert using get_or_insert - only compute is inside the closure
        // dep_values is already resolved outside, so no recursive calls happen while holding the lock
        self.backend
            .get_or_insert(index.clone(), || self.problem.compute(index, dep_values))
    }
}

// =============================================================================
// Builder for ParallelDpCache
// =============================================================================

/// Builder for constructing a `ParallelDpCache`.
///
/// Supports const construction when backend and problem are set via const methods.
/// Note: Thread pool cannot be set in const context.
///
/// # Example (runtime)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelDpCache, DashMapBackend, DpProblem};
///
/// struct Collatz;
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
/// let cache = ParallelDpCache::builder()
///     .backend(DashMapBackend::new())
///     .problem(Collatz)
///     .build();
/// ```
///
/// # Example (const)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelDpCache, ParallelDpCacheBuilder, ParallelArrayBackend, DpProblem};
///
/// struct Fibonacci;
/// impl DpProblem<usize, u64> for Fibonacci {
///     fn deps(&self, n: &usize) -> Vec<usize> {
///         if *n <= 1 { vec![] } else { vec![n - 1, n - 2] }
///     }
///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
///         if *n <= 1 { *n as u64 } else { deps[0] + deps[1] }
///     }
/// }
///
/// const CACHE: ParallelDpCache<usize, u64, ParallelArrayBackend<u64, 21>, Fibonacci> =
///     ParallelDpCacheBuilder::new()
///         .with_backend(ParallelArrayBackend::new())
///         .with_problem(Fibonacci)
///         .build();
/// ```
pub struct ParallelDpCacheBuilder<I, K, B, P> {
    backend: Option<B>,
    problem: Option<P>,
    pool: Option<Arc<ThreadPool>>,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, P> ParallelDpCacheBuilder<I, K, B, P> {
    fn new() -> Self {
        Self {
            backend: None,
            problem: None,
            pool: None,
            _phantom: PhantomData,
        }
    }
}

impl<I, K, B, P> ParallelDpCacheBuilder<I, K, B, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
    P: ParallelDpProblem<I, K>,
{
    /// Sets the backend for the cache.
    pub fn backend(mut self, backend: B) -> Self {
        self.backend = Some(backend);
        self
    }

    /// Sets the problem definition for the cache.
    pub fn problem(mut self, problem: P) -> Self {
        self.problem = Some(problem);
        self
    }

    /// Sets a custom Rayon thread pool for parallel execution.
    pub fn pool(mut self, pool: Arc<ThreadPool>) -> Self {
        self.pool = Some(pool);
        self
    }

    /// Builds the ParallelDpCache.
    ///
    /// # Panics
    ///
    /// Panics if backend or problem is not set.
    pub fn build(self) -> ParallelDpCache<I, K, B, P> {
        ParallelDpCache {
            backend: self.backend.expect("backend is required"),
            problem: self.problem.expect("problem is required"),
            pool: self.pool,
            _phantom: PhantomData,
        }
    }
}

impl<I, K, B, P> ParallelDpCache<I, K, B, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
    P: ParallelDpProblem<I, K>,
{
    /// Creates a new `ParallelDpCache` directly (const-compatible).
    ///
    /// Use this when you need to construct a `ParallelDpCache` at compile time.
    /// Both the backend and problem must support const construction.
    /// Note: Thread pool will be `None` when using const construction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aoc_solutions::utils::dp_cache::{ParallelDpCache, ParallelArrayBackend, DpProblem};
    ///
    /// struct Fibonacci;
    /// impl DpProblem<usize, u64> for Fibonacci {
    ///     fn deps(&self, n: &usize) -> Vec<usize> {
    ///         if *n <= 1 { vec![] } else { vec![n - 1, n - 2] }
    ///     }
    ///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
    ///         if *n <= 1 { *n as u64 } else { deps[0] + deps[1] }
    ///     }
    /// }
    ///
    /// const CACHE: ParallelDpCache<usize, u64, ParallelArrayBackend<u64, 21>, Fibonacci> =
    ///     ParallelDpCache::new_const(ParallelArrayBackend::new(), Fibonacci);
    /// ```
    pub const fn new_const(backend: B, problem: P) -> Self {
        Self {
            backend,
            problem,
            pool: None,
            _phantom: PhantomData,
        }
    }
}

