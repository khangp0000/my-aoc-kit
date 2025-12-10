//! Benchmark comparing DP cache backends using Fibonacci computation.
//!
//! Run with: cargo run --example fibonacci_benchmark --release
//!
//! Computes Fibonacci numbers for various n values and compares:
//! - No cache (direct recursive computation) - baseline
//! - Various cached backends (Array, Vec, HashMap, DashMap, RwLock)
//!
//! This is similar to the pattern benchmark but with a classic DP problem.

use aoc_solutions::utils::dp_cache::{
    ArrayBackend, DashMapBackend, DpCache, DpProblem, HashMapBackend, NoCacheBackend,
    ParallelArrayBackend, ParallelDpCache, ParallelNoCacheBackend, RwLockHashMapBackend,
    VecBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Fibonacci Problem Definition
// =============================================================================

/// Fibonacci problem using the trait-based API
struct Fibonacci;

impl DpProblem<usize, u128> for Fibonacci {
    fn deps(&self, n: &usize) -> Vec<usize> {
        if *n <= 1 {
            vec![]
        } else {
            vec![n - 1, n - 2]
        }
    }

    fn compute(&self, n: &usize, deps: Vec<u128>) -> u128 {
        if *n == 0 {
            0
        } else if *n == 1 {
            1
        } else {
            deps[0] + deps[1]
        }
    }
}

/// Direct recursive computation for verification (exponentially slow)
fn fib_direct(n: usize) -> u128 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fib_direct(n - 1) + fib_direct(n - 2)
    }
}

/// Iterative computation for verification (fast)
fn fib_iterative(n: usize) -> u128 {
    if n == 0 {
        return 0;
    }
    if n == 1 {
        return 1;
    }
    let mut a: u128 = 0;
    let mut b: u128 = 1;
    for _ in 2..=n {
        let c = a + b;
        a = b;
        b = c;
    }
    b
}

const MAX_N: usize = 186; // Max n before u128 overflow
const NUM_QUERIES: usize = 1000;

