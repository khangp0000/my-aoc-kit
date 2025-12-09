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
/// let cache = DpCache::with_problem(VecBackend::new(), Factorial);
/// assert_eq!(cache.get(&5), 120);
/// ```
///
/// # Example (closure-based)
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{DpCache, VecBackend};
///
/// let cache = DpCache::new(
///     VecBackend::new(),
///     |n: &usize| if *n == 0 { vec![] } else { vec![n - 1] },
///     |n: &usize, deps: Vec<u64>| {
///         if *n == 0 { 1 } else { (*n as u64) * deps[0] }
///     },
/// );
///
/// assert_eq!(cache.get(&5), 120);
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
    /// Creates a new DpCache with the given backend and problem definition.
    ///
    /// # Arguments
    ///
    /// - `backend`: The storage backend for cached values
    /// - `problem`: A struct implementing `DpProblem` that defines deps and compute
    pub fn with_problem(backend: B, problem: P) -> Self {
        Self {
            backend: RefCell::new(backend),
            problem,
            _phantom: PhantomData,
        }
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
    /// The computed or cached value for the index.
    ///
    /// # Panics
    ///
    /// May panic or cause undefined behavior if the dependency graph contains cycles.
    pub fn get(&self, index: &I) -> K {
        // Fast path: check if already computed
        if let Some(value) = self.backend.borrow().get(index) {
            return value.clone();
        }

        // Get dependencies and resolve them recursively (no borrow held)
        let deps = self.problem.deps(index);
        let dep_values: Vec<K> = deps.into_iter().map(|dep| self.get(&dep)).collect();

        // Store the value, computing inside the closure only if not already cached
        self.backend
            .borrow_mut()
            .get_or_insert(index.clone(), || self.problem.compute(index, dep_values))
            .clone()
    }
}

/// Wrapper to adapt closure functions to the DpProblem trait.
pub struct ClosureProblem<I, K, D, C>
where
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    dep_fn: D,
    compute_fn: C,
    _phantom: PhantomData<(I, K)>,
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

impl<I, K, B> DpCache<I, K, B, ClosureProblem<I, K, fn(&I) -> Vec<I>, fn(&I, Vec<K>) -> K>>
where
    I: Clone,
    K: Clone,
    B: Backend<I, K>,
{
    /// Creates a new DpCache with the given backend, dependency function, and compute function.
    ///
    /// This is a convenience constructor for the closure-based API.
    ///
    /// # Arguments
    ///
    /// - `backend`: The storage backend for cached values
    /// - `dep_fn`: A function that returns the indices this index depends on
    /// - `compute_fn`: A function that computes the value given the index and resolved dependency values
    pub fn new<D, C>(
        backend: B,
        dep_fn: D,
        compute_fn: C,
    ) -> DpCache<I, K, B, ClosureProblem<I, K, D, C>>
    where
        D: Fn(&I) -> Vec<I>,
        C: Fn(&I, Vec<K>) -> K,
    {
        DpCache {
            backend: RefCell::new(backend),
            problem: ClosureProblem {
                dep_fn,
                compute_fn,
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
        }
    }
}
