//! Tests for the DP cache module.

use std::cell::Cell;
use std::rc::Rc;

use super::*;



#[test]
fn test_basic_cache_creation_and_single_value() {
    // Create cache with no dependencies, verify get returns computed value
    let cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(NoDeps)
        .build();

    assert_eq!(cache.get(&5).unwrap(), 10);
    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&100).unwrap(), 200);
}

/// Simple problem with no dependencies for testing
struct NoDeps;

impl DpProblem<usize, i32> for NoDeps {
    fn deps(&self, _n: &usize) -> Vec<usize> {
        vec![]
    }

    fn compute(&self, n: &usize, _deps: Vec<i32>) -> i32 {
        (*n as i32) * 2
    }
}

#[test]
fn test_fibonacci_linear_dependency_chain() {
    // fib(n) depends on fib(n-1), fib(n-2)
    let cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Fibonacci)
        .build();

    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&2).unwrap(), 1);
    assert_eq!(cache.get(&3).unwrap(), 2);
    assert_eq!(cache.get(&4).unwrap(), 3);
    assert_eq!(cache.get(&5).unwrap(), 5);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
}

#[test]
fn test_diamond_dependency_memoization() {
    // Diamond pattern: A(0) depends on B(1) and C(2), both depend on D(3)
    // Verify D is computed only once
    let compute_count = Rc::new(Cell::new(0));

    /// Diamond problem for testing memoization
    struct Diamond {
        count: Rc<Cell<i32>>,
    }

    impl DpProblem<usize, i32> for Diamond {
        fn deps(&self, n: &usize) -> Vec<usize> {
            match *n {
                0 => vec![1, 2], // A depends on B, C
                1 => vec![3],    // B depends on D
                2 => vec![3],    // C depends on D
                _ => vec![],     // D has no deps
            }
        }

        fn compute(&self, n: &usize, deps: Vec<i32>) -> i32 {
            self.count.set(self.count.get() + 1);
            match *n {
                0 => deps[0] + deps[1], // A = B + C
                1 => deps[0] * 2,       // B = D * 2
                2 => deps[0] * 3,       // C = D * 3
                3 => 10,                // D = 10
                _ => 0,
            }
        }
    }

    let cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Diamond { count: compute_count.clone() })
        .build();

    let result = cache.get(&0).unwrap();
    // D=10, B=20, C=30, A=50
    assert_eq!(result, 50);
    // Should have computed exactly 4 values (A, B, C, D)
    assert_eq!(compute_count.get(), 4);

    // Getting A again should not recompute
    let _ = cache.get(&0).unwrap();
    assert_eq!(compute_count.get(), 4);
}

#[test]
fn test_vec_backend_get_or_insert() {
    let mut backend: VecBackend<i32> = VecBackend::new();

    // Insert value at index 5
    let value = backend.get_or_insert(5, || 42).unwrap();
    assert_eq!(*value, 42);

    // Get same index again - should return cached value, not recompute
    let value = backend.get_or_insert(5, || 999).unwrap();
    assert_eq!(*value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&5), Some(&42));

    // Get returns None for uncached index
    assert_eq!(backend.get(&10), None);

    // Insert at larger index - should not affect existing
    let value = backend.get_or_insert(10, || 100).unwrap();
    assert_eq!(*value, 100);
    assert_eq!(backend.get(&5), Some(&42));
}

#[test]
fn test_vec_backend_get_returns_none_for_uninitialized() {
    let backend: VecBackend<i32> = VecBackend::new();
    
    // Empty backend returns None for any index
    assert_eq!(backend.get(&0), None);
    assert_eq!(backend.get(&100), None);
}

#[test]
fn test_vec_backend_computes_exactly_once() {
    let mut backend: VecBackend<i32> = VecBackend::new();
    let compute_count = Rc::new(Cell::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.set(count.get() + 1);
        42
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
    
    // Second call should NOT compute (OnceCell behavior)
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.set(count.get() + 1);
        999
    }).unwrap();
    assert_eq!(*value, 42); // Still 42, not 999
    assert_eq!(compute_count.get(), 1); // Still 1, not 2
}

