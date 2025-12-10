//! Storage backends for the DP cache.

use std::cell::OnceCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{OnceLock, RwLock};

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
/// Uses `OnceCell` for each element to ensure exactly-once computation.
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
    fn get(&self, index: &usize) -> Option<&K> {
        self.data.get(*index).and_then(|cell| cell.get())
    }

    fn get_or_insert<F>(&mut self, index: usize, compute: F) -> Result<&K, usize>
    where
        F: FnOnce() -> K,
    {
        // Ensure the vec is large enough
        if index >= self.data.len() {
            self.data.resize_with(index + 1, OnceCell::new);
        }

        // Use get_or_init for exactly-once computation
        Ok(self.data[index].get_or_init(compute))
    }
}

// =============================================================================
// Fixed-Size Array Backends
// =============================================================================

/// A 1D fixed-size array backend using const generics.
///
/// This backend provides zero-allocation caching for problems with known,
/// bounded index spaces. The size N is specified at compile time.
/// Uses `OnceCell` for each element to ensure exactly-once computation.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ArrayBackend, Backend};
///
/// let mut backend: ArrayBackend<i32, 10> = ArrayBackend::new();
/// let value = backend.get_or_insert(5, || 42).unwrap();
/// assert_eq!(*value, 42);
/// ```
pub struct ArrayBackend<K, const N: usize> {
    data: [OnceCell<K>; N],
}

impl<K, const N: usize> ArrayBackend<K, N> {
    /// Creates a new ArrayBackend with all elements uninitialized.
    /// This is a const fn, usable in const/static contexts.
    pub const fn new() -> Self {
        Self {
            data: [const { OnceCell::new() }; N],
        }
    }
}

impl<K, const N: usize> Default for ArrayBackend<K, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::fmt::Debug, const N: usize> std::fmt::Debug for ArrayBackend<K, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayBackend")
            .field("size", &N)
            .field("data", &self.data)
            .finish()
    }
}

impl<K, const N: usize> Backend<usize, K> for ArrayBackend<K, N> {
    fn get(&self, index: &usize) -> Option<&K> {
        if *index >= N {
            return None;
        }
        self.data[*index].get()
    }

    fn get_or_insert<F>(&mut self, index: usize, compute: F) -> Result<&K, usize>
    where
        F: FnOnce() -> K,
    {
        if index >= N {
            return Err(index);
        }
        Ok(self.data[index].get_or_init(compute))
    }
}

/// A 2D fixed-size array backend using const generics.
///
/// This backend provides zero-allocation caching for 2D grid-based DP problems
/// with known bounds. Both dimensions (ROWS and COLS) are specified at compile time.
/// Uses `OnceCell` for each element to ensure exactly-once computation.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{Array2DBackend, Backend};
///
/// let mut backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
/// let value = backend.get_or_insert((2, 3), || 42).unwrap();
/// assert_eq!(*value, 42);
/// ```
pub struct Array2DBackend<K, const ROWS: usize, const COLS: usize> {
    data: [[OnceCell<K>; COLS]; ROWS],
}

impl<K, const ROWS: usize, const COLS: usize> Array2DBackend<K, ROWS, COLS> {
    /// Creates a new Array2DBackend with all elements uninitialized.
    /// This is a const fn, usable in const/static contexts.
    pub const fn new() -> Self {
        // Use nested inline const to initialize 2D array
        Self {
            data: [const { [const { OnceCell::new() }; COLS] }; ROWS],
        }
    }
}

impl<K, const ROWS: usize, const COLS: usize> Default for Array2DBackend<K, ROWS, COLS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::fmt::Debug, const ROWS: usize, const COLS: usize> std::fmt::Debug
    for Array2DBackend<K, ROWS, COLS>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Array2DBackend")
            .field("rows", &ROWS)
            .field("cols", &COLS)
            .field("data", &self.data)
            .finish()
    }
}

impl<K, const ROWS: usize, const COLS: usize> Backend<(usize, usize), K>
    for Array2DBackend<K, ROWS, COLS>
{
    fn get(&self, index: &(usize, usize)) -> Option<&K> {
        let (row, col) = *index;
        if row >= ROWS || col >= COLS {
            return None;
        }
        self.data[row][col].get()
    }

    fn get_or_insert<F>(&mut self, index: (usize, usize), compute: F) -> Result<&K, (usize, usize)>
    where
        F: FnOnce() -> K,
    {
        let (row, col) = index;
        if row >= ROWS || col >= COLS {
            return Err((row, col));
        }
        Ok(self.data[row][col].get_or_init(compute))
    }
}

