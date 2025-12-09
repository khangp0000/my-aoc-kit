# Requirements Document

## Introduction

This document specifies requirements for a dynamic programming cache struct that provides memoization with lazy evaluation and dependency resolution. The cache is backed by a pluggable backend trait, uses interior mutability via `RefCell`, and leverages `OnceCell` for thread-safe single initialization of cached values. The module will be located in `aoc-solutions/src/utils/dp_cache.rs`.

This document also specifies requirements for parallel versions of the DP cache using thread-safe backends (`Mutex<Vec>` and `DashMap`) with Rayon for parallel dependency resolution.

## Glossary

- **DpCache**: The main cache struct that provides memoized access to computed values (single-threaded)
- **ParDpCache**: The parallel cache struct that provides thread-safe memoized access using Rayon
- **Backend**: A trait defining storage operations for the single-threaded cache
- **ParBackend**: A trait defining thread-safe storage operations for the parallel cache
- **Index (I)**: The type used to identify cache entries
- **Value (K)**: The type of computed/cached values
- **Dependency Function**: A function `I -> Vec<I>` that returns indices this index depends on
- **Compute Function**: A function `(&I, Vec<K>) -> K` that computes the value for an index
- **OnceCell**: A cell that can be written to only once, providing lazy initialization (single-threaded)
- **OnceLock**: A thread-safe cell that can be written to only once (parallel version)
- **DAG**: Directed Acyclic Graph - the expected structure of dependencies
- **DashMap**: A concurrent HashMap implementation for lock-free concurrent access
- **Rayon**: A data parallelism library for Rust providing parallel iterators
- **ThreadPool**: A Rayon thread pool that can be optionally provided for parallel execution

## Requirements

### Requirement 1

**User Story:** As a developer, I want to create a DP cache with custom dependency and compute functions, so that I can implement memoized recursive algorithms.

#### Acceptance Criteria

1. WHEN a user creates a DpCache THEN the DpCache SHALL accept a dependency function `Fn(&I) -> Vec<I>` and a compute function `Fn(&I, Vec<K>) -> K`
2. WHEN a user creates a DpCache THEN the DpCache SHALL accept a backend implementing the Backend trait
3. WHEN a DpCache is created THEN the DpCache SHALL store the backend in a `RefCell` for interior mutability

### Requirement 2

**User Story:** As a developer, I want to retrieve cached values by index, so that I can access computed results without redundant computation.

#### Acceptance Criteria

1. WHEN a user calls `get(&self, i: I)` on DpCache THEN the DpCache SHALL return a cloned value of type K
2. WHEN `get` is called for an index that exists and is initialized THEN the DpCache SHALL return the cached value without recomputation
3. WHEN `get` is called for an index that does not exist in backend THEN the DpCache SHALL call `ensure_index` on the backend to create the entry
4. WHEN `get` is called for an uninitialized index THEN the DpCache SHALL first resolve all dependencies by calling `get` recursively
5. WHEN all dependencies are resolved THEN the DpCache SHALL initialize the value by calling the compute function with the index and resolved dependency values
6. WHEN the value is initialized THEN the DpCache SHALL clone and return the result

### Requirement 3

**User Story:** As a developer, I want to implement custom storage backends, so that I can use different data structures (Vec, HashMap) based on my index type.

#### Acceptance Criteria

1. WHEN implementing the Backend trait THEN the implementor SHALL provide `get(&self, i: &I) -> &OnceCell<K>` to retrieve a possibly uninitialized cell
2. WHEN implementing the Backend trait THEN the implementor SHALL provide `ensure_index(&mut self, i: I)` to ensure an entry exists for the index
3. WHEN `ensure_index` is called for a new index THEN the Backend SHALL create an empty `OnceCell` for that index
4. WHEN `ensure_index` is called for an existing index THEN the Backend SHALL leave the existing entry unchanged

### Requirement 4

**User Story:** As a developer, I want the cache to handle the borrow lifecycle correctly, so that I can use it without runtime borrow panics.

#### Acceptance Criteria

1. WHEN checking if an index exists THEN the DpCache SHALL use a shared borrow of the backend
2. WHEN calling `ensure_index` THEN the DpCache SHALL use a mutable borrow of the backend and drop it before proceeding
3. WHEN resolving dependencies recursively THEN the DpCache SHALL not hold any backend borrow across recursive calls
4. WHEN initializing via `OnceCell::get_or_init` THEN the DpCache SHALL hold only a shared borrow of the backend

### Requirement 5

**User Story:** As a developer, I want clear documentation about cycle behavior, so that I understand the limitations of the cache.

#### Acceptance Criteria

1. WHEN the dependency graph contains cycles THEN the behavior SHALL be undefined (stack overflow or infinite loop)
2. WHEN documenting the DpCache THEN the documentation SHALL explicitly state that cycle detection is not supported
3. WHEN documenting the DpCache THEN the documentation SHALL state that users must ensure dependencies form a DAG

### Requirement 6

**User Story:** As a developer, I want a Vec-based backend implementation, so that I can use integer indices efficiently.

#### Acceptance Criteria

1. WHEN using VecBackend with index type `usize` THEN the VecBackend SHALL store entries in a `Vec<OnceCell<K>>`
2. WHEN `ensure_index` is called with index `i` THEN the VecBackend SHALL extend the Vec if `i >= len`
3. WHEN `get` is called THEN the VecBackend SHALL return a reference to the OnceCell at that index

### Requirement 7

**User Story:** As a developer, I want a HashMap-based backend implementation, so that I can use arbitrary hashable index types.

#### Acceptance Criteria

1. WHEN using HashMapBackend with index type `I: Hash + Eq` THEN the HashMapBackend SHALL store entries in a `HashMap<I, OnceCell<K>>`
2. WHEN `ensure_index` is called THEN the HashMapBackend SHALL insert an empty OnceCell if the key does not exist
3. WHEN `get` is called THEN the HashMapBackend SHALL return a reference to the OnceCell for that key
