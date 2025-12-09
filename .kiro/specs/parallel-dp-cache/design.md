# Design Document: Parallel Dynamic Programming Cache

## Overview

This document describes the design for extending the existing `DpCache` with parallel execution support. The extension adds `DashMapDpCache`, a thread-safe cache that uses DashMap for storage and Rayon for concurrent dependency resolution. A Collatz chain length benchmark validates correctness and compares performance between sequential and parallel implementations.

The implementation is located in `aoc-solutions/src/utils/dp_cache.rs`.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    DashMapDpCache<I, K, D, C>                           │
│  ┌─────────────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  data: DashMap<I, K>    │  │  dep_fn: D      │  │  compute_fn: C  │  │
│  │  (direct value storage) │  │  I -> Vec<I>    │  │  (&I, Vec<K>)->K│  │
│  └────────┬────────────────┘  └────────┬────────┘  └───────┬─────────┘  │
│           │                            │                   │            │
│  ┌────────┴────────┐                   │                   │            │
│  │ pool: Option<   │                   │                   │            │
│  │   Arc<ThreadPool>>                  │                   │            │
│  └────────┬────────┘                   │                   │            │
│           │                            │                   │            │
│           ▼                            ▼                   ▼            │
│  ┌─────────────────────────────────────────────────────────────────────┐│
│  │                    get(&self, I) -> K                               ││
│  │  1. Fast path: check if already in DashMap                          ││
│  │  2. Get dependencies via dep_fn (no locks held)                     ││
│  │  3. If pool provided: pool.install(|| par_iter deps)                ││
│  │     Else: rayon par_iter deps                                       ││
│  │  4. Insert via entry().or_insert_with(compute_fn)                   ││
│  │  5. Clone and return                                                ││
│  └─────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### DashMapDpCache

A parallel DP cache using DashMap directly for storage. Values are stored directly (not wrapped in `OnceLock`) to avoid lifetime issues with DashMap references.

```rust
pub struct DashMapDpCache<I, K, D, C>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    D: Fn(&I) -> Vec<I> + Send + Sync,
    C: Fn(&I, Vec<K>) -> K + Send + Sync,
{
    data: DashMap<I, K>,
    dep_fn: D,
    compute_fn: C,
    pool: Option<Arc<ThreadPool>>,
}

impl<I, K, D, C> DashMapDpCache<I, K, D, C>
where
    I: Hash + Eq + Clone + Send + Sync,
    K: Clone + Send + Sync,
    D: Fn(&I) -> Vec<I> + Send + Sync,
    C: Fn(&I, Vec<K>) -> K + Send + Sync,
{
    /// Creates a new DashMapDpCache using the global Rayon thread pool.
    pub fn new(dep_fn: D, compute_fn: C) -> Self;
    
    /// Creates a new DashMapDpCache using a custom Rayon thread pool.
    pub fn with_pool(dep_fn: D, compute_fn: C, pool: Arc<ThreadPool>) -> Self;
    
    /// Retrieves the value for the given index, computing it if necessary.
    pub fn get(&self, index: I) -> K;
}
```

### DashMapDpCache::get Implementation Detail

The `get` method resolves dependencies in parallel using Rayon. Dependencies are resolved outside of any DashMap locks to avoid deadlock:

```rust
pub fn get(&self, index: I) -> K {
    // Fast path: check if already computed
    if let Some(entry) = self.data.get(&index) {
        return entry.value().clone();
    }

    // Get dependencies (no locks held)
    let deps = (self.dep_fn)(&index);

    // Resolve dependencies IN PARALLEL using Rayon (no locks held)
    let resolve_deps = || {
        deps.into_par_iter()
            .map(|dep| self.get(dep))  // Recursive parallel calls
            .collect::<Vec<K>>()
    };

    let dep_values = match &self.pool {
        Some(pool) => pool.install(resolve_deps),  // Use provided pool
        None => resolve_deps(),                     // Use global pool
    };

    // Insert using or_insert_with - only compute_fn is inside the closure
    // dep_values is already resolved outside, so no recursive calls happen while holding the lock
    self.data
        .entry(index.clone())
        .or_insert_with(|| (self.compute_fn)(&index, dep_values))
        .value()
        .clone()
}
```

**Key Design Decision**: We cannot use `or_insert_with` for the entire computation because it holds a write lock on the DashMap shard while the closure executes. If the closure calls `self.get()` recursively and hits the same shard, it would deadlock. Instead, we compute dependencies first (releasing any locks), then insert.

## Data Models

### Type Parameters

| Parameter | Description | Bounds |
|-----------|-------------|--------|
| `I` | Index type | `Hash + Eq + Clone + Send + Sync` |
| `K` | Value type | `Clone + Send + Sync` |
| `D` | Dependency function | `Fn(&I) -> Vec<I> + Send + Sync` |
| `C` | Compute function | `Fn(&I, Vec<K>) -> K + Send + Sync` |

### Collatz Chain Length Problem

The Collatz sequence for a number n:
- If n = 1: sequence terminates (chain length = 0)
- If n is even: next = n / 2
- If n is odd: next = 3n + 1

```rust
// Dependency function
fn collatz_deps(n: &u64) -> Vec<u64> {
    if *n <= 1 { vec![] }
    else if n % 2 == 0 { vec![n / 2] }
    else { vec![3 * n + 1] }
}

// Compute function
fn collatz_compute(n: &u64, deps: Vec<u64>) -> u64 {
    if *n <= 1 { 0 }
    else { 1 + deps[0] }
}
```



## Correctness Criteria

The following correctness criteria will be validated through unit tests:

1. **DashMapDpCache correctness**: Parallel cache produces same results as sequential `DpCache`
2. **DashMapDpCache memoization**: Multiple `get` calls compute only once
3. **DashMapDpCache concurrent safety**: Concurrent operations on different keys succeed
4. **Collatz recurrence**: Chain length follows the recurrence relation
5. **Sequential-parallel equivalence**: DashMapDpCache and DpCache produce identical Collatz results

## Error Handling

### Undefined Behavior (Documented)

The following scenarios result in undefined behavior:

1. **Cyclic dependencies**: Same as single-threaded version - stack overflow or deadlock

### Thread Safety Considerations

- DashMap provides lock-free reads and minimal contention for writes to different shards
- Dependencies are resolved outside of any locks to prevent deadlock
- `entry().or_insert_with()` only holds the lock during `compute_fn` execution (not during recursive `get` calls)

## Testing Strategy

### Unit Tests

- DashMapDpCache Fibonacci computation
- DashMapDpCache with custom ThreadPool
- DashMapDpCache memoization verification
- Collatz chain length correctness (base cases, even/odd numbers, known values)
- Parallel cache matches sequential cache for Collatz results

### Benchmark

A benchmark example comparing sequential `DpCache` vs parallel `DashMapDpCache`:
- Input: Collatz chain lengths for numbers 1 to N (configurable)
- Output: Execution time for each implementation
- Verification: Both implementations produce identical results

## Dependencies

New crate dependencies for `aoc-solutions`:
- `rayon = "1.10"` - Data parallelism
- `dashmap = "6"` - Concurrent HashMap
