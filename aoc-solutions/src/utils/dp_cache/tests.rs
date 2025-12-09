//! Tests for the DP cache module.

use std::cell::Cell;
use std::rc::Rc;

use super::*;

// =============================================================================
// Collatz Chain Length Functions (test helpers)
// =============================================================================

/// Returns the dependencies for computing the Collatz chain length of n.
fn collatz_deps(n: &u64) -> Vec<u64> {
    if *n <= 1 {
        vec![]
    } else if n % 2 == 0 {
        vec![n / 2]
    } else {
        vec![3 * n + 1]
    }
}

/// Computes the Collatz chain length given the index and resolved dependencies.
fn collatz_compute(_n: &u64, deps: Vec<u64>) -> u64 {
    if deps.is_empty() {
        0
    } else {
        1 + deps[0]
    }
}

#[test]
fn test_basic_cache_creation_and_single_value() {
    // Create cache with no dependencies, verify get returns computed value
    let cache = DpCache::new(
        VecBackend::new(),
        |_: &usize| vec![],
        |n: &usize, _deps: Vec<i32>| (*n as i32) * 2,
    );

    assert_eq!(cache.get(&5), 10);
    assert_eq!(cache.get(&0), 0);
    assert_eq!(cache.get(&100), 200);
}


#[test]
fn test_fibonacci_linear_dependency_chain() {
    // fib(n) depends on fib(n-1), fib(n-2)
    let cache = DpCache::new(
        VecBackend::new(),
        |n: &usize| {
            if *n <= 1 {
                vec![]
            } else {
                vec![n - 1, n - 2]
            }
        },
        |n: &usize, deps: Vec<u64>| {
            if *n <= 1 {
                *n as u64
            } else {
                deps[0] + deps[1]
            }
        },
    );

    assert_eq!(cache.get(&0), 0);
    assert_eq!(cache.get(&1), 1);
    assert_eq!(cache.get(&2), 1);
    assert_eq!(cache.get(&3), 2);
    assert_eq!(cache.get(&4), 3);
    assert_eq!(cache.get(&5), 5);
    assert_eq!(cache.get(&10), 55);
    assert_eq!(cache.get(&20), 6765);
}


#[test]
fn test_diamond_dependency_memoization() {
    // Diamond pattern: A(0) depends on B(1) and C(2), both depend on D(3)
    // Verify D is computed only once
    let compute_count = Rc::new(Cell::new(0));
    let count_clone = compute_count.clone();

    let cache = DpCache::new(
        VecBackend::new(),
        |n: &usize| match *n {
            0 => vec![1, 2], // A depends on B, C
            1 => vec![3],    // B depends on D
            2 => vec![3],    // C depends on D
            _ => vec![],     // D has no deps
        },
        move |n: &usize, deps: Vec<i32>| {
            count_clone.set(count_clone.get() + 1);
            match *n {
                0 => deps[0] + deps[1], // A = B + C
                1 => deps[0] * 2,       // B = D * 2
                2 => deps[0] * 3,       // C = D * 3
                3 => 10,                // D = 10
                _ => 0,
            }
        },
    );

    let result = cache.get(&0);
    // D=10, B=20, C=30, A=50
    assert_eq!(result, 50);
    // Should have computed exactly 4 values (A, B, C, D)
    assert_eq!(compute_count.get(), 4);

    // Getting A again should not recompute
    let _ = cache.get(&0);
    assert_eq!(compute_count.get(), 4);
}


#[test]
fn test_vec_backend_get_or_insert() {
    let mut backend: VecBackend<i32> = VecBackend::new();

    // Insert value at index 5
    let value = backend.get_or_insert(5, || 42);
    assert_eq!(*value, 42);

    // Get same index again - should return cached value, not recompute
    let value = backend.get_or_insert(5, || 999);
    assert_eq!(*value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&5), Some(&42));

    // Get returns None for uncached index
    assert_eq!(backend.get(&10), None);

    // Insert at larger index - should not affect existing
    let value = backend.get_or_insert(10, || 100);
    assert_eq!(*value, 100);
    assert_eq!(backend.get(&5), Some(&42));
}


#[test]
fn test_hashmap_backend_get_or_insert() {
    let mut backend: HashMapBackend<String, i32> = HashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42);
    assert_eq!(*value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999);
    assert_eq!(*value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(&42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100);
    assert_eq!(*value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(&42));
}