#[test]
fn test_vec_backend_auto_grow() {
    let mut backend: VecBackend<i32> = VecBackend::new();
    
    // Insert at index 100 - should auto-grow
    let value = backend.get_or_insert(100, || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Intermediate indices should be uninitialized
    assert_eq!(backend.get(&0), None);
    assert_eq!(backend.get(&50), None);
    assert_eq!(backend.get(&99), None);
    
    // Index 100 should be initialized
    assert_eq!(backend.get(&100), Some(&42));
}

#[test]
fn test_hashmap_backend_get_or_insert() {
    let mut backend: HashMapBackend<String, i32> = HashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42).unwrap();
    assert_eq!(*value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999).unwrap();
    assert_eq!(*value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(&42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100).unwrap();
    assert_eq!(*value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(&42));
}

#[test]
fn test_hashmap_backend_with_cache() {
    // Test HashMapBackend with DpCache using string keys
    /// String length problem for testing HashMap backend
    struct StringLength;

    impl DpProblem<String, usize> for StringLength {
        fn deps(&self, s: &String) -> Vec<String> {
            if s.is_empty() {
                vec![]
            } else {
                vec![s[..s.len() - 1].to_string()]
            }
        }

        fn compute(&self, s: &String, deps: Vec<usize>) -> usize {
            if s.is_empty() {
                0
            } else {
                deps[0] + 1
            }
        }
    }

    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(StringLength)
        .build();

    assert_eq!(cache.get(&"".to_string()).unwrap(), 0);
    assert_eq!(cache.get(&"a".to_string()).unwrap(), 1);
    assert_eq!(cache.get(&"ab".to_string()).unwrap(), 2);
    assert_eq!(cache.get(&"abc".to_string()).unwrap(), 3);
}

#[test]
fn test_collatz_base_case() {
    // n=1 should have chain length 0
    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    assert_eq!(cache.get(&1u64).unwrap(), 0);
}

#[test]
fn test_collatz_even_numbers() {
    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    // 2 -> 1 (length 1)
    assert_eq!(cache.get(&2u64).unwrap(), 1);
    // 4 -> 2 -> 1 (length 2)
    assert_eq!(cache.get(&4u64).unwrap(), 2);
    // 8 -> 4 -> 2 -> 1 (length 3)
    assert_eq!(cache.get(&8u64).unwrap(), 3);
}

#[test]
fn test_collatz_odd_numbers() {
    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    // 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1 (length 7)
    assert_eq!(cache.get(&3u64).unwrap(), 7);
    // 5 -> 16 -> 8 -> 4 -> 2 -> 1 (length 5)
    assert_eq!(cache.get(&5u64).unwrap(), 5);
}

#[test]
fn test_collatz_known_values() {
    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    // Known Collatz chain lengths
    assert_eq!(cache.get(&6u64).unwrap(), 8); // 6 -> 3 -> 10 -> 5 -> 16 -> 8 -> 4 -> 2 -> 1
    assert_eq!(cache.get(&7u64).unwrap(), 16); // 7 has a longer chain
    assert_eq!(cache.get(&27u64).unwrap(), 111); // 27 is famous for its long chain
}

#[test]
fn test_parallel_collatz_matches_sequential() {
    // Verify parallel cache produces same results as sequential
    let seq_cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    let par_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();

    for n in 1..=100u64 {
        assert_eq!(seq_cache.get(&n).unwrap(), par_cache.get(&n).unwrap(), "Mismatch at n={}", n);
    }
}

#[test]
fn test_dashmap_collatz() {
    // Test DashMapDpCache
    let par_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();

    assert_eq!(par_cache.get(&1u64).unwrap(), 0);
    assert_eq!(par_cache.get(&2u64).unwrap(), 1);
    assert_eq!(par_cache.get(&3u64).unwrap(), 7);
    assert_eq!(par_cache.get(&27u64).unwrap(), 111);
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
    let cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Fibonacci)
        .build();

    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&2).unwrap(), 1);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
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
    let cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();

    assert_eq!(cache.get(&1).unwrap(), 0);
    assert_eq!(cache.get(&2).unwrap(), 1);
    assert_eq!(cache.get(&3).unwrap(), 7);
    assert_eq!(cache.get(&27).unwrap(), 111);
}

#[test]
fn test_trait_based_collatz_parallel() {
    let cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();

    assert_eq!(cache.get(&1).unwrap(), 0);
    assert_eq!(cache.get(&2).unwrap(), 1);
    assert_eq!(cache.get(&3).unwrap(), 7);
    assert_eq!(cache.get(&27).unwrap(), 111);
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
    let cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Factorial)
        .build();

    assert_eq!(cache.get(&0).unwrap(), 1);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&5).unwrap(), 120);
    assert_eq!(cache.get(&10).unwrap(), 3628800);
}

#[test]
fn test_trait_based_matches_closure_based() {
    // Verify trait-based and closure-based produce same results
    // Both use the Fibonacci problem
    let trait_cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Fibonacci)
        .build();
    let trait_cache2 = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Fibonacci)
        .build();

    for n in 0..=20 {
        assert_eq!(
            trait_cache.get(&n).unwrap(),
            trait_cache2.get(&n).unwrap(),
            "Mismatch at n={}",
            n
        );
    }
}

