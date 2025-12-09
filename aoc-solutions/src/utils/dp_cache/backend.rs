//! Storage backends for the DP cache.

use std::cell::OnceCell;
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
/// - `ensure_index` must be called before `get` for any index
/// - `ensure_index` is idempotent: calling it on an existing index preserves the value
/// - `get` panics if called on an index that was never ensured
pub trait Backend<I, K> {
    /// Returns a reference to the OnceCell for the given index.
    ///
    /// # Panics
    ///
    /// Panics if the index has not been ensured via `ensure_index`.
    fn get(&self, index: &I) -> &OnceCell<K>;

    /// Ensures an entry exists for the given index.
    ///
    /// If the index is new, creates an empty `OnceCell`.
    /// If the index already exists, leaves the existing entry unchanged.
    fn ensure_index(&mut self, index: I);
}


/// A Vec-based backend for usize indices.
///
/// This backend is efficient for dense, sequential integer indices starting from 0.
/// The Vec automatically grows to accommodate new indices.
#[derive(Debug)]
pub struct VecBackend<K> {
    data: Vec<OnceCell<K>>,
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
    fn get(&self, index: &usize) -> &OnceCell<K> {
        &self.data[*index]
    }

    fn ensure_index(&mut self, index: usize) {
        if index >= self.data.len() {
            self.data.resize_with(index + 1, OnceCell::new);
        }
    }
}


/// A HashMap-based backend for arbitrary hashable indices.
///
/// This backend supports any index type that implements `Hash + Eq`.
/// It is suitable for sparse indices or non-integer index types.
#[derive(Debug)]
pub struct HashMapBackend<I, K> {
    data: HashMap<I, OnceCell<K>>,
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
    fn get(&self, index: &I) -> &OnceCell<K> {
        self.data.get(index).expect("index not ensured")
    }

    fn ensure_index(&mut self, index: I) {
        self.data.entry(index).or_insert_with(OnceCell::new);
    }
}
