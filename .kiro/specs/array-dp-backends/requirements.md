# Requirements Document

## Introduction

This feature extends the DP cache system with fixed-size array-based backends that leverage Rust's const generics for compile-time size specification. These backends provide zero-allocation caching for problems with known, bounded index spaces, offering better performance than dynamic Vec or HashMap backends for appropriate use cases. The feature includes 1D array, 2D array, and 2D Vec backends, with parallel versions for all array-based backends.

## Glossary

- **Backend**: A storage implementation that provides `get` and `get_or_insert` operations for the DP cache
- **Const Generic**: A Rust feature allowing compile-time constant values as type parameters (e.g., `[T; N]` where `N` is a const)
- **ArrayBackend**: A 1D fixed-size array backend using const generics for size
- **Array2DBackend**: A 2D fixed-size array backend using const generics for both dimensions
- **Vec2DBackend**: A 2D backend using Vec of Vecs for runtime-sized dimensions
- **ParallelBackend**: A thread-safe backend trait for concurrent DP cache access
- **Uninit Cache**: A cache constructed with uninitialized (None) values at compile time
- **DpCache**: The single-threaded dynamic programming cache
- **ParallelDpCache**: The thread-safe parallel dynamic programming cache

## Requirements

### Requirement 1

**User Story:** As a developer, I want a 1D array backend with const generic size, so that I can have zero-allocation caching for problems with known index bounds.

#### Acceptance Criteria

1. WHEN a user creates an ArrayBackend with const generic size N, THE ArrayBackend SHALL allocate a fixed-size array of N elements at construction time
2. WHEN a user calls get on an ArrayBackend with an index within bounds, THE ArrayBackend SHALL return the cached value if present or None if not computed
3. WHEN a user calls get_or_insert on an ArrayBackend with an index within bounds, THE ArrayBackend SHALL compute and store the value if not present, then return a reference
4. WHEN a user calls get or get_or_insert with an out-of-bounds index, THE ArrayBackend SHALL return an error containing the invalid index
5. WHEN a user creates an ArrayBackend using the const fn new(), THE ArrayBackend SHALL be usable in const/static contexts

### Requirement 2

**User Story:** As a developer, I want a 2D array backend with const generic dimensions, so that I can efficiently cache 2D grid-based DP problems with known bounds.

#### Acceptance Criteria

1. WHEN a user creates an Array2DBackend with const generic dimensions ROWS and COLS, THE Array2DBackend SHALL allocate a fixed-size 2D array at construction time
2. WHEN a user calls get on an Array2DBackend with a (row, col) index within bounds, THE Array2DBackend SHALL return the cached value if present or None if not computed
3. WHEN a user calls get_or_insert on an Array2DBackend with a (row, col) index within bounds, THE Array2DBackend SHALL compute and store the value if not present, then return a reference
4. WHEN a user calls get or get_or_insert with an out-of-bounds (row, col) index, THE Array2DBackend SHALL return an error containing the invalid index
5. WHEN a user creates an Array2DBackend using the const fn new(), THE Array2DBackend SHALL be usable in const/static contexts

### Requirement 3

**User Story:** As a developer, I want a 2D Vec backend for runtime-sized dimensions, so that I can cache 2D DP problems where dimensions are only known at runtime.

#### Acceptance Criteria

1. WHEN a user creates a Vec2DBackend with runtime dimensions (rows, cols), THE Vec2DBackend SHALL allocate a Vec of Vecs with the specified dimensions
2. WHEN a user calls get on a Vec2DBackend with a (row, col) index within bounds, THE Vec2DBackend SHALL return the cached value if present or None if not computed
3. WHEN a user calls get_or_insert on a Vec2DBackend with a (row, col) index within bounds, THE Vec2DBackend SHALL compute and store the value if not present, then return a reference
4. WHEN a user calls get or get_or_insert with an out-of-bounds (row, col) index, THE Vec2DBackend SHALL return an error containing the invalid index

### Requirement 4

**User Story:** As a developer, I want parallel versions of the array backends, so that I can use fixed-size caching in multi-threaded DP computations.

#### Acceptance Criteria

1. WHEN a user creates a ParallelArrayBackend with const generic size N, THE ParallelArrayBackend SHALL provide thread-safe access to a fixed-size array
2. WHEN a user creates a ParallelArray2DBackend with const generic dimensions, THE ParallelArray2DBackend SHALL provide thread-safe access to a fixed-size 2D array
3. WHEN multiple threads call get_or_insert concurrently on a parallel array backend, THE parallel array backend SHALL ensure each index is computed exactly once
4. WHEN a user calls get or get_or_insert with an out-of-bounds index on a parallel array backend, THE parallel array backend SHALL return an error containing the invalid index

### Requirement 5

**User Story:** As a developer, I want the DpCache and ParallelDpCache builders to support const backends, so that I can construct caches with compile-time initialized storage.

#### Acceptance Criteria

1. WHEN a user builds a DpCache with a const-constructible backend, THE DpCacheBuilder SHALL accept the backend and construct the cache
2. WHEN a user builds a ParallelDpCache with a const-constructible parallel backend, THE ParallelDpCacheBuilder SHALL accept the backend and construct the cache
3. WHEN a user creates a cache with a const-constructed backend, THE cache SHALL function correctly with the backend's const fn new()
