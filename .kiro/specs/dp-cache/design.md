# Design Document: Dynamic Programming Cache

## Overview

This document describes the design for a dynamic programming cache (`DpCache`) that provides memoization with lazy evaluation and automatic dependency resolution. The cache is backed by a pluggable storage backend, uses `RefCell` for interior mutability, and leverages `OnceCell` for single-initialization semantics.

The module will be located at `aoc-solutions/src/utils/dp_cache.rs`.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         DpCache<I, K, B>                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │  RefCell<B>     │  │  dep_fn         │  │  compute_fn │  │
│  │  (Backend)      │  │  I -> Vec<I>    │  │  (&C,I)->K  │  │
│  └────────┬────────┘  └────────┬────────┘  └──────┬──────┘  │
│           │                    │                  │         │
│           ▼                    ▼                  ▼         │
│  ┌─────────────────────────────────────────────────────────┐│
│  │                    get(&self, I) -> K                   ││
│  │  1. Check if exists & initialized → return clone        ││
│  │  2. ensure_index (borrow_mut, then drop)                ││
│  │  3. Resolve dependencies recursively                    ││
│  │  4. OnceCell::get_or_init with compute_fn               ││
│  │  5. Clone and return                                    ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Backend Trait                            │
│  ┌─────────────────────────────────────────────────────────┐│
│  │  get(&self, &I) -> &OnceCell<K>                         ││
│  │  ensure_index(&mut self, I)                             ││
│  └─────────────────────────────────────────────────────────┘│
│                              │                              │
│              ┌───────────────┴───────────────┐              │
│              ▼                               ▼              │
│  ┌─────────────────────┐       ┌─────────────────────────┐  │
│  │    VecBackend<K>    │       │  HashMapBackend<I, K>   │  │
│  │  Vec<OnceCell<K>>   │       │  HashMap<I,OnceCell<K>> │  │
│  │  Index: usize       │       │  Index: I: Hash + Eq    │  │
│  └─────────────────────┘       └─────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### Backend Trait

```rust
pub trait Backend<I, K> {
    /// Returns a reference to the OnceCell for the given index.
    /// Panics if the index has not been ensured.
    fn get(&self, index: &I) -> &OnceCell<K>;
    
    /// Ensures an entry exists for the given index.
    /// If the index is new, creates an empty OnceCell.
    /// If the index exists, leaves it unchanged.
    fn ensure_index(&mut self, index: I);
}
```

### VecBackend

```rust
pub struct VecBackend<K> {
    data: Vec<OnceCell<K>>,
}

impl<K> VecBackend<K> {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
}

impl<K> Backend<usize, K> for VecBackend<K> {
    fn get(&self, index: &usize) -> &OnceCell<K>;
    fn ensure_index(&mut self, index: usize);
}
```

### HashMapBackend

```rust
pub struct HashMapBackend<I, K> {
    data: HashMap<I, OnceCell<K>>,
}

impl<I, K> HashMapBackend<I, K> {
    pub fn new() -> Self;
}

impl<I: Hash + Eq, K> Backend<I, K> for HashMapBackend<I, K> {
    fn get(&self, index: &I) -> &OnceCell<K>;
    fn ensure_index(&mut self, index: I);
}
```

### DpCache

```rust
pub struct DpCache<I, K, B, D, C>
where
    B: Backend<I, K>,
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    backend: RefCell<B>,
    dep_fn: D,
    compute_fn: C,
    _phantom: PhantomData<(I, K)>,
}

impl<I, K, B, D, C> DpCache<I, K, B, D, C>
where
    I: Clone,
    K: Clone,
    B: Backend<I, K>,
    D: Fn(&I) -> Vec<I>,
    C: Fn(&I, Vec<K>) -> K,
{
    pub fn new(backend: B, dep_fn: D, compute_fn: C) -> Self;
    
    pub fn get(&self, index: I) -> K;
}
```

## Data Models

### Type Parameters

| Parameter | Description | Bounds |
|-----------|-------------|--------|
| `I` | Index type | `Clone` |
| `K` | Value type | `Clone` |
| `B` | Backend type | `Backend<I, K>` |
| `D` | Dependency function | `Fn(&I) -> Vec<I>` |
| `C` | Compute function | `Fn(&I, Vec<K>) -> K` |

### Internal State

- `backend: RefCell<B>` - The storage backend wrapped in RefCell for interior mutability
- `dep_fn: D` - Function that returns dependencies for an index
- `compute_fn: C` - Function that computes the value for an index given the resolved dependency values

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Cache returns correctly computed values

*For any* valid DAG of dependencies and any index `i`, calling `cache.get(i)` SHALL return the value that would be computed by `compute_fn(&cache, &i)` after all dependencies are resolved.

**Validates: Requirements 2.1, 2.4**

### Property 2: Memoization prevents redundant computation

*For any* index `i`, calling `cache.get(i)` multiple times SHALL invoke the compute function exactly once.

**Validates: Requirements 2.2**

### Property 3: Dependencies are resolved before dependents

*For any* DAG where index `i` depends on indices `[d1, d2, ..., dn]`, when computing `cache.get(i)`, all dependency values SHALL be initialized before `compute_fn` is called for `i`.

**Validates: Requirements 2.4**

### Property 4: Backend ensure_index is idempotent

*For any* backend and index `i`, calling `ensure_index(i)` on an already-initialized cell SHALL preserve the existing value.

**Validates: Requirements 3.3, 3.4**

### Property 5: VecBackend ensure-get consistency

*For any* VecBackend and index `i: usize`, after calling `ensure_index(i)`, calling `get(&i)` SHALL return a valid OnceCell reference.

**Validates: Requirements 6.2, 6.3**

### Property 6: HashMapBackend ensure-get consistency

*For any* HashMapBackend and key `k: I` where `I: Hash + Eq`, after calling `ensure_index(k)`, calling `get(&k)` SHALL return a valid OnceCell reference.

**Validates: Requirements 7.2, 7.3**

## Error Handling

### Undefined Behavior (Documented)

The following scenarios result in undefined behavior and are explicitly not handled:

1. **Cyclic dependencies**: If the dependency function returns a graph with cycles, the cache will either:
   - Stack overflow (infinite recursion)
   - Freeze (if tail-call optimized)
   
   Users MUST ensure their dependency graph forms a DAG.

2. **Backend index not ensured**: Calling `backend.get()` for an index that was never ensured will panic.

### Runtime Panics

- `RefCell` borrow violations: Should not occur with correct implementation, but would panic if borrow rules are violated.

## Testing Strategy

### Unit Tests

- Basic cache creation and usage
- Linear dependency chain (fibonacci-style)
- Diamond dependency pattern
- No dependencies (base cases only)
- Memoization verification (compute function called once per index)
- Backend ensure_index idempotence

## Future Considerations

### Concurrent Version

The design can be adapted for concurrent use by replacing:
- `RefCell<B>` → `Mutex<B>` or `RwLock<B>`
- `OnceCell<K>` → `OnceLock<K>`

This works cleanly because:
1. `compute_fn: Fn(&I, Vec<K>) -> K` receives resolved dependency values directly
2. No self-reference needed - compute_fn doesn't access the cache
3. Dependency values are collected **before** acquiring the lock for `get_or_init`
4. The lock held during `get_or_init` doesn't need re-entrant access
5. `Mutex` works because there's no recursive lock acquisition inside the closure

Caveats for concurrent version:
- Same UB for cycles (manifests as deadlock instead of stack overflow)
- Requires `K: Clone + Send + Sync`, `I: Clone + Send + Sync`

This is out of scope for the current implementation but the design supports it.