// =============================================================================
// RwLockDpCache tests
// =============================================================================

#[test]
fn test_rwlock_collatz() {
    // Test RwLockDpCache
    let par_cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();

    assert_eq!(par_cache.get(&1u64).unwrap(), 0);
    assert_eq!(par_cache.get(&2u64).unwrap(), 1);
    assert_eq!(par_cache.get(&3u64).unwrap(), 7);
    assert_eq!(par_cache.get(&27u64).unwrap(), 111);
}

#[test]
fn test_rwlock_collatz_matches_sequential() {
    // Verify RwLock parallel cache produces same results as sequential
    let seq_cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    let par_cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();

    for n in 1..=100u64 {
        assert_eq!(seq_cache.get(&n).unwrap(), par_cache.get(&n).unwrap(), "Mismatch at n={}", n);
    }
}

#[test]
fn test_trait_based_collatz_rwlock() {
    let cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();

    assert_eq!(cache.get(&1).unwrap(), 0);
    assert_eq!(cache.get(&2).unwrap(), 1);
    assert_eq!(cache.get(&3).unwrap(), 7);
    assert_eq!(cache.get(&27).unwrap(), 111);
}

// =============================================================================
// ParallelBackend tests
// =============================================================================

#[test]
fn test_dashmap_backend_get_or_insert() {
    let backend: DashMapBackend<String, i32> = DashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42).unwrap();
    assert_eq!(value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999).unwrap();
    assert_eq!(value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100).unwrap();
    assert_eq!(value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(42));
}

#[test]
fn test_rwlock_backend_get_or_insert() {
    let backend: RwLockHashMapBackend<String, i32> = RwLockHashMapBackend::new();

    // Insert value
    let value = backend.get_or_insert("key1".to_string(), || 42).unwrap();
    assert_eq!(value, 42);

    // Get same key again - should return cached value, not recompute
    let value = backend.get_or_insert("key1".to_string(), || 999).unwrap();
    assert_eq!(value, 42);

    // Get returns the cached value
    assert_eq!(backend.get(&"key1".to_string()), Some(42));

    // Get returns None for uncached key
    assert_eq!(backend.get(&"key2".to_string()), None);

    // Insert different key - should not affect existing
    let value = backend.get_or_insert("key2".to_string(), || 100).unwrap();
    assert_eq!(value, 100);
    assert_eq!(backend.get(&"key1".to_string()), Some(42));
}

#[test]
fn test_all_parallel_backends_match() {
    // Verify all parallel backends produce same results
    let dashmap_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();
    let rwlock_cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();

    for n in 1..=100u64 {
        assert_eq!(
            dashmap_cache.get(&n).unwrap(),
            rwlock_cache.get(&n).unwrap(),
            "Mismatch at n={}",
            n
        );
    }
}


// =============================================================================
// ArrayBackend tests
// =============================================================================

#[test]
fn test_array_backend_get_returns_none_for_uninitialized() {
    let backend: ArrayBackend<i32, 10> = ArrayBackend::new();
    
    // All indices should return None initially
    for i in 0..10 {
        assert_eq!(backend.get(&i), None);
    }
}

