# Requirements Document

## Introduction

This document specifies requirements for extending the existing dynamic programming cache (`DpCache`) with parallel execution support. The extension adds `DashMapDpCache`, a thread-safe cache that uses DashMap for storage and Rayon for concurrent dependency resolution. A benchmark comparing sequential `DpCache` vs parallel `DashMapDpCache` using the Collatz chain length problem validates the implementation.

## Glossary

- **DpCache**: The existing single-threaded memoization cache with pluggable backends (VecBackend, HashMapBackend)
- **DashMapDpCache**: A thread-safe cache using DashMap for storage and Rayon for parallel dependency resolution
- **DashMap**: A concurrent HashMap implementation providing lock-free reads and sharded writes
- **Collatz Sequence**: The sequence where n → n/2 (if even) or n → 3n+1 (if odd), terminating at 1
- **Collatz Chain Length**: The number of steps to reach 1 from a starting number
- **Rayon**: A data parallelism library for Rust
- **ThreadPool**: A Rayon thread pool that can be optionally provided for parallel execution

## Requirements

### Requirement 1

**User Story:** As a developer, I want a parallel DP cache using DashMap, so that I can speed up computation on multi-core systems with concurrent access.

#### Acceptance Criteria

1. WHEN creating a DashMapDpCache THEN the DashMapDpCache SHALL use `DashMap<I, K>` for direct value storage
2. WHEN creating a DashMapDpCache THEN the DashMapDpCache SHALL accept a dependency function and compute function
3. WHEN creating a DashMapDpCache THEN the DashMapDpCache SHALL optionally accept a Rayon ThreadPool via `with_pool`
4. WHEN `get` is called for an uncomputed index THEN the DashMapDpCache SHALL resolve dependencies in parallel using Rayon's `par_iter`
5. WHEN `get` is called for an already-computed index THEN the DashMapDpCache SHALL return the cached value without recomputation
6. WHEN multiple threads access different keys concurrently THEN the DashMapDpCache SHALL allow parallel access without blocking

### Requirement 2

**User Story:** As a developer, I want the parallel cache to avoid deadlocks during recursive dependency resolution, so that the cache operates correctly under concurrent access.

#### Acceptance Criteria

1. WHEN resolving dependencies THEN the DashMapDpCache SHALL resolve all dependencies outside of any DashMap locks
2. WHEN inserting a computed value THEN the DashMapDpCache SHALL use `entry().or_insert_with()` to avoid race conditions
3. WHEN a ThreadPool is provided THEN the DashMapDpCache SHALL use `pool.install()` to execute parallel dependency resolution

### Requirement 3

**User Story:** As a developer, I want to benchmark sequential vs parallel cache using the Collatz chain problem, so that I can compare their performance characteristics.

#### Acceptance Criteria

1. WHEN running the Collatz benchmark THEN the benchmark SHALL compute chain lengths for a configurable range of starting numbers
2. WHEN running the Collatz benchmark THEN the benchmark SHALL measure and report execution time for sequential DpCache
3. WHEN running the Collatz benchmark THEN the benchmark SHALL measure and report execution time for parallel DashMapDpCache
4. WHEN the benchmark completes THEN the benchmark SHALL verify both implementations produce identical results

### Requirement 4

**User Story:** As a developer, I want the parallel cache to handle the Collatz sequence correctly, so that I can validate the implementation with a real DP problem.

#### Acceptance Criteria

1. WHEN computing Collatz chain length for n=1 THEN the cache SHALL return 0 (base case)
2. WHEN computing Collatz chain length for even n THEN the cache SHALL depend on n/2 and return 1 + chain_length(n/2)
3. WHEN computing Collatz chain length for odd n>1 THEN the cache SHALL depend on 3n+1 and return 1 + chain_length(3n+1)
4. WHEN computing chain lengths for multiple starting values THEN the cache SHALL correctly memoize shared intermediate values

### Requirement 5

**User Story:** As a developer, I want helper functions for the Collatz problem, so that I can easily use the cache for this common benchmark.

#### Acceptance Criteria

1. WHEN calling `collatz_deps(n)` THEN the function SHALL return an empty Vec for n <= 1
2. WHEN calling `collatz_deps(n)` for even n > 1 THEN the function SHALL return `vec![n/2]`
3. WHEN calling `collatz_deps(n)` for odd n > 1 THEN the function SHALL return `vec![3*n+1]`
4. WHEN calling `collatz_compute(n, deps)` with empty deps THEN the function SHALL return 0
5. WHEN calling `collatz_compute(n, deps)` with non-empty deps THEN the function SHALL return `1 + deps[0]`

