//! Benchmark comparing DP cache backends using the Collatz chain length problem.
//!
//! Run with: cargo run --example collatz_benchmark --release
//!
//! This benchmark computes Collatz chain lengths for random numbers and compares:
//! - HashMapBackend with DpCache (sequential)
//! - DashMapBackend with ParallelDpCache (parallel)
//! - RwLockHashMapBackend with ParallelDpCache (parallel)
//!
//! Note: VecBackend is not suitable for Collatz because the sequence can produce
//! very large intermediate values (e.g., 3n+1 for large odd n), causing massive
//! memory allocation.

use aoc_solutions::utils::dp_cache::{
    DashMapBackend, DpCache, DpProblem, HashMapBackend, ParallelDpCache, RwLockHashMapBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Collatz Problem Definition
// =============================================================================

/// Collatz chain length problem using the trait-based API
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

const NUM_SAMPLES: usize = 50000;
const MAX_N: u64 = 10_000_000;

/// Simple LCG random number generator for reproducibility
fn generate_random_inputs(seed: u64, count: usize, max: u64) -> Vec<u64> {
    let mut rng = seed;
    (0..count)
        .map(|_| {
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

    let inputs = generate_random_inputs(42, NUM_SAMPLES, MAX_N);
    println!("Sample inputs: {:?}...\n", &inputs[..5]);

    // 1. HashMapBackend with DpCache (sequential)
    let start = Instant::now();
    let hashmap_cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    let hashmap_results: Vec<u64> = inputs.iter().map(|n| hashmap_cache.get(n).unwrap()).collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential):       {:?}", hashmap_time);

    // 2. ParallelDpCache with DashMapBackend (sequential iteration)
    let start = Instant::now();
    let dashmap_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();
    let dashmap_results: Vec<u64> = inputs.iter().map(|n| dashmap_cache.get(n).unwrap()).collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):         {:?}", dashmap_time);

    // 3. ParallelDpCache with DashMapBackend + parallel iteration
    let start = Instant::now();
    let dashmap_cache2 = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();
    let dashmap_par_results: Vec<u64> = inputs.par_iter().map(|n| dashmap_cache2.get(n).unwrap()).collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:         {:?}", dashmap_par_time);

    // 4. ParallelDpCache with RwLockHashMapBackend (sequential iteration)
    let start = Instant::now();
    let rwlock_cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();
    let rwlock_results: Vec<u64> = inputs.iter().map(|n| rwlock_cache.get(n).unwrap()).collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend (parallel):   {:?}", rwlock_time);

    // 5. ParallelDpCache with RwLockHashMapBackend + parallel iteration
    let start = Instant::now();
    let rwlock_cache2 = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();
    let rwlock_par_results: Vec<u64> = inputs.par_iter().map(|n| rwlock_cache2.get(n).unwrap()).collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par_iter:   {:?}", rwlock_par_time);

    // Verify all backends produce identical results
    println!("\nVerifying results...");
    let mut all_match = true;

    for i in 0..NUM_SAMPLES {
        if hashmap_results[i] != dashmap_results[i]
            || hashmap_results[i] != dashmap_par_results[i]
            || hashmap_results[i] != rwlock_results[i]
            || hashmap_results[i] != rwlock_par_results[i]
        {
            println!(
                "Mismatch at input {}: HashMap={}, DashMap={}, DashMap+par={}, RwLock={}, RwLock+par={}",
                inputs[i], hashmap_results[i], dashmap_results[i], dashmap_par_results[i],
                rwlock_results[i], rwlock_par_results[i]
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
    println!("  Parallel (RwLock):        {:?}", rwlock_time);
    println!("  RwLock + par_iter:        {:?}", rwlock_par_time);

    let best_parallel = dashmap_time
        .min(dashmap_par_time)
        .min(rwlock_time)
        .min(rwlock_par_time);
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