/// A 2D Vec-based backend for runtime-sized dimensions with auto-grow.
///
/// This backend provides caching for 2D grid-based DP problems where
/// dimensions are only known at runtime. Uses `OnceCell` for each element
/// to ensure exactly-once computation. The Vec automatically grows to
/// accommodate new indices, similar to `VecBackend`.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{Vec2DBackend, Backend};
///
/// // Create with auto-grow (no initial size)
/// let mut backend: Vec2DBackend<i32> = Vec2DBackend::new();
/// let value = backend.get_or_insert((2, 3), || 42).unwrap();
/// assert_eq!(*value, 42);
///
/// // Create with capacity hint for better performance
/// let mut backend: Vec2DBackend<i32> = Vec2DBackend::with_capacity(100, 50);
/// ```
#[derive(Debug)]
pub struct Vec2DBackend<K> {
    data: Vec<Vec<OnceCell<K>>>,
    /// Capacity hint for columns (used when creating new rows)
    col_capacity: usize,
}

impl<K> Vec2DBackend<K> {
    /// Creates a new empty Vec2DBackend with auto-grow capability.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            col_capacity: 0,
        }
    }

    /// Creates a new Vec2DBackend with the specified capacity hints.
    ///
    /// The outer Vec is created with capacity for `row_capacity` rows.
    /// When new rows are created, inner Vecs are created with capacity
    /// for `col_capacity` columns.
    pub fn with_capacity(row_capacity: usize, col_capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(row_capacity),
            col_capacity,
        }
    }

    /// Returns the current number of rows.
    pub fn rows(&self) -> usize {
        self.data.len()
    }

    /// Returns the number of columns in the specified row, or 0 if row doesn't exist.
    pub fn cols(&self, row: usize) -> usize {
        self.data.get(row).map(|r| r.len()).unwrap_or(0)
    }

    /// Ensures the backend has at least the specified dimensions.
    fn ensure_capacity(&mut self, row: usize, col: usize) {
        // Grow rows if needed
        if row >= self.data.len() {
            self.data.resize_with(row + 1, || {
                if self.col_capacity > 0 {
                    Vec::with_capacity(self.col_capacity)
                } else {
                    Vec::new()
                }
            });
        }

        // Grow columns in the target row if needed
        if col >= self.data[row].len() {
            self.data[row].resize_with(col + 1, OnceCell::new);
        }
    }
}

impl<K> Default for Vec2DBackend<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> Backend<(usize, usize), K> for Vec2DBackend<K> {
    fn get(&self, index: &(usize, usize)) -> Option<&K> {
        let (row, col) = *index;
        self.data.get(row).and_then(|r| r.get(col)).and_then(|cell| cell.get())
    }

    fn get_or_insert<F>(&mut self, index: (usize, usize), compute: F) -> Result<&K, (usize, usize)>
    where
        F: FnOnce() -> K,
    {
        let (row, col) = index;
        
        // Auto-grow to accommodate the index
        self.ensure_capacity(row, col);
        
        // Use get_or_init for exactly-once computation
        Ok(self.data[row][col].get_or_init(compute))
    }
}

/// A no-op backend that never caches values.
///
/// This backend always recomputes values on every `get_or_insert` call and
/// always returns `None` for `get`. Useful for benchmarking to isolate
/// the overhead of the DpCache wrapper from actual caching mechanisms.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{NoCacheBackend, Backend};
///
/// let mut backend: NoCacheBackend<usize, i32> = NoCacheBackend::new();
/// // Always recomputes - no caching
/// let value = backend.get_or_insert(5, || 42).unwrap();
/// assert_eq!(*value, 42);
/// // get always returns None
/// assert!(backend.get(&5).is_none());
/// ```
#[derive(Debug, Default)]
pub struct NoCacheBackend<I, K> {
    /// Temporary storage for the last computed value (to return a reference)
    last_value: Option<K>,
    _phantom: std::marker::PhantomData<(I, K)>,
}