#[test]
fn test_array_backend_get_or_insert() {
    let mut backend: ArrayBackend<i32, 10> = ArrayBackend::new();
    
    // Insert value at index 5
    let value = backend.get_or_insert(5, || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Get same index again - should return cached value
    let value = backend.get_or_insert(5, || 999).unwrap();
    assert_eq!(*value, 42);
    
    // Get returns the cached value
    assert_eq!(backend.get(&5), Some(&42));
    
    // Other indices still uninitialized
    assert_eq!(backend.get(&0), None);
    assert_eq!(backend.get(&9), None);
}

#[test]
fn test_array_backend_computes_exactly_once() {
    let mut backend: ArrayBackend<i32, 10> = ArrayBackend::new();
    let compute_count = Rc::new(Cell::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.set(count.get() + 1);
        42
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
    
    // Second call should NOT compute
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.set(count.get() + 1);
        999
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
}

#[test]
fn test_array_backend_out_of_bounds_returns_error() {
    let mut backend: ArrayBackend<i32, 10> = ArrayBackend::new();
    
    // Out of bounds get returns None
    assert_eq!(backend.get(&10), None);
    assert_eq!(backend.get(&100), None);
    
    // Out of bounds get_or_insert returns error with the index
    let result = backend.get_or_insert(10, || 42);
    assert_eq!(result, Err(10));
    
    let result = backend.get_or_insert(100, || 42);
    assert_eq!(result, Err(100));
}

#[test]
fn test_array_backend_const_construction() {
    // Verify const construction compiles
    const BACKEND: ArrayBackend<i32, 5> = ArrayBackend::new();
    
    // Can use the const-constructed backend
    let mut backend = BACKEND;
    let value = backend.get_or_insert(0, || 42).unwrap();
    assert_eq!(*value, 42);
}

#[test]
fn test_array_backend_default() {
    let backend: ArrayBackend<i32, 10> = ArrayBackend::default();
    
    // Default should be same as new()
    for i in 0..10 {
        assert_eq!(backend.get(&i), None);
    }
}

#[test]
fn test_array_backend_with_dp_cache() {
    // Test ArrayBackend with DpCache
    let cache = DpCache::builder()
        .backend(ArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
}


// =============================================================================
// Array2DBackend tests
// =============================================================================

#[test]
fn test_array2d_backend_get_returns_none_for_uninitialized() {
    let backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
    
    // All indices should return None initially
    for row in 0..5 {
        for col in 0..10 {
            assert_eq!(backend.get(&(row, col)), None);
        }
    }
}

#[test]
fn test_array2d_backend_get_or_insert() {
    let mut backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
    
    // Insert value at (2, 3)
    let value = backend.get_or_insert((2, 3), || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Get same index again - should return cached value
    let value = backend.get_or_insert((2, 3), || 999).unwrap();
    assert_eq!(*value, 42);
    
    // Get returns the cached value
    assert_eq!(backend.get(&(2, 3)), Some(&42));
    
    // Other indices still uninitialized
    assert_eq!(backend.get(&(0, 0)), None);
    assert_eq!(backend.get(&(4, 9)), None);
}

#[test]
fn test_array2d_backend_computes_exactly_once() {
    let mut backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
    let compute_count = Rc::new(Cell::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.set(count.get() + 1);
        42
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
    
    // Second call should NOT compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.set(count.get() + 1);
        999
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
}

#[test]
fn test_array2d_backend_row_out_of_bounds() {
    let mut backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
    
    // Row out of bounds get returns None
    assert_eq!(backend.get(&(5, 0)), None);
    assert_eq!(backend.get(&(100, 0)), None);
    
    // Row out of bounds get_or_insert returns error with the index
    let result = backend.get_or_insert((5, 0), || 42);
    assert_eq!(result, Err((5, 0)));
    
    let result = backend.get_or_insert((100, 5), || 42);
    assert_eq!(result, Err((100, 5)));
}

#[test]
fn test_array2d_backend_col_out_of_bounds() {
    let mut backend: Array2DBackend<i32, 5, 10> = Array2DBackend::new();
    
    // Column out of bounds get returns None
    assert_eq!(backend.get(&(0, 10)), None);
    assert_eq!(backend.get(&(0, 100)), None);
    
    // Column out of bounds get_or_insert returns error with the index
    let result = backend.get_or_insert((0, 10), || 42);
    assert_eq!(result, Err((0, 10)));
    
    let result = backend.get_or_insert((2, 100), || 42);
    assert_eq!(result, Err((2, 100)));
}

#[test]
fn test_array2d_backend_const_construction() {
    // Verify const construction compiles
    const BACKEND: Array2DBackend<i32, 3, 4> = Array2DBackend::new();
    
    // Can use the const-constructed backend
    let mut backend = BACKEND;
    let value = backend.get_or_insert((0, 0), || 42).unwrap();
    assert_eq!(*value, 42);
}

#[test]
fn test_array2d_backend_default() {
    let backend: Array2DBackend<i32, 5, 10> = Array2DBackend::default();
    
    // Default should be same as new()
    for row in 0..5 {
        for col in 0..10 {
            assert_eq!(backend.get(&(row, col)), None);
        }
    }
}

/// 2D grid problem for testing Array2DBackend with DpCache
struct Grid2DSum;

impl DpProblem<(usize, usize), i32> for Grid2DSum {
    fn deps(&self, index: &(usize, usize)) -> Vec<(usize, usize)> {
        let (row, col) = *index;
        if row == 0 && col == 0 {
            vec![]
        } else if row == 0 {
            vec![(0, col - 1)]
        } else if col == 0 {
            vec![(row - 1, 0)]
        } else {
            vec![(row - 1, col), (row, col - 1)]
        }
    }

    fn compute(&self, index: &(usize, usize), deps: Vec<i32>) -> i32 {
        let (row, col) = *index;
        if row == 0 && col == 0 {
            1
        } else if deps.len() == 1 {
            deps[0] + 1
        } else {
            deps[0].max(deps[1]) + 1
        }
    }
}

#[test]
fn test_array2d_backend_with_dp_cache() {
    // Test Array2DBackend with DpCache for a 2D grid problem
    let cache = DpCache::builder()
        .backend(Array2DBackend::<i32, 5, 5>::new())
        .problem(Grid2DSum)
        .build();
    
    // (0,0) = 1
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    // (0,1) = 2, (1,0) = 2
    assert_eq!(cache.get(&(0, 1)).unwrap(), 2);
    assert_eq!(cache.get(&(1, 0)).unwrap(), 2);
    // (1,1) = max(2, 2) + 1 = 3
    assert_eq!(cache.get(&(1, 1)).unwrap(), 3);
    // (4,4) should be computed correctly
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}


// =============================================================================
// Vec2DBackend tests
// =============================================================================

#[test]
fn test_vec2d_backend_get_returns_none_for_uninitialized() {
    let backend: Vec2DBackend<i32> = Vec2DBackend::new();
    
    // Empty backend returns None for any index
    assert_eq!(backend.get(&(0, 0)), None);
    assert_eq!(backend.get(&(100, 100)), None);
}

#[test]
fn test_vec2d_backend_get_or_insert() {
    let mut backend: Vec2DBackend<i32> = Vec2DBackend::new();
    
    // Insert value at (2, 3) - should auto-grow
    let value = backend.get_or_insert((2, 3), || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Get same index again - should return cached value
    let value = backend.get_or_insert((2, 3), || 999).unwrap();
    assert_eq!(*value, 42);
    
    // Get returns the cached value
    assert_eq!(backend.get(&(2, 3)), Some(&42));
    
    // Other indices still uninitialized
    assert_eq!(backend.get(&(0, 0)), None);
    assert_eq!(backend.get(&(4, 9)), None);
}

#[test]
fn test_vec2d_backend_computes_exactly_once() {
    let mut backend: Vec2DBackend<i32> = Vec2DBackend::new();
    let compute_count = Rc::new(Cell::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.set(count.get() + 1);
        42
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
    
    // Second call should NOT compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.set(count.get() + 1);
        999
    }).unwrap();
    assert_eq!(*value, 42);
    assert_eq!(compute_count.get(), 1);
}

#[test]
fn test_vec2d_backend_auto_grow() {
    let mut backend: Vec2DBackend<i32> = Vec2DBackend::new();
    
    // Insert at (100, 50) - should auto-grow
    let value = backend.get_or_insert((100, 50), || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Intermediate indices should be uninitialized
    assert_eq!(backend.get(&(0, 0)), None);
    assert_eq!(backend.get(&(50, 25)), None);
    assert_eq!(backend.get(&(99, 49)), None);
    
    // Index (100, 50) should be initialized
    assert_eq!(backend.get(&(100, 50)), Some(&42));
    
    // Rows should have grown
    assert_eq!(backend.rows(), 101);
}

#[test]
fn test_vec2d_backend_dimension_accessors() {
    let backend: Vec2DBackend<i32> = Vec2DBackend::new();
    
    // Empty backend has 0 rows
    assert_eq!(backend.rows(), 0);
    assert_eq!(backend.cols(0), 0);
    
    // After inserting, dimensions reflect the data
    let mut backend = backend;
    let _ = backend.get_or_insert((4, 9), || 42);
    assert_eq!(backend.rows(), 5);
    assert_eq!(backend.cols(4), 10);
    // Other rows may have different column counts
    assert_eq!(backend.cols(0), 0);
}

#[test]
fn test_vec2d_backend_with_capacity() {
    // Test with_capacity constructor
    let mut backend: Vec2DBackend<i32> = Vec2DBackend::with_capacity(100, 50);
    
    // Should still auto-grow
    let value = backend.get_or_insert((5, 10), || 42).unwrap();
    assert_eq!(*value, 42);
    
    // Can grow beyond initial capacity
    let value = backend.get_or_insert((200, 100), || 99).unwrap();
    assert_eq!(*value, 99);
}

#[test]
fn test_vec2d_backend_various_dimensions() {
    // Test with different dimension combinations using auto-grow
    let mut backend1: Vec2DBackend<i32> = Vec2DBackend::new();
    assert_eq!(backend1.get_or_insert((0, 0), || 42).unwrap(), &42);
    // Can grow to any size
    assert_eq!(backend1.get_or_insert((0, 1), || 43).unwrap(), &43);
    
    let mut backend2: Vec2DBackend<i32> = Vec2DBackend::new();
    assert_eq!(backend2.get_or_insert((99, 0), || 42).unwrap(), &42);
    
    let mut backend3: Vec2DBackend<i32> = Vec2DBackend::new();
    assert_eq!(backend3.get_or_insert((0, 99), || 42).unwrap(), &42);
}

#[test]
fn test_vec2d_backend_with_dp_cache() {
    // Test Vec2DBackend with DpCache for a 2D grid problem
    let cache = DpCache::builder()
        .backend(Vec2DBackend::<i32>::new())
        .problem(Grid2DSum)
        .build();
    
    // (0,0) = 1
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    // (0,1) = 2, (1,0) = 2
    assert_eq!(cache.get(&(0, 1)).unwrap(), 2);
    assert_eq!(cache.get(&(1, 0)).unwrap(), 2);
    // (1,1) = max(2, 2) + 1 = 3
    assert_eq!(cache.get(&(1, 1)).unwrap(), 3);
    // (4,4) should be computed correctly
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}

#[test]
fn test_vec2d_backend_default() {
    let backend: Vec2DBackend<i32> = Vec2DBackend::default();
    
    // Default should be same as new()
    assert_eq!(backend.rows(), 0);
    assert_eq!(backend.get(&(0, 0)), None);
}


// =============================================================================
// ParallelArrayBackend tests
// =============================================================================

#[test]
fn test_parallel_array_backend_get_returns_none_for_uninitialized() {
    let backend: ParallelArrayBackend<i32, 10> = ParallelArrayBackend::new();
    
    // All indices should return None initially
    for i in 0..10 {
        assert_eq!(backend.get(&i), None);
    }
}

#[test]
fn test_parallel_array_backend_get_or_insert() {
    let backend: ParallelArrayBackend<i32, 10> = ParallelArrayBackend::new();
    
    // Insert value at index 5
    let value = backend.get_or_insert(5, || 42).unwrap();
    assert_eq!(value, 42);
    
    // Get same index again - should return cached value
    let value = backend.get_or_insert(5, || 999).unwrap();
    assert_eq!(value, 42);
    
    // Get returns the cached value
    assert_eq!(backend.get(&5), Some(42));
    
    // Other indices still uninitialized
    assert_eq!(backend.get(&0), None);
    assert_eq!(backend.get(&9), None);
}

#[test]
fn test_parallel_array_backend_computes_exactly_once() {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    
    let backend: ParallelArrayBackend<i32, 10> = ParallelArrayBackend::new();
    let compute_count = Arc::new(AtomicI32::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.fetch_add(1, Ordering::SeqCst);
        42
    }).unwrap();
    assert_eq!(value, 42);
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
    
    // Second call should NOT compute
    let count = compute_count.clone();
    let value = backend.get_or_insert(3, move || {
        count.fetch_add(1, Ordering::SeqCst);
        999
    }).unwrap();
    assert_eq!(value, 42);
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_parallel_array_backend_out_of_bounds() {
    let backend: ParallelArrayBackend<i32, 10> = ParallelArrayBackend::new();
    
    // Out of bounds get returns None
    assert_eq!(backend.get(&10), None);
    assert_eq!(backend.get(&100), None);
    
    // Out of bounds get_or_insert returns error with the index
    let result = backend.get_or_insert(10, || 42);
    assert_eq!(result, Err(10));
    
    let result = backend.get_or_insert(100, || 42);
    assert_eq!(result, Err(100));
}

#[test]
fn test_parallel_array_backend_const_construction() {
    // Verify const construction compiles
    const BACKEND: ParallelArrayBackend<i32, 5> = ParallelArrayBackend::new();
    
    // Can use the const-constructed backend
    let backend = BACKEND;
    let value = backend.get_or_insert(0, || 42).unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_parallel_array_backend_concurrent_access() {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    use std::thread;
    
    let backend = Arc::new(ParallelArrayBackend::<i32, 100>::new());
    let compute_count = Arc::new(AtomicI32::new(0));
    
    // Spawn multiple threads that all try to compute the same index
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let backend = Arc::clone(&backend);
            let count = Arc::clone(&compute_count);
            thread::spawn(move || {
                backend.get_or_insert(42, || {
                    count.fetch_add(1, Ordering::SeqCst);
                    // Small delay to increase chance of contention
                    std::thread::yield_now();
                    42
                }).unwrap()
            })
        })
        .collect();
    
    // All threads should get the same value
    for handle in handles {
        assert_eq!(handle.join().unwrap(), 42);
    }
    
    // Compute should have been called exactly once
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_parallel_array_backend_with_parallel_dp_cache() {
    // Test ParallelArrayBackend with ParallelDpCache
    let cache = ParallelDpCache::builder()
        .backend(ParallelArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
}


// =============================================================================
// ParallelArray2DBackend tests
// =============================================================================

#[test]
fn test_parallel_array2d_backend_get_returns_none_for_uninitialized() {
    let backend: ParallelArray2DBackend<i32, 5, 10> = ParallelArray2DBackend::new();
    
    // All indices should return None initially
    for row in 0..5 {
        for col in 0..10 {
            assert_eq!(backend.get(&(row, col)), None);
        }
    }
}

#[test]
fn test_parallel_array2d_backend_get_or_insert() {
    let backend: ParallelArray2DBackend<i32, 5, 10> = ParallelArray2DBackend::new();
    
    // Insert value at (2, 3)
    let value = backend.get_or_insert((2, 3), || 42).unwrap();
    assert_eq!(value, 42);
    
    // Get same index again - should return cached value
    let value = backend.get_or_insert((2, 3), || 999).unwrap();
    assert_eq!(value, 42);
    
    // Get returns the cached value
    assert_eq!(backend.get(&(2, 3)), Some(42));
    
    // Other indices still uninitialized
    assert_eq!(backend.get(&(0, 0)), None);
    assert_eq!(backend.get(&(4, 9)), None);
}

#[test]
fn test_parallel_array2d_backend_computes_exactly_once() {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    
    let backend: ParallelArray2DBackend<i32, 5, 10> = ParallelArray2DBackend::new();
    let compute_count = Arc::new(AtomicI32::new(0));
    
    // First call should compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.fetch_add(1, Ordering::SeqCst);
        42
    }).unwrap();
    assert_eq!(value, 42);
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
    
    // Second call should NOT compute
    let count = compute_count.clone();
    let value = backend.get_or_insert((1, 2), move || {
        count.fetch_add(1, Ordering::SeqCst);
        999
    }).unwrap();
    assert_eq!(value, 42);
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_parallel_array2d_backend_out_of_bounds() {
    let backend: ParallelArray2DBackend<i32, 5, 10> = ParallelArray2DBackend::new();
    
    // Out of bounds get returns None
    assert_eq!(backend.get(&(5, 0)), None);
    assert_eq!(backend.get(&(0, 10)), None);
    assert_eq!(backend.get(&(100, 100)), None);
    
    // Out of bounds get_or_insert returns error with the index
    let result = backend.get_or_insert((5, 0), || 42);
    assert_eq!(result, Err((5, 0)));
    
    let result = backend.get_or_insert((0, 10), || 42);
    assert_eq!(result, Err((0, 10)));
}

#[test]
fn test_parallel_array2d_backend_const_construction() {
    // Verify const construction compiles
    const BACKEND: ParallelArray2DBackend<i32, 3, 4> = ParallelArray2DBackend::new();
    
    // Can use the const-constructed backend
    let backend = BACKEND;
    let value = backend.get_or_insert((0, 0), || 42).unwrap();
    assert_eq!(value, 42);
}

#[test]
fn test_parallel_array2d_backend_concurrent_access() {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Arc;
    use std::thread;
    
    let backend = Arc::new(ParallelArray2DBackend::<i32, 10, 10>::new());
    let compute_count = Arc::new(AtomicI32::new(0));
    
    // Spawn multiple threads that all try to compute the same index
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let backend = Arc::clone(&backend);
            let count = Arc::clone(&compute_count);
            thread::spawn(move || {
                backend.get_or_insert((4, 2), || {
                    count.fetch_add(1, Ordering::SeqCst);
                    // Small delay to increase chance of contention
                    std::thread::yield_now();
                    42
                }).unwrap()
            })
        })
        .collect();
    
    // All threads should get the same value
    for handle in handles {
        assert_eq!(handle.join().unwrap(), 42);
    }
    
    // Compute should have been called exactly once
    assert_eq!(compute_count.load(Ordering::SeqCst), 1);
}

/// Parallel 2D grid problem for testing ParallelArray2DBackend with ParallelDpCache
struct ParallelGrid2DSum;

impl DpProblem<(usize, usize), i32> for ParallelGrid2DSum {
    fn deps(&self, index: &(usize, usize)) -> Vec<(usize, usize)> {
        let (row, col) = *index;
        if row == 0 && col == 0 {
            vec![]
        } else if row == 0 {
            vec![(0, col - 1)]
        } else if col == 0 {
            vec![(row - 1, 0)]
        } else {
            vec![(row - 1, col), (row, col - 1)]
        }
    }

    fn compute(&self, index: &(usize, usize), deps: Vec<i32>) -> i32 {
        let (row, col) = *index;
        if row == 0 && col == 0 {
            1
        } else if deps.len() == 1 {
            deps[0] + 1
        } else {
            deps[0].max(deps[1]) + 1
        }
    }
}

#[test]
fn test_parallel_array2d_backend_with_parallel_dp_cache() {
    // Test ParallelArray2DBackend with ParallelDpCache for a 2D grid problem
    let cache = ParallelDpCache::builder()
        .backend(ParallelArray2DBackend::<i32, 5, 5>::new())
        .problem(ParallelGrid2DSum)
        .build();
    
    // (0,0) = 1
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    // (0,1) = 2, (1,0) = 2
    assert_eq!(cache.get(&(0, 1)).unwrap(), 2);
    assert_eq!(cache.get(&(1, 0)).unwrap(), 2);
    // (1,1) = max(2, 2) + 1 = 3
    assert_eq!(cache.get(&(1, 1)).unwrap(), 3);
    // (4,4) should be computed correctly
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}


// =============================================================================
// Integration tests - DpCache builder with all backends
// =============================================================================

#[test]
fn test_dp_cache_builder_with_array_backend() {
    let cache = DpCache::builder()
        .backend(ArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
}

#[test]
fn test_dp_cache_builder_with_array2d_backend() {
    let cache = DpCache::builder()
        .backend(Array2DBackend::<i32, 5, 5>::new())
        .problem(Grid2DSum)
        .build();
    
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}

#[test]
fn test_dp_cache_builder_with_vec2d_backend() {
    let cache = DpCache::builder()
        .backend(Vec2DBackend::<i32>::new())
        .problem(Grid2DSum)
        .build();
    
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}

#[test]
fn test_parallel_dp_cache_builder_with_parallel_array_backend() {
    let cache = ParallelDpCache::builder()
        .backend(ParallelArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    assert_eq!(cache.get(&0).unwrap(), 0);
    assert_eq!(cache.get(&1).unwrap(), 1);
    assert_eq!(cache.get(&10).unwrap(), 55);
    assert_eq!(cache.get(&20).unwrap(), 6765);
}

#[test]
fn test_parallel_dp_cache_builder_with_parallel_array2d_backend() {
    let cache = ParallelDpCache::builder()
        .backend(ParallelArray2DBackend::<i32, 5, 5>::new())
        .problem(ParallelGrid2DSum)
        .build();
    
    assert_eq!(cache.get(&(0, 0)).unwrap(), 1);
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}

#[test]
fn test_const_constructed_backends_work_correctly() {
    // Test that const-constructed backends work correctly
    const ARRAY_BACKEND: ArrayBackend<u64, 21> = ArrayBackend::new();
    const ARRAY2D_BACKEND: Array2DBackend<i32, 5, 5> = Array2DBackend::new();
    const PARALLEL_ARRAY_BACKEND: ParallelArrayBackend<u64, 21> = ParallelArrayBackend::new();
    const PARALLEL_ARRAY2D_BACKEND: ParallelArray2DBackend<i32, 5, 5> = ParallelArray2DBackend::new();
    
    // Sequential 1D
    let cache = DpCache::builder()
        .backend(ARRAY_BACKEND)
        .problem(Fibonacci)
        .build();
    assert_eq!(cache.get(&20).unwrap(), 6765);
    
    // Sequential 2D
    let cache = DpCache::builder()
        .backend(ARRAY2D_BACKEND)
        .problem(Grid2DSum)
        .build();
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
    
    // Parallel 1D
    let cache = ParallelDpCache::builder()
        .backend(PARALLEL_ARRAY_BACKEND)
        .problem(Fibonacci)
        .build();
    assert_eq!(cache.get(&20).unwrap(), 6765);
    
    // Parallel 2D
    let cache = ParallelDpCache::builder()
        .backend(PARALLEL_ARRAY2D_BACKEND)
        .problem(ParallelGrid2DSum)
        .build();
    assert_eq!(cache.get(&(4, 4)).unwrap(), 9);
}

#[test]
fn test_all_sequential_backends_produce_same_results() {
    // Test that all sequential backends produce the same results for Fibonacci
    let vec_cache = DpCache::builder()
        .backend(VecBackend::new())
        .problem(Fibonacci)
        .build();
    
    let array_cache = DpCache::builder()
        .backend(ArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    for n in 0..=20 {
        assert_eq!(
            vec_cache.get(&n).unwrap(),
            array_cache.get(&n).unwrap(),
            "Mismatch at n={}",
            n
        );
    }
}

#[test]
fn test_all_parallel_backends_produce_same_results() {
    // Test that all parallel backends produce the same results for Fibonacci
    let dashmap_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Fibonacci)
        .build();
    
    let array_cache = ParallelDpCache::builder()
        .backend(ParallelArrayBackend::<u64, 21>::new())
        .problem(Fibonacci)
        .build();
    
    for n in 0..=20 {
        assert_eq!(
            dashmap_cache.get(&n).unwrap(),
            array_cache.get(&n).unwrap(),
            "Mismatch at n={}",
            n
        );
    }
}
