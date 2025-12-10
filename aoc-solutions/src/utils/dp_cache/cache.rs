//! Single-threaded DP cache implementation.

use std::cell::RefCell;
use std::marker::PhantomData;

use super::backend::Backend;
use super::problem::DpProblem;

/// A dynamic programming cache with lazy evaluation and dependency resolution.
///
/// `DpCache` provides memoization for recursive computations where values may depend
/// on other values. Dependencies are resolved automatically and each value is computed
/// exactly once.
///
/// # Type Parameters
///
/// - `I`: Index type (must implement `Clone`)
/// - `K`: Value type (must implement `Clone`)
/// - `B`: Backend storage type (must implement `Backend<I, K>`)
/// - `P`: Problem type (must implement `DpProblem<I, K>`)
///
/// # Warning: No Cycle Detection
///
/// This cache does NOT detect cycles in the dependency graph. If cycles exist,
/// the behavior is undefined (stack overflow or infinite loop).
/// **Users MUST ensure dependencies form a DAG.**
///
/// # Example (trait-based)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, VecBackend};
///
/// struct Factorial;
///
/// impl DpProblem<usize, u64> for Factorial {
///     fn deps(&self, n: &usize) -> Vec<usize> {
///         if *n == 0 { vec![] } else { vec![n - 1] }
///     }
///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
///         if *n == 0 { 1 } else { (*n as u64) * deps[0] }
///     }
/// }
///
/// let cache = DpCache::builder()
///     .backend(VecBackend::new())
///     .problem(Factorial)
///     .build();
/// assert_eq!(cache.get(&5).unwrap(), 120);
/// ```
///
/// # Example (closure-based with ClosureProblem)
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
/// assert_eq!(cache.get(&10).unwrap(), 55);
/// ```
pub struct DpCache<I, K, B, P>
where
    B: Backend<I, K>,
    P: DpProblem<I, K>,
{
    backend: RefCell<B>,
    problem: P,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, P> DpCache<I, K, B, P>
where
    I: Clone,
    K: Clone,
    B: Backend<I, K>,
    P: DpProblem<I, K>,
{
    /// Creates a new builder for DpCache.
    pub fn builder() -> DpCacheBuilder<I, K, B, P> {
        DpCacheBuilder::new()
    }

    /// Retrieves the value for the given index, computing it if necessary.
    ///
    /// If the value is already cached, returns a clone of the cached value.
    /// Otherwise, resolves all dependencies recursively, computes the value
    /// using the compute function, caches it, and returns a clone.
    ///
    /// # Arguments
    ///
    /// - `index`: A reference to the index to retrieve the value for
    ///
    /// # Returns
    ///
    /// `Ok(value)` - The computed or cached value for the index.
    /// `Err(index)` - If the index cannot be stored (e.g., out of bounds for fixed-size backends).
    ///
    /// # Panics
    ///
    /// May panic or cause undefined behavior if the dependency graph contains cycles.
    pub fn get(&self, index: &I) -> Result<K, I> {
        // Fast path: check if already computed
        if let Some(value) = self.backend.borrow().get(index) {
            return Ok(value.clone());
        }

        // Get dependencies and resolve them recursively (no borrow held)
        let deps = self.problem.deps(index);
        let dep_values: Result<Vec<K>, I> = deps.into_iter().map(|dep| self.get(&dep)).collect();
        let dep_values = dep_values?;

        // Store the value, computing inside the closure only if not already cached
        Ok(self.backend
            .borrow_mut()
            .get_or_insert(index.clone(), || self.problem.compute(index, dep_values))?
            .clone())
    }
}

// =============================================================================
// Builder for DpCache
// =============================================================================

/// Builder for constructing a `DpCache`.
///
/// Supports const construction when backend and problem are set via const methods.
///
/// # Example (runtime)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, VecBackend};
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
/// let cache = DpCache::builder()
///     .backend(VecBackend::new())
///     .problem(Fibonacci)
///     .build();
/// ```
///
/// # Example (const)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpCache, DpCacheBuilder, DpProblem, ArrayBackend};
///
/// struct Factorial;
/// impl DpProblem<usize, u64> for Factorial {
///     fn deps(&self, n: &usize) -> Vec<usize> {
///         if *n == 0 { vec![] } else { vec![n - 1] }
///     }
///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
///         if *n == 0 { 1 } else { (*n as u64) * deps[0] }
///     }
/// }
///
/// const CACHE: DpCache<usize, u64, ArrayBackend<u64, 21>, Factorial> =
///     DpCacheBuilder::new()
///         .with_backend(ArrayBackend::new())
///         .with_problem(Factorial)
///         .build();
/// ```
pub struct DpCacheBuilder<I, K, B, P> {
    backend: Option<B>,
    problem: Option<P>,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, P> DpCacheBuilder<I, K, B, P> {
    fn new() -> Self {
        Self {
            backend: None,
            problem: None,
            _phantom: PhantomData,
        }
    }
}

impl<I, K, B, P> DpCacheBuilder<I, K, B, P>
where
    I: Clone,
    K: Clone,
    B: Backend<I, K>,
    P: DpProblem<I, K>,
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

    /// Builds the DpCache.
    ///
    /// # Panics
    ///
    /// Panics if backend or problem is not set.
    pub fn build(self) -> DpCache<I, K, B, P> {
        DpCache {
            backend: RefCell::new(self.backend.expect("backend is required")),
            problem: self.problem.expect("problem is required"),
            _phantom: PhantomData,
        }
    }
}

impl<I, K, B, P> DpCache<I, K, B, P>
where
    B: Backend<I, K>,
    P: DpProblem<I, K>,
{
    /// Creates a new `DpCache` directly (const-compatible).
    ///
    /// Use this when you need to construct a `DpCache` at compile time.
    /// Both the backend and problem must support const construction.
    ///
    /// # Example
    ///
    /// ```rust
    /// use aoc_solutions::utils::dp_cache::{DpCache, DpProblem, ArrayBackend};
    ///
    /// struct Factorial;
    /// impl DpProblem<usize, u64> for Factorial {
    ///     fn deps(&self, n: &usize) -> Vec<usize> {
    ///         if *n == 0 { vec![] } else { vec![n - 1] }
    ///     }
    ///     fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
    ///         if *n == 0 { 1 } else { (*n as u64) * deps[0] }
    ///     }
    /// }
    ///
    /// const CACHE: DpCache<usize, u64, ArrayBackend<u64, 21>, Factorial> =
    ///     DpCache::new_const(ArrayBackend::new(), Factorial);
    /// ```
    pub const fn new_const(backend: B, problem: P) -> Self {
        Self {
            backend: RefCell::new(backend),
            problem,
            _phantom: PhantomData,
        }
    }
}


