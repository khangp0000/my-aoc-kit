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
    ArrayBackend, DashMapBackend, DpCache, DpProblem, HashMapBackend, ParallelArrayBackend,
    ParallelDpCache, RwLockHashMapBackend, VecBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Pattern Problem Definition
// =============================================================================

/// Pattern problem with configurable spacing (i zeros between 1s)
struct Pattern {
    i: usize,
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
            let multiplier = 10u64.checked_pow((self.i + 1) as u32)?;
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

    // 1. ArrayBackend with DpCache (sequential) - zero allocation
    const MAX_J: usize = 101;
    println!("Running ArrayBackend (sequential, zero-alloc)...");
    let start = Instant::now();
    let array_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(ArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let array_time = start.elapsed();
    println!("ArrayBackend (sequential):   {:?}", array_time);

    // 2. VecBackend with DpCache (sequential)
    println!("Running VecBackend (sequential)...");
    let start = Instant::now();
    let vec_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(VecBackend::with_capacity(j + 1))
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let vec_time = start.elapsed();
    println!("VecBackend (sequential):     {:?}", vec_time);

    // 3. HashMapBackend with DpCache (sequential)
    println!("Running HashMapBackend (sequential)...");
    let start = Instant::now();
    let hashmap_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = DpCache::builder()
                .backend(HashMapBackend::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential): {:?}", hashmap_time);

    // 4. ParallelArrayBackend with ParallelDpCache (sequential iteration)
    println!("Running ParallelArrayBackend (parallel, zero-alloc)...");
    let start = Instant::now();
    let par_array_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_array_time = start.elapsed();
    println!("ParallelArrayBackend:        {:?}", par_array_time);

    // 5. ParallelArrayBackend + par_iter
    println!("Running ParallelArrayBackend + par_iter...");
    let start = Instant::now();
    let par_array_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(ParallelArrayBackend::<Option<u64>, MAX_J>::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let par_array_par_time = start.elapsed();
    println!("ParallelArrayBackend + par:  {:?}", par_array_par_time);

    // 6. ParallelDpCache with DashMapBackend (sequential iteration)
    println!("Running DashMapBackend (parallel)...");
    let start = Instant::now();
    let dashmap_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):   {:?}", dashmap_time);

    // 7. ParallelDpCache with DashMapBackend + parallel iteration
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // 8. ParallelDpCache with RwLockHashMapBackend (sequential iteration)
    println!("Running RwLockHashMapBackend (parallel)...");
    let start = Instant::now();
    let rwlock_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend (parallel): {:?}", rwlock_time);

    // 9. ParallelDpCache with RwLockHashMapBackend + parallel iteration
    println!("Running RwLockHashMapBackend + par_iter...");
    let start = Instant::now();
    let rwlock_par_results: Vec<Option<u64>> = test_cases
        .par_iter()
        .map(|&(i, j)| {
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(Pattern { i })
                .build();
            cache.get(&j).unwrap()
        })
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par_iter: {:?}", rwlock_par_time);

    // 10. Direct computation for verification
    println!("\nVerifying against direct computation...");
    let direct_results: Vec<Option<u64>> = test_cases
        .iter()
        .map(|&(i, j)| compute_pattern_direct(i, j))
        .collect();

    let mut all_match = true;
    let mut mismatches = 0;
    for (idx, &(i, j)) in test_cases.iter().enumerate() {
        let direct = direct_results[idx];
        if direct != array_results[idx]
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
                    "Mismatch at i={}, j={}: direct={:?}, array={:?}, vec={:?}, hashmap={:?}",
                    i, j, direct, array_results[idx], vec_results[idx], hashmap_results[idx]
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
            match direct_results[idx] {
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
    println!("  ArrayBackend (sequential):       {:?}", array_time);
    println!("  VecBackend (sequential):         {:?}", vec_time);
    println!("  HashMapBackend (sequential):     {:?}", hashmap_time);
    println!("  ParallelArrayBackend:            {:?}", par_array_time);
    println!("  ParallelArrayBackend + par_iter: {:?}", par_array_par_time);
    println!("  DashMapBackend (parallel):       {:?}", dashmap_time);
    println!("  DashMapBackend + par_iter:       {:?}", dashmap_par_time);
    println!("  RwLockHashMapBackend (parallel): {:?}", rwlock_time);
    println!("  RwLockHashMapBackend + par_iter: {:?}", rwlock_par_time);
}
