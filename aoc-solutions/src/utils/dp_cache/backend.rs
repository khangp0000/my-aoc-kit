//! Storage backends for the DP cache.

use std::collections::HashMap;
use std::hash::Hash;

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
/// - The compute function passed to `get_or_insert` should not require `&self` reference
///   to the cache (dependencies should already be resolved)
pub trait Backend<I, K> {
    /// Returns a reference to the cached value for the given index, if it exists.
    fn get(&self, index: &I) -> Option<&K>;

    /// Returns the cached value, or computes and stores it using the provided function.
    ///
    /// The compute function receives no arguments - all dependencies should be
    /// captured in the closure before calling this method.
    fn get_or_insert<F>(&mut self, index: I, compute: F) -> &K
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

    fn get_or_insert<F>(&mut self, index: usize, compute: F) -> &K
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

        self.data[index].as_ref().unwrap()
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

    fn get_or_insert<F>(&mut self, index: I, compute: F) -> &K
    where
        F: FnOnce() -> K,
    {
        self.data.entry(index).or_insert_with(compute)
    }
}