#[test]
fn test_hashmap_backend_with_cache() {
    // Test HashMapBackend with DpCache using string keys
    let cache = DpCache::new(
        HashMapBackend::new(),
        |s: &String| {
            if s.is_empty() {
                vec![]
            } else {
                vec![s[..s.len() - 1].to_string()]
            }
        },
        |s: &String, deps: Vec<usize>| {
            if s.is_empty() {
                0
            } else {
                deps[0] + 1
            }
        },
    );

    assert_eq!(cache.get(&"".to_string()), 0);
    assert_eq!(cache.get(&"a".to_string()), 1);
    assert_eq!(cache.get(&"ab".to_string()), 2);
    assert_eq!(cache.get(&"abc".to_string()), 3);
}

#[test]
fn test_collatz_base_case() {
    // n=1 should have chain length 0
    let cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    assert_eq!(cache.get(&1u64), 0);
}

#[test]
fn test_collatz_even_numbers() {
    let cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    // 2 -> 1 (length 1)
    assert_eq!(cache.get(&2u64), 1);
    // 4 -> 2 -> 1 (length 2)
    assert_eq!(cache.get(&4u64), 2);
    // 8 -> 4 -> 2 -> 1 (length 3)
    assert_eq!(cache.get(&8u64), 3);
}

#[test]
fn test_collatz_odd_numbers() {
    let cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    // 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1 (length 7)
    assert_eq!(cache.get(&3u64), 7);
    // 5 -> 16 -> 8 -> 4 -> 2 -> 1 (length 5)
    assert_eq!(cache.get(&5u64), 5);
}

#[test]
fn test_collatz_known_values() {
    let cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    // Known Collatz chain lengths
    assert_eq!(cache.get(&6u64), 8);   // 6 -> 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1
    assert_eq!(cache.get(&7u64), 16);  // 7 has a longer chain
    assert_eq!(cache.get(&27u64), 111); // 27 is famous for its long chain
}

#[test]
fn test_parallel_collatz_matches_sequential() {
    // Verify parallel cache produces same results as sequential
    let seq_cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    let par_cache = ParallelDpCache::new(DashMapBackend::new(), collatz_deps, collatz_compute);

    for n in 1..=100u64 {
        assert_eq!(
            seq_cache.get(&n),
            par_cache.get(&n),
            "Mismatch at n={}",
            n
        );
    }
}

#[test]
fn test_dashmap_collatz() {
    // Test DashMapDpCache
    let par_cache = ParallelDpCache::new(DashMapBackend::new(), collatz_deps, collatz_compute);

    assert_eq!(par_cache.get(&1u64), 0);
    assert_eq!(par_cache.get(&2u64), 1);
    assert_eq!(par_cache.get(&3u64), 7);
    assert_eq!(par_cache.get(&27u64), 111);
}


// =============================================================================
// Trait-based API tests
// =============================================================================

/// Fibonacci problem using the trait-based API
struct Fibonacci;

impl DpProblem<usize, u64> for Fibonacci {
    fn deps(&self, n: &usize) -> Vec<usize> {
        if *n <= 1 {
            vec![]
        } else {
            vec![n - 1, n - 2]
        }
    }

    fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
        if *n <= 1 {
            *n as u64
        } else {
            deps[0] + deps[1]
        }
    }
}

#[test]
fn test_trait_based_fibonacci() {
    let cache = DpCache::with_problem(VecBackend::new(), Fibonacci);

    assert_eq!(cache.get(&0), 0);
    assert_eq!(cache.get(&1), 1);
    assert_eq!(cache.get(&2), 1);
    assert_eq!(cache.get(&10), 55);
    assert_eq!(cache.get(&20), 6765);
}

/// Collatz problem using the trait-based API
struct Collatz;

impl DpProblem<u64, u64> for Collatz {
    fn deps(&self, n: &u64) -> Vec<u64> {
        if *n <= 1 {
            vec![]
        } else if n % 2 == 0 {
            vec![n / 2]
        } else {
            vec![3 * n + 1]
        }
    }

    fn compute(&self, _n: &u64, deps: Vec<u64>) -> u64 {
        if deps.is_empty() {
            0
        } else {
            1 + deps[0]
        }
    }
}

#[test]
fn test_trait_based_collatz_sequential() {
    let cache = DpCache::with_problem(HashMapBackend::new(), Collatz);

    assert_eq!(cache.get(&1), 0);
    assert_eq!(cache.get(&2), 1);
    assert_eq!(cache.get(&3), 7);
    assert_eq!(cache.get(&27), 111);
}

