//! Benchmark comparing DP cache backends using Longest Common Subsequence (LCS).
//!
//! Run with: cargo run --example lcs_benchmark --release
//!
//! LCS is a classic 2D DP problem: given two strings, find the length of their
//! longest common subsequence.
//!
//! This benchmark compares:
//! - Manual bottom-up DP (optimal)
//! - DpCache with various backends
//! - Parallel execution

use aoc_solutions::utils::dp_cache::{
    Array2DBackend, DashMapBackend, DpCache, DpProblem, HashMapBackend, ParallelArray2DBackend,
    ParallelDpCache, RwLockHashMapBackend, Vec2DBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// LCS Problem Definition
// =============================================================================

const MAX_LEN: usize = 100;

/// LCS problem using the trait-based API
struct LcsProblem<'a> {
    a: &'a [u8],
    b: &'a [u8],
}

impl<'a> LcsProblem<'a> {
    fn new(a: &'a [u8], b: &'a [u8]) -> Self {
        Self { a, b }
    }
}

impl<'a> DpProblem<(usize, usize), usize> for LcsProblem<'a> {
    fn deps(&self, pos: &(usize, usize)) -> Vec<(usize, usize)> {
        let (i, j) = *pos;
        if i == 0 || j == 0 {
            vec![]
        } else if self.a[i - 1] == self.b[j - 1] {
            vec![(i - 1, j - 1)]
        } else {
            vec![(i - 1, j), (i, j - 1)]
        }
    }

    fn compute(&self, pos: &(usize, usize), deps: Vec<usize>) -> usize {
        let (i, j) = *pos;
        if i == 0 || j == 0 {
            0
        } else if self.a[i - 1] == self.b[j - 1] {
            deps[0] + 1
        } else {
            deps[0].max(deps[1])
        }
    }
}

