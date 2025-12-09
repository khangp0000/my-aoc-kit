//! Benchmark comparing DP cache backends using the Collatz chain length problem.
//!
//! Run with: cargo run --example collatz_benchmark --release
//!
//! This benchmark computes Collatz chain lengths for 500 uniformly distributed random
//! numbers in range [1, MAX_N] and compares:
//! - HashMapBackend with DpCache (sequential)
//! - DashMapDpCache (parallel)
//!
//! Note: VecBackend is not suitable for Collatz because the sequence can produce
//! very large intermediate values (e.g., 3n+1 for large odd n), causing massive
//! memory allocation.

use aoc_solutions::utils::dp_cache::{DashMapDpCache, DpCache, HashMapBackend};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Collatz Chain Length Functions
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

const NUM_SAMPLES: usize = 50000;
const MAX_N: u64 = 10_000_000;

/// Simple LCG random number generator for reproducibility
fn generate_random_inputs(seed: u64, count: usize, max: u64) -> Vec<u64> {
    let mut rng = seed;
    (0..count)
        .map(|_| {
            // LCG: x = (a * x + c) mod m
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            (rng % max) + 1
        })
        .collect()
}

fn main() {
    println!("Collatz Chain Length Benchmark");
    println!("==============================");
    println!(
        "Computing chain lengths for {} random numbers in range [1, {}]\n",
        NUM_SAMPLES, MAX_N
    );

    // Generate random inputs (same for all backends)
    let inputs = generate_random_inputs(42, NUM_SAMPLES, MAX_N);

    println!("Sample inputs: {:?}...\n", &inputs[..5]);

    // 1. HashMapBackend with DpCache (sequential)
    let start = Instant::now();
    let hashmap_cache = DpCache::new(HashMapBackend::new(), collatz_deps, collatz_compute);
    let hashmap_results: Vec<u64> = inputs.iter().map(|&n| hashmap_cache.get(n)).collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential):       {:?}", hashmap_time);

    // 2. DashMapDpCache (parallel, sequential iteration over inputs)
    let start = Instant::now();
    let dashmap_cache = DashMapDpCache::new(collatz_deps, collatz_compute);
    let dashmap_results: Vec<u64> = inputs.iter().map(|&n| dashmap_cache.get(n)).collect();
    let dashmap_time = start.elapsed();
    println!("DashMapDpCache (parallel):         {:?}", dashmap_time);

    // 3. DashMapDpCache with parallel iteration over inputs
    let start = Instant::now();
    let dashmap_cache2 = DashMapDpCache::new(collatz_deps, collatz_compute);
    let dashmap_par_results: Vec<u64> = inputs
        .par_iter()
        .map(|&n| dashmap_cache2.get(n))
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapDpCache + par_iter:         {:?}", dashmap_par_time);

    // Verify all backends produce identical results
    println!("\nVerifying results...");
    let mut all_match = true;

    for i in 0..NUM_SAMPLES {
        let hashmap_val = hashmap_results[i];
        let dashmap_val = dashmap_results[i];
        let dashmap_par_val = dashmap_par_results[i];

        if hashmap_val != dashmap_val || hashmap_val != dashmap_par_val {
            println!(
                "Mismatch at input {}: HashMap={}, DashMap={}, DashMap+par={}",
                inputs[i], hashmap_val, dashmap_val, dashmap_par_val
            );
            all_match = false;
        }
    }

    if all_match {
        println!("✓ All backends produce identical results!");
    } else {
        println!("✗ Results do not match!");
    }

    // Print some sample results
    println!("\nSample results:");
    for i in 0..3 {
        println!("  n={}: chain_length={}", inputs[i], hashmap_results[i]);
    }

    // Summary
    println!("\nSummary:");
    println!("  Sequential (HashMap):     {:?}", hashmap_time);
    println!("  Parallel (DashMap):       {:?}", dashmap_time);
    println!("  DashMap + par_iter:       {:?}", dashmap_par_time);

    let best_parallel = dashmap_time.min(dashmap_par_time);
    if best_parallel < hashmap_time {
        println!(
            "  Best speedup: {:.2}x",
            hashmap_time.as_secs_f64() / best_parallel.as_secs_f64()
        );
    } else {
        println!(
            "  Slowdown: {:.2}x (parallel overhead)",
            best_parallel.as_secs_f64() / hashmap_time.as_secs_f64()
        );
    }
}
