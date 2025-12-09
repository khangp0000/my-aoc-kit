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

use aoc_solutions::utils::dp_cache::{
    DashMapBackend, DpCache, HashMapBackend, ParallelDpCache, RwLockHashMapBackend, VecBackend,
};
use rayon::prelude::*;
use std::time::Instant;

/// Dependencies: pattern(j) depends on pattern(j-1)
fn pattern_deps(j: &usize) -> Vec<usize> {
    if *j <= 1 {
        vec![]
    } else {
        vec![j - 1]
    }
}

/// Compute function for given i (zeros between 1s)
/// Returns None on overflow
fn make_pattern_compute(
    i: usize,
) -> impl Fn(&usize, Vec<Option<u64>>) -> Option<u64> + Clone + Send + Sync {
    move |j: &usize, deps: Vec<Option<u64>>| {
        if *j == 0 {
            Some(0)
        } else if *j == 1 {
            Some(1)
        } else {
            // Get previous value
            let prev = deps.first()?.as_ref()?;
            // Multiply by 10^(i+1) and add 1
            // e.g., i=2: prev * 1000 + 1
            let multiplier = 10u64.checked_pow((i + 1) as u32)?;
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

fn main() {
    println!("Decimal Pattern DP Cache Benchmark");
    println!("===================================");
    println!("Pattern: 100...100...1 (decimal) with i zeros between j ones\n");

    // Test parameters
    // i: zeros between 1s (0-10)
    // j: number of 1s (1-100)
    let i_values: Vec<usize> = (0..=10).collect();
    let j_values: Vec<usize> = (1..=100).collect();

    // Generate all (i, j) pairs
    let test_cases: Vec<(usize, usize)> = i_values
        .iter()
        .flat_map(|&i| j_values.iter().map(move |&j| (i, j)))
        .collect();

    println!(
        "Testing {} combinations (i: 0-10, j: 1-100)\n",
        test_cases.len()
    );

    // 1. VecBackend with DpCache (sequential)
    println!("Running VecBackend (sequential)...");
    let start = Instant::now();
    let mut vec_results: Vec<Option<u64>> = Vec::with_capacity(test_cases.len());
    for &(i, j) in &test_cases {
        let cache = DpCache::new(
            VecBackend::with_capacity(j + 1),
            pattern_deps,
            make_pattern_compute(i),
        );
        let result = cache.get(&j);
        vec_results.push(result);
    }
    let vec_time = start.elapsed();
    println!("VecBackend (sequential):     {:?}", vec_time);

    // 2. HashMapBackend with DpCache (sequential)
    println!("Running HashMapBackend (sequential)...");
    let start = Instant::now();
    let mut hashmap_results: Vec<Option<u64>> = Vec::with_capacity(test_cases.len());
    for &(i, j) in &test_cases {
        let cache = DpCache::new(HashMapBackend::new(), pattern_deps, make_pattern_compute(i));
        let result = cache.get(&j);
        hashmap_results.push(result);
    }
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential): {:?}", hashmap_time);

    // 3. ParallelDpCache with DashMapBackend (parallel deps)
    println!("Running DashMapBackend (parallel)...");
    let start = Instant::now();
    let mut dashmap_results: Vec<Option<u64>> = Vec::with_capacity(test_cases.len());
    for &(i, j) in &test_cases {
        let cache =
            ParallelDpCache::new(DashMapBackend::new(), pattern_deps, make_pattern_compute(i));
        let result = cache.get(&j);
        dashmap_results.push(result);
    }
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):   {:?}", dashmap_time);

    // 4. ParallelDpCache with DashMapBackend + parallel iteration
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache =
                ParallelDpCache::new(DashMapBackend::new(), pattern_deps, make_pattern_compute(i));
            cache.get(&j)
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // 5. ParallelDpCache with RwLockHashMapBackend (parallel deps)
    println!("Running RwLockHashMapBackend (parallel)...");
    let start = Instant::now();
    let mut rwlock_results: Vec<Option<u64>> = Vec::with_capacity(test_cases.len());
    for &(i, j) in &test_cases {
        let cache = ParallelDpCache::new(
            RwLockHashMapBackend::new(),
            pattern_deps,
            make_pattern_compute(i),
        );
        let result = cache.get(&j);
        rwlock_results.push(result);
    }
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend (parallel): {:?}", rwlock_time);

    // 6. ParallelDpCache with RwLockHashMapBackend + parallel iteration
    println!("Running RwLockHashMapBackend + par_iter...");
    let start = Instant::now();
    let rwlock_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::new(
                RwLockHashMapBackend::new(),
                pattern_deps,
                make_pattern_compute(i),
            );
            cache.get(&j)
        })
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par_iter: {:?}", rwlock_par_time);

    // 7. Direct computation for verification
    println!("\nVerifying against direct computation...");
    let direct_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| compute_pattern_direct(i, j))
        .collect();

    let mut all_match = true;
    let mut mismatches = 0;
    for (idx, &(i, j)) in test_cases.iter().enumerate() {
        let direct = direct_results[idx];
        let vec_r = vec_results[idx];
        let hashmap_r = hashmap_results[idx];
        let dashmap_r = dashmap_results[idx];
        let dashmap_par_r = dashmap_par_results[idx];
        let rwlock_r = rwlock_results[idx];
        let rwlock_par_r = rwlock_par_results[idx];

        if direct != vec_r
            || direct != hashmap_r
            || direct != dashmap_r
            || direct != dashmap_par_r
            || direct != rwlock_r
            || direct != rwlock_par_r
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at i={}, j={}: direct={:?}, vec={:?}, hashmap={:?}, dashmap={:?}, dashmap_par={:?}, rwlock={:?}, rwlock_par={:?}",
                    i, j, direct, vec_r, hashmap_r, dashmap_r, dashmap_par_r, rwlock_r, rwlock_par_r
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
        let idx = test_cases.iter().position(|&x| x == (i, j));
        if let Some(idx) = idx {
            let result = direct_results[idx];
            match result {
                Some(v) => println!("  i={}, j={}: {}", i, j, v),
                None => println!("  i={}, j={}: overflow", i, j),
            }
        }
    }

    // Count overflows
    let overflow_count = direct_results.iter().filter(|r| r.is_none()).count();
    println!(
        "\nOverflow cases: {} / {} ({:.1}%)",
        overflow_count,
        test_cases.len(),
        100.0 * overflow_count as f64 / test_cases.len() as f64
    );

    // Summary
    println!("\nPerformance Summary:");
    println!("  VecBackend (sequential):     {:?}", vec_time);
    println!("  HashMapBackend (sequential): {:?}", hashmap_time);
    println!("  DashMapDpCache (parallel):   {:?}", dashmap_time);
    println!("  DashMapDpCache + par_iter:   {:?}", dashmap_par_time);
    println!("  RwLockDpCache (parallel):    {:?}", rwlock_time);
    println!("  RwLockDpCache + par_iter:    {:?}", rwlock_par_time);
}