impl<I, K> NoCacheBackend<I, K> {
    /// Creates a new NoCacheBackend.
    pub fn new() -> Self {
        Self {
            last_value: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I, K> Backend<I, K> for NoCacheBackend<I, K> {
    fn get(&self, _index: &I) -> Option<&K> {
        // Never cached - always return None
        None
    }

    fn get_or_insert<F>(&mut self, _index: I, compute: F) -> Result<&K, I>
    where
        F: FnOnce() -> K,
    {
        // Always recompute - never cache
        self.last_value = Some(compute());
        Ok(self.last_value.as_ref().unwrap())
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

/// A thread-safe no-op backend that never caches values.
///
/// This backend always recomputes values on every `get_or_insert` call and
/// always returns `None` for `get`. Useful for benchmarking to isolate
/// the overhead of the ParallelDpCache wrapper from actual caching mechanisms.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelNoCacheBackend, ParallelBackend};
///
/// let backend: ParallelNoCacheBackend<usize, i32> = ParallelNoCacheBackend::new();
/// // Always recomputes - no caching
/// let value = backend.get_or_insert(5, || 42).unwrap();
/// assert_eq!(value, 42);
/// // get always returns None
/// assert!(backend.get(&5).is_none());
/// ```
#[derive(Debug, Default)]
pub struct ParallelNoCacheBackend<I, K> {
    _phantom: std::marker::PhantomData<(I, K)>,
}

impl<I, K> ParallelNoCacheBackend<I, K> {
    /// Creates a new ParallelNoCacheBackend.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<I, K> ParallelBackend<I, K> for ParallelNoCacheBackend<I, K>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
{
    fn get(&self, _index: &I) -> Option<K> {
        // Never cached - always return None
        None
    }

    fn get_or_insert<F>(&self, _index: I, compute: F) -> Result<K, I>
    where
        F: FnOnce() -> K,
    {
        // Always recompute - never cache
        Ok(compute())
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

// =============================================================================
// Parallel Fixed-Size Array Backends
// =============================================================================

/// A thread-safe 1D fixed-size array backend using const generics.
///
/// This backend provides thread-safe caching for problems with known,
/// bounded index spaces. Uses `OnceLock` for each element to ensure
/// exactly-once computation with lock-free reads after initialization.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelArrayBackend, ParallelBackend};
///
/// let backend: ParallelArrayBackend<i32, 10> = ParallelArrayBackend::new();
/// let value = backend.get_or_insert(5, || 42).unwrap();
/// assert_eq!(value, 42);
/// ```
pub struct ParallelArrayBackend<K, const N: usize> {
    data: [OnceLock<K>; N],
}

impl<K, const N: usize> ParallelArrayBackend<K, N> {
    /// Creates a new ParallelArrayBackend with all elements uninitialized.
    /// This is a const fn, usable in const/static contexts.
    pub const fn new() -> Self {
        Self {
            data: [const { OnceLock::new() }; N],
        }
    }
}

impl<K, const N: usize> Default for ParallelArrayBackend<K, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::fmt::Debug, const N: usize> std::fmt::Debug for ParallelArrayBackend<K, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelArrayBackend")
            .field("size", &N)
            .field("data", &self.data)
            .finish()
    }
}

impl<K, const N: usize> ParallelBackend<usize, K> for ParallelArrayBackend<K, N>
where
    K: Clone + Send + Sync,
{
    fn get(&self, index: &usize) -> Option<K> {
        if *index >= N {
            return None;
        }
        self.data[*index].get().cloned()
    }

    fn get_or_insert<F>(&self, index: usize, compute: F) -> Result<K, usize>
    where
        F: FnOnce() -> K,
    {
        if index >= N {
            return Err(index);
        }
        Ok(self.data[index].get_or_init(compute).clone())
    }
}

/// A thread-safe 2D fixed-size array backend using const generics.
///
/// This backend provides thread-safe caching for 2D grid-based DP problems
/// with known bounds. Uses `OnceLock` for each element to ensure
/// exactly-once computation with lock-free reads after initialization.
///
/// # Example
///
/// ```rust
/// use aoc_solutions::utils::dp_cache::{ParallelArray2DBackend, ParallelBackend};
///
/// let backend: ParallelArray2DBackend<i32, 5, 10> = ParallelArray2DBackend::new();
/// let value = backend.get_or_insert((2, 3), || 42).unwrap();
/// assert_eq!(value, 42);
/// ```
pub struct ParallelArray2DBackend<K, const ROWS: usize, const COLS: usize> {
    data: [[OnceLock<K>; COLS]; ROWS],
}

impl<K, const ROWS: usize, const COLS: usize> ParallelArray2DBackend<K, ROWS, COLS> {
    /// Creates a new ParallelArray2DBackend with all elements uninitialized.
    /// This is a const fn, usable in const/static contexts.
    pub const fn new() -> Self {
        Self {
            data: [const { [const { OnceLock::new() }; COLS] }; ROWS],
        }
    }
}

impl<K, const ROWS: usize, const COLS: usize> Default for ParallelArray2DBackend<K, ROWS, COLS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: std::fmt::Debug, const ROWS: usize, const COLS: usize> std::fmt::Debug
    for ParallelArray2DBackend<K, ROWS, COLS>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelArray2DBackend")
            .field("rows", &ROWS)
            .field("cols", &COLS)
            .field("data", &self.data)
            .finish()
    }
}

impl<K, const ROWS: usize, const COLS: usize> ParallelBackend<(usize, usize), K>
    for ParallelArray2DBackend<K, ROWS, COLS>
where
    K: Clone + Send + Sync,
{
    fn get(&self, index: &(usize, usize)) -> Option<K> {
        let (row, col) = *index;
        if row >= ROWS || col >= COLS {
            return None;
        }
        self.data[row][col].get().cloned()
    }

    fn get_or_insert<F>(&self, index: (usize, usize), compute: F) -> Result<K, (usize, usize)>
    where
        F: FnOnce() -> K,
    {
        let (row, col) = index;
        if row >= ROWS || col >= COLS {
            return Err((row, col));
        }
        Ok(self.data[row][col].get_or_init(compute).clone())
    }
}
