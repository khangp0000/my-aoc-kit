//! Storage backends for the DP cache.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;

use dashmap::DashMap;

/// A storage backend for the DP cache.
///
/// This trait defines the interface for storing and retrieving cached values.
/// Implementations can use different data structures (Vec, HashMap, etc.)
/// based on the index type requirements.
///
/// # Contract
///
/// - `get` returns `None` if the index has not been computed yet
/// - `get_or_insert` computes and stores the value if not present, then returns a reference
/// - Returns `Err(index)` if the index cannot be stored (e.g., out of bounds for fixed-size backends)
/// - The compute function passed to `get_or_insert` should not require `&self` reference
///   to the cache (dependencies should already be resolved)
pub trait Backend<I, K> {
    /// Returns a reference to the cached value for the given index, if it exists.
    fn get(&self, index: &I) -> Option<&K>;

    /// Returns the cached value, or computes and stores it using the provided function.
    ///
    /// Returns `Err(index)` if the index cannot be stored (e.g., out of bounds).
    ///
    /// The compute function receives no arguments - all dependencies should be
    /// captured in the closure before calling this method.
    fn get_or_insert<F>(&mut self, index: I, compute: F) -> Result<&K, I>
    where
        F: FnOnce() -> K;
}

/// A thread-safe storage backend for parallel DP cache.
///
/// This trait defines the interface for concurrent storage and retrieval.
/// Unlike `Backend`, methods take `&self` and return owned values to avoid
/// lifetime issues with concurrent access.
///
/// # Contract
///
/// - `get` returns `None` if the index has not been computed yet
/// - `get_or_insert` computes and stores the value if not present, then returns a clone
/// - Implementations must be thread-safe
pub trait ParallelBackend<I, K>: Send + Sync {
    /// Returns a clone of the cached value for the given index, if it exists.
    fn get(&self, index: &I) -> Option<K>;

    /// Returns the cached value (cloned), or computes and stores it using the provided function.
    ///
    /// The compute function receives no arguments - all dependencies should be
    /// captured in the closure before calling this method.
    fn get_or_insert<F>(&self, index: I, compute: F) -> Result<K, I>
    where
        F: FnOnce() -> K;
}

/// A Vec-based backend for usize indices.
///
/// This backend is efficient for dense, sequential integer indices starting from 0.
/// The Vec automatically grows to accommodate new indices.
#[derive(Debug)]
pub struct VecBackend<K> {
    data: Vec<Option<K>>,
}

impl<K> VecBackend<K> {
    /// Creates a new empty VecBackend.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a new VecBackend with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
}

impl<K> Default for VecBackend<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Backend<usize, K> for VecBackend<K> {
    fn get(&self, index: &usize) -> Option<&K> {
        self.data.get(*index).and_then(|opt| opt.as_ref())
    }

    fn get_or_insert<F>(&mut self, index: usize, compute: F) -> Result<&K, usize>
    where
        F: FnOnce() -> K,
    {
        // Ensure the vec is large enough
        if index >= self.data.len() {
            self.data.resize_with(index + 1, || None);
        }

        // Compute if not present
        if self.data[index].is_none() {
            self.data[index] = Some(compute());
        }

        Ok(self.data[index].as_ref().unwrap())
    }
}

/// A HashMap-based backend for arbitrary hashable indices.
///
/// This backend supports any index type that implements `Hash + Eq`.
/// It is suitable for sparse indices or non-integer index types.
#[derive(Debug)]
pub struct HashMapBackend<I, K> {
    data: HashMap<I, K>,
}

impl<I, K> HashMapBackend<I, K> {
    /// Creates a new empty HashMapBackend.
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<I, K> Default for HashMapBackend<I, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Hash + Eq, K> Backend<I, K> for HashMapBackend<I, K> {
    fn get(&self, index: &I) -> Option<&K> {
        self.data.get(index)
    }

    fn get_or_insert<F>(&mut self, index: I, compute: F) -> Result<&K, I>
    where
        F: FnOnce() -> K,
    {
        Ok(self.data.entry(index).or_insert_with(compute))
    }
}

// =============================================================================
// Parallel Backends
// =============================================================================

/// A DashMap-based backend for parallel DP cache.
///
/// This backend provides lock-free concurrent access using DashMap's
/// sharded locking strategy. It's efficient for high-contention scenarios.
#[derive(Debug)]
pub struct DashMapBackend<I, K>
where
    I: Hash + Eq,
{
    data: DashMap<I, K>,
}

impl<I, K> DashMapBackend<I, K>
where
    I: Hash + Eq,
{
    /// Creates a new empty DashMapBackend.
    pub fn new() -> Self {
        Self {
            data: DashMap::new(),
        }
    }
}

impl<I, K> Default for DashMapBackend<I, K>
where
    I: Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I, K> ParallelBackend<I, K> for DashMapBackend<I, K>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
{
    fn get(&self, index: &I) -> Option<K> {
        self.data.get(index).map(|entry| entry.value().clone())
    }

    fn get_or_insert<F>(&self, index: I, compute: F) -> Result<K, I>
    where
        F: FnOnce() -> K,
    {
        Ok(self.data
            .entry(index)
            .or_insert_with(compute)
            .value()
            .clone())
    }
}

/// A RwLock<HashMap>-based backend for parallel DP cache.
///
/// This backend uses a single RwLock around a HashMap. It's simpler than
/// DashMap but may have higher contention under heavy concurrent access.
/// Good for scenarios with more reads than writes.
pub struct RwLockHashMapBackend<I, K> {
    data: RwLock<HashMap<I, K>>,
}

impl<I, K> RwLockHashMapBackend<I, K> {
    /// Creates a new empty RwLockHashMapBackend.
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

impl<I, K> Default for RwLockHashMapBackend<I, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I, K> std::fmt::Debug for RwLockHashMapBackend<I, K>
where
    I: std::fmt::Debug,
    K: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.data.read() {
            Ok(guard) => f.debug_struct("RwLockHashMapBackend").field("data", &*guard).finish(),
            Err(_) => f.debug_struct("RwLockHashMapBackend").field("data", &"<locked>").finish(),
        }
    }
}

impl<I, K> ParallelBackend<I, K> for RwLockHashMapBackend<I, K>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
{
    fn get(&self, index: &I) -> Option<K> {
        self.data
            .read()
            .expect("RwLock poisoned")
            .get(index)
            .cloned()
    }

    fn get_or_insert<F>(&self, index: I, compute: F) -> Result<K, I>
    where
        F: FnOnce() -> K,
    {
        // Fast path: check with read lock
        {
            let read_guard = self.data.read().expect("RwLock poisoned");
            if let Some(value) = read_guard.get(&index) {
                return Ok(value.clone());
            }
        }

        // Slow path: acquire write lock and insert
        let mut write_guard = self.data.write().expect("RwLock poisoned");
        // Double-check after acquiring write lock (another thread may have inserted)
        Ok(write_guard.entry(index).or_insert_with(compute).clone())
    }
}