#[test]
fn test_trait_based_collatz_parallel() {
    let cache = ParallelDpCache::with_problem(DashMapBackend::new(), Collatz);

    assert_eq!(cache.get(&1), 0);
    assert_eq!(cache.get(&2), 1);
    assert_eq!(cache.get(&3), 7);
    assert_eq!(cache.get(&27), 111);
}

/// Factorial problem using the trait-based API
struct Factorial;

impl DpProblem<usize, u64> for Factorial {
    fn deps(&self, n: &usize) -> Vec<usize> {
        if *n == 0 {
            vec![]
        } else {
            vec![n - 1]
        }
    }

    fn compute(&self, n: &usize, deps: Vec<u64>) -> u64 {
        if *n == 0 {
            1
        } else {
            (*n as u64) * deps[0]
        }
    }
}

#[test]
fn test_trait_based_factorial() {
    let cache = DpCache::with_problem(VecBackend::new(), Factorial);

    assert_eq!(cache.get(&0), 1);
    assert_eq!(cache.get(&1), 1);
    assert_eq!(cache.get(&5), 120);
    assert_eq!(cache.get(&10), 3628800);
}

#[test]
fn test_trait_based_matches_closure_based() {
    // Verify trait-based and closure-based produce same results
    let trait_cache = DpCache::with_problem(VecBackend::new(), Fibonacci);
    let closure_cache = DpCache::new(
        VecBackend::new(),
        |n: &usize| {
            if *n <= 1 {
                vec![]
            } else {
                vec![n - 1, n - 2]
            }
        },
        |n: &usize, deps: Vec<u64>| {
            if *n <= 1 {
                *n as u64
            } else {
                deps[0] + deps[1]
            }
        },
    );

    for n in 0..=20 {
        assert_eq!(trait_cache.get(&n), closure_cache.get(&n), "Mismatch at n={}", n);
    }
}

// =============================================================================
// RwLockDpCache tests
// =============================================================================

#[test]
fn test_rwlock_collatz() {
    // Test RwLockDpCache
    let par_cache =
        ParallelDpCache::new(RwLockHashMapBackend::new(), collatz_deps, collatz_compute);

    assert_eq!(par_cache.get(&1u64), 0);
    assert_eq!(par_cache.get(&2u64), 1);
    assert_eq!(par_cache.get(&3u64), 7);
    assert_eq!(par_cache.get(&27u64), 111);
}

#[test]
fn test_rwlock_collatz_matches_sequential() {
    // Verify RwLock parallel cache produces same results as sequential
    let seq_cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    let par_cache =
        ParallelDpCache::new(RwLockHashMapBackend::new(), collatz_deps, collatz_compute);

    for n in 1..=100u64 {
        assert_eq!(
            seq_cache.get(&n),
            par_cache.get(&n),
            "Mismatch at n={}",
            n
        );
    }
}

#[test]
fn test_trait_based_collatz_rwlock() {
    let cache = ParallelDpCache::with_problem(RwLockHashMapBackend::new(), Collatz);

    assert_eq!(cache.get(&1), 0);
    assert_eq!(cache.get(&2), 1);
    assert_eq!(cache.get(&3), 7);
    assert_eq!(cache.get(&27), 111);
}

// =============================================================================
// ParallelBackend tests
// =============================================================================

#[test]
fn test_dashmap_backend_get_or_insert() {
    let backend: DashMapBackend<String, i32> = DashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42);
    assert_eq!(value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999);
    assert_eq!(value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100);
    assert_eq!(value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(42));
}

#[test]
fn test_rwlock_backend_get_or_insert() {
    let backend: RwLockHashMapBackend<String, i32> = RwLockHashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42);
    assert_eq!(value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999);
    assert_eq!(value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100);
    assert_eq!(value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(42));
}

#[test]
fn test_all_parallel_backends_match() {
    // Verify all parallel backends produce same results
    let dashmap_cache = ParallelDpCache::new(DashMapBackend::new(), collatz_deps, collatz_compute);
    let rwlock_cache =
        ParallelDpCache::new(RwLockHashMapBackend::new(), collatz_deps, collatz_compute);

    for n in 1..=100u64 {
        assert_eq!(
            dashmap_cache.get(&n),
            rwlock_cache.get(&n),
            "Mismatch at n={}",
            n
        );
    }
}