/// Manual bottom-up DP with local 2D array
fn lcs_bottom_up<const N: usize, const M: usize>(a: &[u8], b: &[u8]) -> usize {
    let mut dp = [[0usize; M]; N];
    for i in 0..a.len() + 1 {
        for j in 0..b.len() + 1 {
            dp[i][j] = if i == 0 || j == 0 {
                0
            } else if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }
    dp[a.len()][b.len()]
}

/// Manual bottom-up DP with Vec
fn lcs_bottom_up_vec(a: &[u8], b: &[u8]) -> usize {
    let n = a.len() + 1;
    let m = b.len() + 1;
    let mut dp = vec![vec![0usize; m]; n];
    for i in 0..n {
        for j in 0..m {
            dp[i][j] = if i == 0 || j == 0 {
                0
            } else if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }
    dp[a.len()][b.len()]
}

/// Generate random string for benchmarking
fn generate_random_string(seed: u64, len: usize) -> Vec<u8> {
    let mut rng = seed;
    (0..len)
        .map(|_| {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            b'a' + ((rng >> 32) % 26) as u8
        })
        .collect()
}

const STR_LEN: usize = 99; // MAX_LEN - 1 for dp array indexing
const NUM_PAIRS: usize = 100;

fn main() {
    println!("Longest Common Subsequence (LCS) Benchmark");
    println!("===========================================\n");

    // Generate test string pairs
    let pairs: Vec<(Vec<u8>, Vec<u8>)> = (0..NUM_PAIRS)
        .map(|i| {
            let a = generate_random_string(42 + i as u64, STR_LEN);
            let b = generate_random_string(1000 + i as u64, STR_LEN);
            (a, b)
        })
        .collect();

    println!(
        "Testing {} string pairs, each ~{} chars\n",
        NUM_PAIRS, STR_LEN
    );
    println!(
        "Sample: a[0..10]={:?}, b[0..10]={:?}",
        String::from_utf8_lossy(&pairs[0].0[0..10]),
        String::from_utf8_lossy(&pairs[0].1[0..10])
    );

    // =========================================================================
    // Local array cache (manual DP)
    // =========================================================================
    println!("\n=== Local array cache (manual DP) ===");

    // Bottom-up with local 2D array
    println!("Running bottom-up DP (local 2D array)...");
    let start = Instant::now();
    let bottom_up_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| lcs_bottom_up::<MAX_LEN, MAX_LEN>(a, b))
        .collect();
    let bottom_up_time = start.elapsed();
    println!("Bottom-up (local array):     {:?}", bottom_up_time);

    // Bottom-up with Vec
    println!("Running bottom-up DP (Vec)...");
    let start = Instant::now();
    let bottom_up_vec_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| lcs_bottom_up_vec(a, b))
        .collect();
    let bottom_up_vec_time = start.elapsed();
    println!("Bottom-up (Vec):             {:?}", bottom_up_vec_time);

    // Bottom-up + par_iter
    println!("Running bottom-up + par_iter...");
    let start = Instant::now();
    let bottom_up_par_results: Vec<usize> = pairs
        .par_iter()
        .map(|(a, b)| lcs_bottom_up::<MAX_LEN, MAX_LEN>(a, b))
        .collect();
    let bottom_up_par_time = start.elapsed();
    println!("Bottom-up + par_iter:        {:?}", bottom_up_par_time);

    // =========================================================================
    // DpCache struct - Single-threaded
    // =========================================================================
    println!("\n=== DpCache struct - Single-threaded ===");

    // Array2DBackend
    println!("Running Array2DBackend...");
    let start = Instant::now();
    let array2d_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = DpCache::builder()
                .backend(Array2DBackend::<usize, MAX_LEN, MAX_LEN>::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let array2d_time = start.elapsed();
    println!("Array2DBackend:              {:?}", array2d_time);

    // Vec2DBackend
    println!("Running Vec2DBackend...");
    let start = Instant::now();
    let vec2d_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = DpCache::builder()
                .backend(Vec2DBackend::with_capacity(a.len() + 1, b.len() + 1))
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let vec2d_time = start.elapsed();
    println!("Vec2DBackend:                {:?}", vec2d_time);

    // HashMapBackend
    println!("Running HashMapBackend...");
    let start = Instant::now();
    let hashmap_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = DpCache::builder()
                .backend(HashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend:              {:?}", hashmap_time);

    // =========================================================================
    // DpCache struct - Parallel backends (sequential iteration)
    // =========================================================================
    println!("\n=== Parallel backends (sequential iteration) ===");

    // ParallelArray2DBackend
    println!("Running ParallelArray2DBackend...");
    let start = Instant::now();
    let par_array2d_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = ParallelDpCache::builder()
                .backend(ParallelArray2DBackend::<usize, MAX_LEN, MAX_LEN>::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let par_array2d_time = start.elapsed();
    println!("ParallelArray2DBackend:      {:?}", par_array2d_time);

    // DashMapBackend
    println!("Running DashMapBackend...");
    let start = Instant::now();
    let dashmap_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend:              {:?}", dashmap_time);

    // RwLockHashMapBackend
    println!("Running RwLockHashMapBackend...");
    let start = Instant::now();
    let rwlock_results: Vec<usize> = pairs
        .iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend:        {:?}", rwlock_time);

    // =========================================================================
    // DpCache struct - Parallel iteration (par_iter)
    // =========================================================================
    println!("\n=== Parallel iteration (par_iter) ===");

    // ParallelArray2DBackend + par_iter
    println!("Running ParallelArray2DBackend + par_iter...");
    let start = Instant::now();
    let par_array2d_par_results: Vec<usize> = pairs
        .par_iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = ParallelDpCache::builder()
                .backend(ParallelArray2DBackend::<usize, MAX_LEN, MAX_LEN>::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let par_array2d_par_time = start.elapsed();
    println!("ParallelArray2DBackend+par:  {:?}", par_array2d_par_time);

    // DashMapBackend + par_iter
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<usize> = pairs
        .par_iter()
        .map(|(a, b)| {
            let problem = LcsProblem::new(a, b);
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(a.len(), b.len())).unwrap()
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // =========================================================================
    // Verification
    // =========================================================================
    println!("\nVerifying results...");

    let mut all_match = true;
    let mut mismatches = 0;
    for i in 0..NUM_PAIRS {
        let expected = bottom_up_results[i];
        if expected != bottom_up_vec_results[i]
            || expected != bottom_up_par_results[i]
            || expected != array2d_results[i]
            || expected != vec2d_results[i]
            || expected != hashmap_results[i]
            || expected != par_array2d_results[i]
            || expected != dashmap_results[i]
            || expected != rwlock_results[i]
            || expected != par_array2d_par_results[i]
            || expected != dashmap_par_results[i]
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at pair {}: expected={}, array2d={}, hashmap={}",
                    i, expected, array2d_results[i], hashmap_results[i]
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
    println!("\nSample LCS lengths:");
    for i in 0..3 {
        println!("  Pair {}: LCS length = {}", i, bottom_up_results[i]);
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Performance Summary ===");

    println!("\nLocal array cache (manual DP):");
    println!("  Bottom-up (local array):    {:?}", bottom_up_time);
    println!("  Bottom-up (Vec):            {:?}", bottom_up_vec_time);
    println!("  Bottom-up + par_iter:       {:?}", bottom_up_par_time);

    println!("\nDpCache struct - Single-threaded:");
    println!("  Array2DBackend:             {:?}", array2d_time);
    println!("  Vec2DBackend:               {:?}", vec2d_time);
    println!("  HashMapBackend:             {:?}", hashmap_time);

    println!("\nParallel backends (sequential iteration):");
    println!("  ParallelArray2DBackend:     {:?}", par_array2d_time);
    println!("  DashMapBackend:             {:?}", dashmap_time);
    println!("  RwLockHashMapBackend:       {:?}", rwlock_time);

    println!("\nParallel iteration (par_iter):");
    println!("  ParallelArray2DBackend:     {:?}", par_array2d_par_time);
    println!("  DashMapBackend:             {:?}", dashmap_par_time);

    // Comparisons
    let best_dpcache = array2d_time
        .min(vec2d_time)
        .min(hashmap_time)
        .min(par_array2d_par_time)
        .min(dashmap_par_time);

    println!("\nLocal array vs DpCache struct:");
    println!(
        "  Bottom-up vs Array2DBackend: {:.2}x faster",
        array2d_time.as_secs_f64() / bottom_up_time.as_secs_f64()
    );
    println!(
        "  Bottom-up Vec vs Vec2DBackend: {:.2}x faster",
        vec2d_time.as_secs_f64() / bottom_up_vec_time.as_secs_f64()
    );

    println!("\nBackend comparison:");
    println!(
        "  Array2DBackend vs Vec2DBackend: {:.2}x",
        vec2d_time.as_secs_f64() / array2d_time.as_secs_f64()
    );
    println!(
        "  Array2DBackend vs HashMapBackend: {:.2}x",
        hashmap_time.as_secs_f64() / array2d_time.as_secs_f64()
    );
    println!("  Best DpCache time: {:?}", best_dpcache);

    println!("\n=== Conclusion ===");
    println!(
        "For LCS, local bottom-up is {:.1}x faster than DpCache Array2DBackend.",
        array2d_time.as_secs_f64() / bottom_up_time.as_secs_f64()
    );
    println!("LCS has regular 2D dependencies, making manual DP straightforward.");
}
