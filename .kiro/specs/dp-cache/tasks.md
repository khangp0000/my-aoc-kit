# Implementation Plan

- [x] 1. Set up module structure
  - Create `aoc-solutions/src/utils/mod.rs` if it doesn't exist
  - Create `aoc-solutions/src/utils/dp_cache.rs`
  - Export utils module from `aoc-solutions/src/lib.rs`
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 2. Implement Backend trait and implementations
  - [x] 2.1 Define the Backend trait with `get` and `ensure_index` methods
    - Define trait with generic parameters `I` (index) and `K` (value)
    - `get(&self, index: &I) -> &OnceCell<K>`
    - `ensure_index(&mut self, index: I)`
    - _Requirements: 3.1, 3.2_
  - [x] 2.2 Implement VecBackend for usize indices
    - Struct with `Vec<OnceCell<K>>`
    - `new()` and `with_capacity(usize)` constructors
    - Implement Backend trait: extend vec in `ensure_index` if needed
    - _Requirements: 6.1, 6.2, 6.3_
  - [x] 2.3 Implement HashMapBackend for hashable indices
    - Struct with `HashMap<I, OnceCell<K>>`
    - `new()` constructor
    - Implement Backend trait: insert empty OnceCell in `ensure_index` if key missing
    - _Requirements: 7.1, 7.2, 7.3_

- [x] 3. Implement DpCache struct
  - [x] 3.1 Define DpCache struct with generic parameters
    - `backend: RefCell<B>`
    - `dep_fn: D` where `D: Fn(&I) -> Vec<I>`
    - `compute_fn: C` where `C: Fn(&I, Vec<K>) -> K`
    - Add PhantomData for unused type parameters if needed
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 3.2 Implement `new` constructor
    - Accept backend, dep_fn, and compute_fn
    - Wrap backend in RefCell
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 3.3 Implement `get(&self, index: I) -> K` method
    - Check if index exists and is initialized, return clone if so
    - Call `ensure_index` via `borrow_mut`, drop borrow immediately
    - Get dependencies via `dep_fn`
    - Recursively resolve dependencies by calling `get` for each
    - Borrow backend, get OnceCell, call `get_or_init` with compute_fn
    - Clone and return result
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 4.1, 4.2, 4.3, 4.4_

- [x] 4. Add documentation
  - Add module-level documentation explaining the cache purpose
  - Document Backend trait and its contract
  - Document DpCache with usage examples
  - Add explicit warning about cycle behavior (UB: stack overflow or freeze)
  - Document that users must ensure dependencies form a DAG
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 5. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. Write unit tests
  - [x] 6.1 Test basic cache creation and single value retrieval
    - Create cache with no dependencies, verify get returns computed value
    - _Requirements: 1.1, 2.1_
  - [x] 6.2 Test linear dependency chain (fibonacci-style)
    - fib(n) depends on fib(n-1), fib(n-2)
    - Verify correct values computed
    - _Requirements: 2.4_
  - [x] 6.3 Test diamond dependency pattern
    - A depends on B and C, both B and C depend on D
    - Verify D computed only once (memoization)
    - _Requirements: 2.2_
  - [x] 6.4 Test VecBackend ensure_index idempotence
    - Initialize a cell, call ensure_index again, verify value preserved
    - _Requirements: 3.3, 3.4_
  - [x] 6.5 Test HashMapBackend ensure_index idempotence
    - Initialize a cell, call ensure_index again, verify value preserved
    - _Requirements: 3.3, 3.4_

- [x] 7. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
