//! Benchmark comparing DP cache backends using a decimal pattern computation.
//!
//! Run with: cargo run --example pattern_benchmark --release
//!
//! Given i (zeros between 1s) and j (number of 1s), compute the u64 value of
//! the decimal pattern: 100...100...1 where there are i zeros between each of j ones.
//!
//! Examples:
//! - i=2, j=3 → 1001001 (decimal)
//! - i=0, j=3 → 111 (decimal)
//! - i=1, j=4 → 1010101 (decimal)
//!
//! Returns None if the result would overflow u64.
//!
//! This benchmark compares:
//! - No cache (direct iterative computation) - baseline
//! - Various cached backends (Array, Vec, HashMap, DashMap, RwLock)

use aoc_solutions::utils::dp_cache::{
    ArrayBackend, DashMapBackend, DpCache, DpProblem, HashMapBackend, NoCacheBackend,
    ParallelArrayBackend, ParallelDpCache, ParallelNoCacheBackend, RwLockHashMapBackend,
    VecBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Pattern Problem Definition
// =============================================================================

/// Pattern problem with precomputed multiplier (avoids repeated pow computation)
/// Stores Option<u64> to handle overflow case where multiplier itself overflows
struct Pattern {
    multiplier: Option<u64>,
}

impl Pattern {
    /// Create a new Pattern from i (number of zeros between 1s)
    /// Precomputes the multiplier = 10^(i+1)
    fn new(i: usize) -> Self {
        Self {
            multiplier: 10u64.checked_pow((i + 1) as u32),
        }
    }
}

impl DpProblem<usize, Option<u64>> for Pattern {
    fn deps(&self, j: &usize) -> Vec<usize> {
        if *j <= 1 {
            vec![]
        } else {
            vec![j - 1]
        }
    }

    fn compute(&self, j: &usize, deps: Vec<Option<u64>>) -> Option<u64> {
        if *j == 0 {
            Some(0)
        } else if *j == 1 {
            Some(1)
        } else {
            let prev = deps.first()?.as_ref()?;
            let multiplier = self.multiplier?;
            prev.checked_mul(multiplier)?.checked_add(1)
        }
    }
}

/// Direct computation for verification
fn compute_pattern_direct(i: usize, j: usize) -> Option<u64> {
    if j == 0 {
        return Some(0);
    }
    let mut result: u64 = 1;
    let multiplier = 10u64.checked_pow((i + 1) as u32)?;
    for _ in 1..j {
        result = result.checked_mul(multiplier)?;
        result = result.checked_add(1)?;
    }
    Some(result)
}

const MAX_J: usize = 101;

fn main() {
    println!("Decimal Pattern DP Cache Benchmark");
    println!("===================================");
    println!("Pattern: 100...100...1 (decimal) with i zeros between j ones\n");

    let i_values: Vec<usize> = (0..=10).collect();
    let j_values: Vec<usize> = (1..=100).collect();

    let test_cases: Vec<(usize, usize)> = i_values
        .iter()
        .flat_map(|&i| j_values.iter().map(move |&j| (i, j)))
        .collect();

    println!(
        "Testing {} combinations (i: 0-10, j: 1-100)\n",
        test_cases.len()
    );

    // =========================================================================
    // Single-threaded benchmarks
    // =========================================================================
    println!("=== Single-threaded ===");

    // No cache - direct computation (baseline)
    println!("Running no cache (direct computation)...");
    let start = Instant::now();
    let no_cache_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| compute_pattern_direct(i, j))
        .collect();
    let no_cache_time = start.elapsed();
    println!("No cache (direct):           {:?}", no_cache_time);

    // NoCacheBackend with DpCache (measures DpCache wrapper overhead)
    println!("Running NoCacheBackend (DpCache wrapper overhead)...");
    let start = Instant::now();
    let nocache_backend_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(NoCacheBackend::<usize, Option<u64>>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let nocache_backend_time = start.elapsed();
    println!("NoCacheBackend (wrapper):    {:?}", nocache_backend_time);

    // ArrayBackend with DpCache (sequential) - zero allocation
    println!("Running ArrayBackend (sequential, zero-alloc)...");
    let start = Instant::now();
    let array_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(ArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let array_time = start.elapsed();
    println!("ArrayBackend (sequential):   {:?}", array_time);

    // VecBackend with DpCache (sequential)
    println!("Running VecBackend (sequential)...");
    let start = Instant::now();
    let vec_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(VecBackend::with_capacity(j + 1))
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let vec_time = start.elapsed();
    println!("VecBackend (sequential):     {:?}", vec_time);

    // HashMapBackend with DpCache (sequential)
    println!("Running HashMapBackend (sequential)...");
    let start = Instant::now();
    let hashmap_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(HashMapBackend::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
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
    let par_nocache_backend_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<usize, Option<u64>>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_nocache_backend_time = start.elapsed();
    println!("ParallelNoCacheBackend:      {:?}", par_nocache_backend_time);

    // ParallelArrayBackend with ParallelDpCache (sequential iteration)
    println!("Running ParallelArrayBackend (parallel, zero-alloc)...");
    let start = Instant::now();
    let par_array_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_array_time = start.elapsed();
    println!("ParallelArrayBackend:        {:?}", par_array_time);

    // ParallelDpCache with DashMapBackend (sequential iteration)
    println!("Running DashMapBackend (parallel)...");
    let start = Instant::now();
    let dashmap_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):   {:?}", dashmap_time);

    // ParallelDpCache with RwLockHashMapBackend (sequential iteration)
    println!("Running RwLockHashMapBackend (parallel)...");
    let start = Instant::now();
    let rwlock_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend:        {:?}", rwlock_time);

    // =========================================================================
    // Parallel iteration (par_iter)
    // =========================================================================
    println!("\n=== Parallel iteration (par_iter) ===");

    // No cache + parallel iteration
    println!("Running no cache + par_iter...");
    let start = Instant::now();
    let no_cache_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| compute_pattern_direct(i, j))
        .collect();
    let no_cache_par_time = start.elapsed();
    println!("No cache + par_iter:         {:?}", no_cache_par_time);

    // ParallelNoCacheBackend + par_iter
    println!("Running ParallelNoCacheBackend + par_iter...");
    let start = Instant::now();
    let par_nocache_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<usize, Option<u64>>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_nocache_par_time = start.elapsed();
    println!("ParallelNoCacheBackend+par:  {:?}", par_nocache_par_time);

    // ParallelArrayBackend + par_iter
    println!("Running ParallelArrayBackend + par_iter...");
    let start = Instant::now();
    let par_array_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_array_par_time = start.elapsed();
    println!("ParallelArrayBackend + par:  {:?}", par_array_par_time);

    // ParallelDpCache with DashMapBackend + parallel iteration
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // ParallelDpCache with RwLockHashMapBackend + parallel iteration
    println!("Running RwLockHashMapBackend + par_iter...");
    let start = Instant::now();
    let rwlock_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Pattern::new(i))
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par:  {:?}", rwlock_par_time);

    // =========================================================================
    // Verification
    // =========================================================================
    println!("\nVerifying against direct computation...");

    let mut all_match = true;
    let mut mismatches = 0;
    for (idx, &(i, j)) in test_cases.iter().enumerate() {
        let direct = no_cache_results[idx];
        if direct != no_cache_par_results[idx]
            || direct != nocache_backend_results[idx]
            || direct != par_nocache_backend_results[idx]
            || direct != par_nocache_par_results[idx]
            || direct != array_results[idx]
            || direct != vec_results[idx]
            || direct != hashmap_results[idx]
            || direct != par_array_results[idx]
            || direct != par_array_par_results[idx]
            || direct != dashmap_results[idx]
            || direct != dashmap_par_results[idx]
            || direct != rwlock_results[idx]
            || direct != rwlock_par_results[idx]
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at i={}, j={}: direct={:?}, nocache_backend={:?}, par_nocache_par={:?}",
                    i, j, direct, nocache_backend_results[idx], par_nocache_par_results[idx]
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
    for &(i, j) in &[(0, 3), (1, 4), (2, 3), (3, 5), (0, 19), (0, 20), (5, 3), (10, 2)] {
        if let Some(idx) = test_cases.iter().position(|&x| x == (i, j)) {
            match no_cache_results[idx] {
                Some(v) => println!("  i={}, j={}: {}", i, j, v),
                None => println!("  i={}, j={}: overflow", i, j),
            }
        }
    }

    // Count overflows
    let overflow_count = no_cache_results.iter().filter(|r| r.is_none()).count();
    println!(
        "\nOverflow cases: {} / {} ({:.1}%)",
        overflow_count,
        test_cases.len(),
        100.0 * overflow_count as f64 / test_cases.len() as f64
    );

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Performance Summary ===");
    println!("\nSingle-threaded:");
    println!("  No cache (baseline):       {:?}", no_cache_time);
    println!("  NoCacheBackend (wrapper):  {:?}", nocache_backend_time);
    println!("  ArrayBackend:              {:?}", array_time);
    println!("  VecBackend:                {:?}", vec_time);
    println!("  HashMapBackend:            {:?}", hashmap_time);

    println!("\nParallel backends (sequential iteration):");
    println!("  ParallelNoCacheBackend:    {:?}", par_nocache_backend_time);
    println!("  ParallelArrayBackend:      {:?}", par_array_time);
    println!("  DashMapBackend:            {:?}", dashmap_time);
    println!("  RwLockHashMapBackend:      {:?}", rwlock_time);

    println!("\nParallel iteration (par_iter):");
    println!("  No cache + par_iter:       {:?}", no_cache_par_time);
    println!("  ParallelNoCacheBackend:    {:?}", par_nocache_par_time);
    println!("  ParallelArrayBackend:      {:?}", par_array_par_time);
    println!("  DashMapBackend:            {:?}", dashmap_par_time);
    println!("  RwLockHashMapBackend:      {:?}", rwlock_par_time);

    // Cache benefit analysis
    let best_cached = array_time
        .min(vec_time)
        .min(hashmap_time)
        .min(par_array_time)
        .min(par_array_par_time)
        .min(dashmap_time)
        .min(dashmap_par_time)
        .min(rwlock_time)
        .min(rwlock_par_time);
    let best_no_cache = no_cache_time.min(no_cache_par_time);

    println!("\nCache benefit analysis:");
    if best_cached < best_no_cache {
        println!(
            "  Cache speedup vs no-cache: {:.2}x",
            best_no_cache.as_secs_f64() / best_cached.as_secs_f64()
        );
    } else {
        println!(
            "  Cache overhead: {:.2}x slower than no-cache",
            best_cached.as_secs_f64() / best_no_cache.as_secs_f64()
        );
    }
    println!(
        "  Note: Pattern computation is O(j) per call, so caching benefit depends on repeated subproblems"
    );
}