fn main() {
    println!("Fibonacci DP Cache Benchmark");
    println!("============================");
    println!("Computing Fibonacci numbers for n in 0..{}\n", MAX_N);

    // Generate test cases - compute fib(n) for various n values
    let test_cases: Vec<usize> = (0..NUM_QUERIES).map(|i| i % MAX_N).collect();

    println!("Testing {} queries (n: 0-{})\n", test_cases.len(), MAX_N - 1);

    // =========================================================================
    // Iterative baseline (optimal solution)
    // =========================================================================
    println!("=== Iterative baseline (optimal) ===");

    // Iterative computation - the optimal O(n) solution
    println!("Running iterative computation (full range)...");
    let start = Instant::now();
    let iterative_results: Vec<u128> = test_cases.iter().map(|&n| fib_iterative(n)).collect();
    let iterative_time = start.elapsed();
    println!("Iterative (sequential):      {:?}", iterative_time);

    // Iterative + par_iter
    println!("Running iterative + par_iter...");
    let start = Instant::now();
    let iterative_par_results: Vec<u128> =
        test_cases.par_iter().map(|&n| fib_iterative(n)).collect();
    let iterative_par_time = start.elapsed();
    println!("Iterative + par_iter:        {:?}", iterative_par_time);

    // =========================================================================
    // Single-threaded benchmarks
    // =========================================================================
    println!("\n=== Single-threaded ===");

    // No cache - direct computation (baseline) - only for small n
    println!("Running no cache (direct recursive) on small n...");
    let small_test_cases: Vec<usize> = (0..100).map(|i| i % 30).collect(); // Only up to 30
    let start = Instant::now();
    let no_cache_results: Vec<u128> = small_test_cases.iter().map(|&n| fib_direct(n)).collect();
    let no_cache_time = start.elapsed();
    println!("No cache (n<=30):            {:?}", no_cache_time);

    // NoCacheBackend with DpCache (measures DpCache wrapper overhead)
    println!("Running NoCacheBackend (DpCache wrapper overhead)...");
    let start = Instant::now();
    let nocache_backend_results: Vec<u128> = small_test_cases
        .iter()
        .map(|&n| {
            let cache = DpCache::builder()
                .backend(NoCacheBackend::<usize, u128>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let nocache_backend_time = start.elapsed();
    println!("NoCacheBackend (wrapper):    {:?}", nocache_backend_time);

    // ArrayBackend with DpCache (sequential) - zero allocation
    println!("Running ArrayBackend (sequential, zero-alloc)...");
    let start = Instant::now();
    let array_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = DpCache::builder()
                .backend(ArrayBackend::<u128, MAX_N>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let array_time = start.elapsed();
    println!("ArrayBackend (sequential):   {:?}", array_time);

    // VecBackend with DpCache (sequential)
    println!("Running VecBackend (sequential)...");
    let start = Instant::now();
    let vec_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = DpCache::builder()
                .backend(VecBackend::with_capacity(n + 1))
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let vec_time = start.elapsed();
    println!("VecBackend (sequential):     {:?}", vec_time);

    // HashMapBackend with DpCache (sequential)
    println!("Running HashMapBackend (sequential)...");
    let start = Instant::now();
    let hashmap_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = DpCache::builder()
                .backend(HashMapBackend::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential): {:?}", hashmap_time);

    // =========================================================================
    // Parallel backends (sequential iteration)
    // =========================================================================
    println!("\n=== Parallel backends (sequential iteration) ===");

    // ParallelNoCacheBackend with ParallelDpCache (measures parallel wrapper overhead)
    println!("Running ParallelNoCacheBackend (parallel wrapper overhead)...");
    let start = Instant::now();
    let par_nocache_backend_results: Vec<u128> = small_test_cases
        .iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<usize, u128>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let par_nocache_backend_time = start.elapsed();
    println!("ParallelNoCacheBackend:      {:?}", par_nocache_backend_time);

    // ParallelArrayBackend with ParallelDpCache (sequential iteration)
    println!("Running ParallelArrayBackend (parallel, zero-alloc)...");
    let start = Instant::now();
    let par_array_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<u128, MAX_N>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let par_array_time = start.elapsed();
    println!("ParallelArrayBackend:        {:?}", par_array_time);

    // DashMapBackend with ParallelDpCache
    println!("Running DashMapBackend (parallel)...");
    let start = Instant::now();
    let dashmap_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):   {:?}", dashmap_time);

    // RwLockHashMapBackend with ParallelDpCache
    println!("Running RwLockHashMapBackend (parallel)...");
    let start = Instant::now();
    let rwlock_results: Vec<u128> = test_cases
        .iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend:        {:?}", rwlock_time);

    // =========================================================================
    // Parallel iteration (par_iter)
    // =========================================================================
    println!("\n=== Parallel iteration (par_iter) ===");

    // No cache + parallel iteration (small n only)
    println!("Running no cache + par_iter (small n)...");
    let start = Instant::now();
    let no_cache_par_results: Vec<u128> = small_test_cases
        .par_iter()
        .map(|&n| fib_direct(n))
        .collect();
    let no_cache_par_time = start.elapsed();
    println!("No cache + par_iter:         {:?}", no_cache_par_time);

    // ParallelNoCacheBackend + par_iter
    println!("Running ParallelNoCacheBackend + par_iter...");
    let start = Instant::now();
    let par_nocache_par_results: Vec<u128> = small_test_cases
        .par_iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<usize, u128>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let par_nocache_par_time = start.elapsed();
    println!("ParallelNoCacheBackend+par:  {:?}", par_nocache_par_time);

    // ParallelArrayBackend + par_iter
    println!("Running ParallelArrayBackend + par_iter...");
    let start = Instant::now();
    let par_array_par_results: Vec<u128> = test_cases
        .par_iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<u128, MAX_N>::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let par_array_par_time = start.elapsed();
    println!("ParallelArrayBackend + par:  {:?}", par_array_par_time);

    // DashMapBackend + par_iter
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<u128> = test_cases
        .par_iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // RwLockHashMapBackend + par_iter
    println!("Running RwLockHashMapBackend + par_iter...");
    let start = Instant::now();
    let rwlock_par_results: Vec<u128> = test_cases
        .par_iter()
        .map(|&n| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Fibonacci)
                .build();
            cache.get(&n).unwrap()
        })
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par:  {:?}", rwlock_par_time);

    // =========================================================================
    // Verification
    // =========================================================================
    println!("\nVerifying against iterative computation...");

    let mut all_match = true;
    let mut mismatches = 0;

    // Verify small test cases (no cache results)
    for (idx, &n) in small_test_cases.iter().enumerate() {
        let expected = fib_iterative(n);
        if no_cache_results[idx] != expected
            || nocache_backend_results[idx] != expected
            || par_nocache_backend_results[idx] != expected
            || no_cache_par_results[idx] != expected
            || par_nocache_par_results[idx] != expected
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at n={}: expected={}, no_cache={}, nocache_backend={}",
                    n, expected, no_cache_results[idx], nocache_backend_results[idx]
                );
            }
            all_match = false;
            mismatches += 1;
        }
    }

    // Verify full test cases
    for (idx, &n) in test_cases.iter().enumerate() {
        let expected = fib_iterative(n);
        if iterative_results[idx] != expected
            || iterative_par_results[idx] != expected
            || array_results[idx] != expected
            || vec_results[idx] != expected
            || hashmap_results[idx] != expected
            || par_array_results[idx] != expected
            || dashmap_results[idx] != expected
            || rwlock_results[idx] != expected
            || par_array_par_results[idx] != expected
            || dashmap_par_results[idx] != expected
            || rwlock_par_results[idx] != expected
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at n={}: expected={}, array={}, hashmap={}",
                    n, expected, array_results[idx], hashmap_results[idx]
                );
            }
            all_match = false;
            mismatches += 1;
        }
    }

    if all_match {
        println!("✓ All backends produce identical results!");
    } else {
        println!("✗ {} mismatches found!", mismatches);
    }

    // Sample results
    println!("\nSample results:");
    for n in [0, 1, 10, 50, 100, 185] {
        println!("  fib({}) = {}", n, fib_iterative(n));
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Performance Summary ===");

    println!("\nIterative baseline (optimal O(n) solution):");
    println!("  Iterative (sequential):     {:?}", iterative_time);
    println!("  Iterative + par_iter:       {:?}", iterative_par_time);

    println!("\nFull range (n: 0-{}) - Single-threaded:", MAX_N - 1);
    println!("  ArrayBackend:               {:?}", array_time);
    println!("  VecBackend:                 {:?}", vec_time);
    println!("  HashMapBackend:             {:?}", hashmap_time);

    println!("\nFull range - Parallel backends (sequential iteration):");
    println!("  ParallelArrayBackend:       {:?}", par_array_time);
    println!("  DashMapBackend:             {:?}", dashmap_time);
    println!("  RwLockHashMapBackend:       {:?}", rwlock_time);

    println!("\nFull range - Parallel iteration (par_iter):");
    println!("  ParallelArrayBackend:       {:?}", par_array_par_time);
    println!("  DashMapBackend:             {:?}", dashmap_par_time);
    println!("  RwLockHashMapBackend:       {:?}", rwlock_par_time);

    println!("\nSmall n (n <= 30) - Wrapper overhead:");
    println!("  No cache (direct):          {:?}", no_cache_time);
    println!("  NoCacheBackend (wrapper):   {:?}", nocache_backend_time);
    println!("  ParallelNoCacheBackend:     {:?}", par_nocache_backend_time);
    println!("  No cache + par_iter:        {:?}", no_cache_par_time);
    println!("  ParallelNoCacheBackend+par: {:?}", par_nocache_par_time);

    // Backend comparison
    let best_cached = array_time
        .min(vec_time)
        .min(hashmap_time)
        .min(par_array_time)
        .min(par_array_par_time)
        .min(dashmap_time)
        .min(dashmap_par_time)
        .min(rwlock_time)
        .min(rwlock_par_time);

    println!("\nBackend comparison:");
    println!(
        "  ArrayBackend vs VecBackend: {:.2}x",
        vec_time.as_secs_f64() / array_time.as_secs_f64()
    );
    println!(
        "  ArrayBackend vs HashMapBackend: {:.2}x",
        hashmap_time.as_secs_f64() / array_time.as_secs_f64()
    );
    println!("  Best cached time: {:?}", best_cached);

    println!("\nIterative vs DP Cache:");
    println!(
        "  ArrayBackend vs Iterative: {:.2}x slower",
        array_time.as_secs_f64() / iterative_time.as_secs_f64()
    );
    println!(
        "  Best cached vs Iterative: {:.2}x slower",
        best_cached.as_secs_f64() / iterative_time.as_secs_f64()
    );

    println!("\nWrapper overhead (small n):");
    println!(
        "  NoCacheBackend vs direct: {:.2}x",
        nocache_backend_time.as_secs_f64() / no_cache_time.as_secs_f64()
    );
    println!(
        "  ParallelNoCacheBackend vs direct: {:.2}x",
        par_nocache_backend_time.as_secs_f64() / no_cache_time.as_secs_f64()
    );

    println!("\n=== Conclusion ===");
    println!("For Fibonacci, iterative is {:.0}x faster than the best DP cache.", 
             best_cached.as_secs_f64() / iterative_time.as_secs_f64());
    println!("DP cache is useful when:");
    println!("  - No simple iterative solution exists");
    println!("  - Subproblems have complex dependencies (not just n-1, n-2)");
    println!("  - You need memoization across multiple queries");
}
