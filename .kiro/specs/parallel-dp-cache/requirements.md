# Requirements Document

## Introduction

This document specifies requirements for extending the existing dynamic programming cache (`DpCache`) with parallel execution support. The extension adds `ParallelDpCache`, a thread-safe cache with pluggable backends that uses Rayon for concurrent dependency resolution. Two parallel backends are provided: `DashMapBackend` (using DashMap for lock-free concurrent access) and `RwLockHashMapBackend` (using RwLock<HashMap> for simpler read-heavy workloads). Benchmarks comparing sequential `DpCache` vs parallel `ParallelDpCache` using the Collatz chain length problem validate the implementation.

## Glossary

- **DpCache**: The existing single-threaded memoization cache with pluggable backends (VecBackend, HashMapBackend)
- **ParallelDpCache**: A thread-safe cache with pluggable parallel backends and Rayon for parallel dependency resolution
- **ParallelBackend**: A trait defining the interface for thread-safe storage backends
- **DashMapBackend**: A parallel backend using DashMap for lock-free concurrent access
- **RwLockHashMapBackend**: A parallel backend using RwLock<HashMap> for simpler concurrent access
- **DashMapDpCache**: Type alias for `ParallelDpCache` with `DashMapBackend`
- **RwLockDpCache**: Type alias for `ParallelDpCache` with `RwLockHashMapBackend`
- **DashMap**: A concurrent HashMap implementation providing lock-free reads and sharded writes
- **Collatz Sequence**: The sequence where n → n/2 (if even) or n → 3n+1 (if odd), terminating at 1
- **Collatz Chain Length**: The number of steps to reach 1 from a starting number
- **Rayon**: A data parallelism library for Rust
- **ThreadPool**: A Rayon thread pool that can be optionally provided for parallel execution

## Requirements

### Requirement 1

**User Story:** As a developer, I want a parallel DP cache with pluggable backends, so that I can choose the best concurrency strategy for my use case.

#### Acceptance Criteria

1. WHEN creating a ParallelDpCache THEN the ParallelDpCache SHALL accept any type implementing the ParallelBackend trait
2. WHEN creating a ParallelDpCache THEN the ParallelDpCache SHALL accept a problem definition implementing ParallelDpProblem trait
3. WHEN creating a ParallelDpCache THEN the ParallelDpCache SHALL optionally accept a Rayon ThreadPool via the builder
4. WHEN `get` is called for an uncomputed index THEN the ParallelDpCache SHALL resolve dependencies in parallel using Rayon's `par_iter`
5. WHEN `get` is called for an already-computed index THEN the ParallelDpCache SHALL return the cached value without recomputation
6. WHEN multiple threads access different keys concurrently THEN the ParallelDpCache SHALL allow parallel access without blocking

### Requirement 2

**User Story:** As a developer, I want a DashMap-based parallel backend, so that I can have lock-free concurrent access with minimal contention.

#### Acceptance Criteria

1. WHEN creating a DashMapBackend THEN the DashMapBackend SHALL use `DashMap<I, K>` for direct value storage
2. WHEN calling `get` on DashMapBackend THEN the DashMapBackend SHALL return a clone of the cached value if present
3. WHEN calling `get_or_insert` on DashMapBackend THEN the DashMapBackend SHALL compute and store the value atomically if not present
4. WHEN multiple threads access different shards THEN the DashMapBackend SHALL allow concurrent access without blocking

### Requirement 3

**User Story:** As a developer, I want an RwLock<HashMap>-based parallel backend, so that I have a simpler alternative for read-heavy workloads.

#### Acceptance Criteria

1. WHEN creating a RwLockHashMapBackend THEN the RwLockHashMapBackend SHALL use `RwLock<HashMap<I, K>>` for storage
2. WHEN calling `get` on RwLockHashMapBackend THEN the RwLockHashMapBackend SHALL acquire a read lock and return a clone
3. WHEN calling `get_or_insert` on RwLockHashMapBackend THEN the RwLockHashMapBackend SHALL use double-checked locking to minimize write lock contention
4. WHEN multiple threads read concurrently THEN the RwLockHashMapBackend SHALL allow parallel reads without blocking

### Requirement 4

**User Story:** As a developer, I want the parallel cache to avoid deadlocks during recursive dependency resolution, so that the cache operates correctly under concurrent access.

#### Acceptance Criteria

1. WHEN resolving dependencies THEN the ParallelDpCache SHALL resolve all dependencies outside of any backend locks
2. WHEN inserting a computed value THEN the ParallelDpCache SHALL use the backend's `get_or_insert` to avoid race conditions
3. WHEN a ThreadPool is provided THEN the ParallelDpCache SHALL use `pool.install()` to execute parallel dependency resolution

### Requirement 5

**User Story:** As a developer, I want to benchmark sequential vs parallel cache using the Collatz chain problem, so that I can compare their performance characteristics.

#### Acceptance Criteria

1. WHEN running the Collatz benchmark THEN the benchmark SHALL compute chain lengths for a configurable range of starting numbers
2. WHEN running the Collatz benchmark THEN the benchmark SHALL measure and report execution time for sequential DpCache with HashMapBackend
3. WHEN running the Collatz benchmark THEN the benchmark SHALL measure and report execution time for ParallelDpCache with DashMapBackend
4. WHEN running the Collatz benchmark THEN the benchmark SHALL measure and report execution time for ParallelDpCache with RwLockHashMapBackend
5. WHEN the benchmark completes THEN the benchmark SHALL verify all implementations produce identical results

### Requirement 6

**User Story:** As a developer, I want the parallel cache to handle the Collatz sequence correctly, so that I can validate the implementation with a real DP problem.

#### Acceptance Criteria

1. WHEN computing Collatz chain length for n=1 THEN the cache SHALL return 0 (base case)
2. WHEN computing Collatz chain length for even n THEN the cache SHALL depend on n/2 and return 1 + chain_length(n/2)
3. WHEN computing Collatz chain length for odd n>1 THEN the cache SHALL depend on 3n+1 and return 1 + chain_length(3n+1)
4. WHEN computing chain lengths for multiple starting values THEN the cache SHALL correctly memoize shared intermediate values
