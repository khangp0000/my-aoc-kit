//! Benchmark comparing DP cache backends using the Collatz chain length problem.
//!
//! Run with: cargo run --example collatz_benchmark --release
//!
//! This benchmark computes Collatz chain lengths for random numbers and compares:
//! - No cache (direct recursive computation) - baseline
//! - HashMapBackend with DpCache (sequential)
//! - DashMapBackend with ParallelDpCache (parallel)
//! - RwLockHashMapBackend with ParallelDpCache (parallel)
//!
//! Note: VecBackend and ArrayBackend are not suitable for Collatz because the
//! sequence can produce very large intermediate values (e.g., 3n+1 for large odd n),
//! causing massive memory allocation or out-of-bounds errors. Use HashMapBackend
//! or DashMapBackend for sparse/unbounded index spaces like Collatz.

use aoc_solutions::utils::dp_cache::{
    DashMapBackend, DpCache, DpProblem, HashMapBackend, NoCacheBackend, ParallelDpCache,
    ParallelNoCacheBackend, RwLockHashMapBackend,
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

/// Direct recursive Collatz computation without any caching (baseline)
fn collatz_no_cache(n: u64) -> u64 {
    if n <= 1 {
        0
    } else if n % 2 == 0 {
        1 + collatz_no_cache(n / 2)
    } else {
        1 + collatz_no_cache(3 * n + 1)
    }
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

    // =========================================================================
    // Single-threaded benchmarks
    // =========================================================================
    println!("=== Single-threaded ===");

    // No cache - direct recursive computation (baseline)
    let start = Instant::now();
    let no_cache_results: Vec<u64> = inputs.iter().map(|&n| collatz_no_cache(n)).collect();
    let no_cache_time = start.elapsed();
    println!("No cache (direct recursive):   {:?}", no_cache_time);

    // NoCacheBackend with DpCache (measures DpCache wrapper overhead)
    let start = Instant::now();
    let nocache_backend = DpCache::builder()
        .backend(NoCacheBackend::<u64, u64>::new())
        .problem(Collatz)
        .build();
    let nocache_backend_results: Vec<u64> = inputs
        .iter()
        .map(|n| nocache_backend.get(n).unwrap())
        .collect();
    let nocache_backend_time = start.elapsed();
    println!("NoCacheBackend (wrapper):      {:?}", nocache_backend_time);

    // HashMapBackend with DpCache (sequential)
    let start = Instant::now();
    let hashmap_cache = DpCache::builder()
        .backend(HashMapBackend::new())
        .problem(Collatz)
        .build();
    let hashmap_results: Vec<u64> = inputs.iter().map(|n| hashmap_cache.get(n).unwrap()).collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential):   {:?}", hashmap_time);

    // =========================================================================
    // Parallel backends (sequential iteration)
    // =========================================================================
    println!("\n=== Parallel backends (sequential iteration) ===");

    // ParallelNoCacheBackend with ParallelDpCache (measures parallel wrapper overhead)
    let start = Instant::now();
    let par_nocache_backend = ParallelDpCache::builder()
        .backend(ParallelNoCacheBackend::<u64, u64>::new())
        .problem(Collatz)
        .build();
    let par_nocache_backend_results: Vec<u64> = inputs
        .iter()
        .map(|n| par_nocache_backend.get(n).unwrap())
        .collect();
    let par_nocache_backend_time = start.elapsed();
    println!("ParallelNoCacheBackend:        {:?}", par_nocache_backend_time);

    // ParallelDpCache with DashMapBackend (sequential iteration)
    let start = Instant::now();
    let dashmap_cache = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();
    let dashmap_results: Vec<u64> = inputs.iter().map(|n| dashmap_cache.get(n).unwrap()).collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):     {:?}", dashmap_time);

    // ParallelDpCache with RwLockHashMapBackend (sequential iteration)
    let start = Instant::now();
    let rwlock_cache = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();
    let rwlock_results: Vec<u64> = inputs.iter().map(|n| rwlock_cache.get(n).unwrap()).collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend:          {:?}", rwlock_time);

    // =========================================================================
    // Parallel iteration (par_iter)
    // =========================================================================
    println!("\n=== Parallel iteration (par_iter) ===");

    // No cache + parallel iteration
    let start = Instant::now();
    let no_cache_par_results: Vec<u64> = inputs.par_iter().map(|&n| collatz_no_cache(n)).collect();
    let no_cache_par_time = start.elapsed();
    println!("No cache + par_iter:           {:?}", no_cache_par_time);

    // ParallelNoCacheBackend + par_iter
    let start = Instant::now();
    let par_nocache_backend2 = ParallelDpCache::builder()
        .backend(ParallelNoCacheBackend::<u64, u64>::new())
        .problem(Collatz)
        .build();
    let par_nocache_par_results: Vec<u64> = inputs
        .par_iter()
        .map(|n| par_nocache_backend2.get(n).unwrap())
        .collect();
    let par_nocache_par_time = start.elapsed();
    println!("ParallelNoCacheBackend+par:    {:?}", par_nocache_par_time);

    // ParallelDpCache with DashMapBackend + parallel iteration
    let start = Instant::now();
    let dashmap_cache2 = ParallelDpCache::builder()
        .backend(DashMapBackend::new())
        .problem(Collatz)
        .build();
    let dashmap_par_results: Vec<u64> = inputs
        .par_iter()
        .map(|n| dashmap_cache2.get(n).unwrap())
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:     {:?}", dashmap_par_time);

    // ParallelDpCache with RwLockHashMapBackend + parallel iteration
    let start = Instant::now();
    let rwlock_cache2 = ParallelDpCache::builder()
        .backend(RwLockHashMapBackend::new())
        .problem(Collatz)
        .build();
    let rwlock_par_results: Vec<u64> = inputs
        .par_iter()
        .map(|n| rwlock_cache2.get(n).unwrap())
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par:    {:?}", rwlock_par_time);

    // =========================================================================
    // Verification
    // =========================================================================
    println!("\nVerifying results...");
    let mut all_match = true;

    for i in 0..NUM_SAMPLES {
        if no_cache_results[i] != hashmap_results[i]
            || no_cache_results[i] != dashmap_results[i]
            || no_cache_results[i] != dashmap_par_results[i]
            || no_cache_results[i] != rwlock_results[i]
            || no_cache_results[i] != rwlock_par_results[i]
            || no_cache_results[i] != no_cache_par_results[i]
            || no_cache_results[i] != nocache_backend_results[i]
            || no_cache_results[i] != par_nocache_backend_results[i]
            || no_cache_results[i] != par_nocache_par_results[i]
        {
            println!(
                "Mismatch at input {}: NoCache={}, NoCacheBackend={}, HashMap={}",
                inputs[i], no_cache_results[i], nocache_backend_results[i], hashmap_results[i]
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

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Performance Summary ===");
    println!("\nSingle-threaded:");
    println!("  No cache (baseline):       {:?}", no_cache_time);
    println!("  NoCacheBackend (wrapper):  {:?}", nocache_backend_time);
    println!("  HashMapBackend:            {:?}", hashmap_time);

    println!("\nParallel backends (sequential iteration):");
    println!("  ParallelNoCacheBackend:    {:?}", par_nocache_backend_time);
    println!("  DashMapBackend:            {:?}", dashmap_time);
    println!("  RwLockHashMapBackend:      {:?}", rwlock_time);

    println!("\nParallel iteration (par_iter):");
    println!("  No cache + par_iter:       {:?}", no_cache_par_time);
    println!("  ParallelNoCacheBackend:    {:?}", par_nocache_par_time);
    println!("  DashMapBackend:            {:?}", dashmap_par_time);
    println!("  RwLockHashMapBackend:      {:?}", rwlock_par_time);

    // Compare cached vs non-cached
    let best_cached = hashmap_time
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

    let best_parallel = dashmap_time
        .min(dashmap_par_time)
        .min(rwlock_time)
        .min(rwlock_par_time);
    if best_parallel < hashmap_time {
        println!(
            "  Parallel speedup vs sequential: {:.2}x",
            hashmap_time.as_secs_f64() / best_parallel.as_secs_f64()
        );
    }
}
