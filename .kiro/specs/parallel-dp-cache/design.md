# Design Document: Parallel Dynamic Programming Cache

## Overview

This document describes the design for extending the existing `DpCache` with parallel execution support. The extension adds `ParallelDpCache`, a thread-safe cache with pluggable backends that uses Rayon for concurrent dependency resolution. Two parallel backends are provided: `DashMapBackend` (using DashMap for lock-free concurrent access) and `RwLockHashMapBackend` (using RwLock<HashMap> for simpler read-heavy workloads).

The implementation is located in `aoc-solutions/src/utils/dp_cache/`.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ParallelDpCache<I, K, B, P>                          │
│  ┌─────────────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  backend: B             │  │  problem: P     │  │  pool: Option<  │  │
│  │  (ParallelBackend)      │  │  (DpProblem)    │  │  Arc<ThreadPool>│  │
│  └────────┬────────────────┘  └────────┬────────┘  └───────┬─────────┘  │
│           │                            │                   │            │
│           ▼                            ▼                   ▼            │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                    get(&self, &I) -> K                              ││
│  │  1. Fast path: check if already in backend                          ││
│  │  2. Get dependencies via problem.deps() (no locks held)             ││
│  │  3. If pool provided: pool.install(|| par_iter deps)                ││
│  │     Else: rayon par_iter deps                                       ││
│  │  4. Insert via backend.get_or_insert(compute)                       ││
│  │  5. Clone and return                                                ││
│  └─────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│                    ParallelBackend Implementations                       │
├─────────────────────────────────┬───────────────────────────────────────┤
│  DashMapBackend<I, K>           │  RwLockHashMapBackend<I, K>           │
│  ┌───────────────────────────┐  │  ┌─────────────────────────────────┐  │
│  │  data: DashMap<I, K>      │  │  │  data: RwLock<HashMap<I, K>>    │  │
│  │  - Lock-free reads        │  │  │  - Read lock for gets           │  │
│  │  - Sharded writes         │  │  │  - Double-checked locking       │  │
│  │  - High concurrency       │  │  │  - Simpler implementation       │  │
│  └───────────────────────────┘  │  └─────────────────────────────────┘  │
└─────────────────────────────────┴───────────────────────────────────────┘
```

## Components and Interfaces

### ParallelBackend Trait

```rust
pub trait ParallelBackend<I, K>: Send + Sync {
    /// Returns a clone of the cached value for the given index, if it exists.
    fn get(&self, index: &I) -> Option<K>;

    /// Returns the cached value (cloned), or computes and stores it using the provided function.
    fn get_or_insert<F>(&self, index: I, compute: F) -> K
    where
        F: FnOnce() -> K;
}
```

### ParallelDpCache

```rust
pub struct ParallelDpCache<I, K, B, P>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    B: ParallelBackend<I, K>,
    P: ParallelDpProblem<I, K>,
{
    backend: B,
    problem: P,
    pool: Option<Arc<ThreadPool>>,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, P> ParallelDpCache<I, K, B, P> {
    pub fn builder() -> ParallelDpCacheBuilder<I, K, B, P>;
    pub fn get(&self, index: &I) -> K;
}
```

### DashMapBackend

```rust
pub struct DashMapBackend<I, K>
where
    I: Hash + Eq,
{
    data: DashMap<I, K>,
}

impl<I, K> DashMapBackend<I, K> {
    pub fn new() -> Self;
}
```

### RwLockHashMapBackend

```rust
pub struct RwLockHashMapBackend<I, K> {
    data: RwLock<HashMap<I, K>>,
}

impl<I, K> RwLockHashMapBackend<I, K> {
    pub fn new() -> Self;
}
```

### Type Aliases

```rust
pub type DashMapDpCache<I, K, P> = ParallelDpCache<I, K, DashMapBackend<I, K>, P>;
pub type RwLockDpCache<I, K, P> = ParallelDpCache<I, K, RwLockHashMapBackend<I, K>, P>;
```

## Data Models

### Type Parameters

| Parameter | Description | Bounds |
|-----------|-------------|--------|
| `I` | Index type | `Hash + Eq + Clone + Send + Sync` |
| `K` | Value type | `Clone + Send + Sync` |
| `B` | Backend type | `ParallelBackend<I, K>` |
| `P` | Problem type | `ParallelDpProblem<I, K>` |

### Collatz Chain Length Problem

```rust
struct Collatz;

impl DpProblem<u64, u64> for Collatz {
    fn deps(&self, n: &u64) -> Vec<u64> {
        if *n <= 1 { vec![] }
        else if n % 2 == 0 { vec![n / 2] }
        else { vec![3 * n + 1] }
    }

    fn compute(&self, _n: &u64, deps: Vec<u64>) -> u64 {
        if deps.is_empty() { 0 } else { 1 + deps[0] }
    }
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Sequential-Parallel Equivalence
*For any* DP problem and any index, the result from `ParallelDpCache` with `DashMapBackend` SHALL equal the result from sequential `DpCache` with `HashMapBackend`.
**Validates: Requirements 1.4, 1.5, 2.3**

### Property 2: Backend Equivalence
*For any* DP problem and any index, the result from `ParallelDpCache` with `DashMapBackend` SHALL equal the result from `ParallelDpCache` with `RwLockHashMapBackend`.
**Validates: Requirements 2.3, 3.3**

### Property 3: Memoization Correctness
*For any* index that has been computed, subsequent calls to `get` SHALL return the same value without recomputation.
**Validates: Requirements 1.5**

### Property 4: Collatz Recurrence
*For any* n > 1, the Collatz chain length SHALL equal 1 + chain_length(next) where next = n/2 (even) or 3n+1 (odd).
**Validates: Requirements 6.2, 6.3**

## Error Handling

### Undefined Behavior (Documented)

1. **Cyclic dependencies**: Results in deadlock or stack overflow

### Thread Safety Considerations

- DashMapBackend provides lock-free reads and minimal contention for writes to different shards
- RwLockHashMapBackend uses double-checked locking to minimize write lock contention
- Dependencies are resolved outside of any locks to prevent deadlock
- `get_or_insert` only holds the lock during `compute` execution (not during recursive `get` calls)

## Testing Strategy

### Unit Tests

- ParallelDpCache with DashMapBackend: Fibonacci, Collatz computation
- ParallelDpCache with RwLockHashMapBackend: Fibonacci, Collatz computation
- Backend get_or_insert behavior for both backends
- Memoization verification (compute called only once per index)
- Sequential-parallel equivalence for Collatz results
- Backend equivalence (DashMap vs RwLock produce same results)

### Benchmark Examples

Two benchmark examples comparing all backends:
1. `collatz_benchmark.rs`: Collatz chain lengths for random numbers
2. `pattern_benchmark.rs`: Decimal pattern computation

Both benchmarks:
- Measure execution time for each backend
- Verify all backends produce identical results
- Compare sequential vs parallel performance

## Dependencies

Crate dependencies for `aoc-solutions`:
- `rayon = "1.10"` - Data parallelism
- `dashmap = "6"` - Concurrent HashMap
