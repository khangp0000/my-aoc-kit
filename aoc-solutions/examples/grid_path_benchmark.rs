//! Benchmark comparing 2D DP cache backends using the minimum path sum problem.
//!
//! Run with: cargo run --example grid_path_benchmark --release
//!
//! Given a grid of non-negative integers, find the minimum path sum from
//! top-left to bottom-right, moving only right or down.
//!
//! This benchmark compares 2D backends:
//! - No cache (direct recursive computation) - baseline
//! - Array2DBackend (fixed-size, zero allocation)
//! - Vec2DBackend (dynamic size)
//! - HashMapBackend (sparse)
//! - ParallelArray2DBackend (thread-safe fixed-size)
//! - DashMapBackend (thread-safe sparse)
//! - RwLockHashMapBackend (thread-safe sparse)

use aoc_solutions::utils::dp_cache::{
    Array2DBackend, DashMapBackend, DpCache, DpProblem, HashMapBackend, NoCacheBackend,
    ParallelArray2DBackend, ParallelDpCache, ParallelNoCacheBackend, RwLockHashMapBackend,
    Vec2DBackend,
};
use rayon::prelude::*;
use std::time::Instant;

// =============================================================================
// Grid Path Problem Definition
// =============================================================================

/// Minimum path sum problem using the trait-based API
struct MinPathSum<'a> {
    grid: &'a [Vec<u32>],
}

impl<'a> MinPathSum<'a> {
    fn new(grid: &'a [Vec<u32>]) -> Self {
        Self { grid }
    }
}

impl<'a> DpProblem<(usize, usize), u32> for MinPathSum<'a> {
    fn deps(&self, pos: &(usize, usize)) -> Vec<(usize, usize)> {
        let (row, col) = *pos;
        let mut deps = Vec::new();
        if row > 0 {
            deps.push((row - 1, col)); // from above
        }
        if col > 0 {
            deps.push((row, col - 1)); // from left
        }
        deps
    }

    fn compute(&self, pos: &(usize, usize), deps: Vec<u32>) -> u32 {
        let (row, col) = *pos;
        let cell_value = self.grid[row][col];

        if deps.is_empty() {
            // Top-left corner
            cell_value
        } else {
            // Minimum of dependencies + current cell
            cell_value + deps.into_iter().min().unwrap()
        }
    }
}

/// Direct recursive computation for verification
fn min_path_sum_direct(grid: &[Vec<u32>], row: usize, col: usize) -> u32 {
    let cell_value = grid[row][col];
    if row == 0 && col == 0 {
        cell_value
    } else if row == 0 {
        cell_value + min_path_sum_direct(grid, row, col - 1)
    } else if col == 0 {
        cell_value + min_path_sum_direct(grid, row - 1, col)
    } else {
        cell_value
            + min_path_sum_direct(grid, row - 1, col)
                .min(min_path_sum_direct(grid, row, col - 1))
    }
}

/// Generate a random grid for benchmarking
fn generate_random_grid(seed: u64, rows: usize, cols: usize, max_value: u32) -> Vec<Vec<u32>> {
    let mut rng = seed;
    (0..rows)
        .map(|_| {
            (0..cols)
                .map(|_| {
                    rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
                    (rng % max_value as u64) as u32
                })
                .collect()
        })
        .collect()
}

const GRID_SIZE: usize = 50;
const NUM_GRIDS: usize = 100;
const SMALL_GRID_SIZE: usize = 15;
const NUM_SMALL_GRIDS: usize = 10;

/// Bottom-up DP using a local 2D array (no DpCache struct)
fn min_path_sum_bottom_up<const R: usize, const C: usize>(grid: &[Vec<u32>]) -> u32 {
    let mut dp = [[0u32; C]; R];
    for row in 0..R {
        for col in 0..C {
            let cell = grid[row][col];
            dp[row][col] = if row == 0 && col == 0 {
                cell
            } else if row == 0 {
                cell + dp[row][col - 1]
            } else if col == 0 {
                cell + dp[row - 1][col]
            } else {
                cell + dp[row - 1][col].min(dp[row][col - 1])
            };
        }
    }
    dp[R - 1][C - 1]
}

/// Bottom-up DP using a Vec (dynamic size, no DpCache struct)
fn min_path_sum_bottom_up_vec(grid: &[Vec<u32>]) -> u32 {
    let rows = grid.len();
    let cols = grid[0].len();
    let mut dp = vec![vec![0u32; cols]; rows];
    for row in 0..rows {
        for col in 0..cols {
            let cell = grid[row][col];
            dp[row][col] = if row == 0 && col == 0 {
                cell
            } else if row == 0 {
                cell + dp[row][col - 1]
            } else if col == 0 {
                cell + dp[row - 1][col]
            } else {
                cell + dp[row - 1][col].min(dp[row][col - 1])
            };
        }
    }
    dp[rows - 1][cols - 1]
}

fn main() {
    println!("Minimum Path Sum Benchmark (2D DP)");
    println!("===================================\n");

    // Generate test grids
    let grids: Vec<Vec<Vec<u32>>> = (0..NUM_GRIDS)
        .map(|i| generate_random_grid(42 + i as u64, GRID_SIZE, GRID_SIZE, 100))
        .collect();

    let small_grids: Vec<Vec<Vec<u32>>> = (0..NUM_SMALL_GRIDS)
        .map(|i| generate_random_grid(42 + i as u64, SMALL_GRID_SIZE, SMALL_GRID_SIZE, 100))
        .collect();

    println!("Sample grid[0][0..3][0..3]:");
    for row in 0..3 {
        println!("  {:?}", &grids[0][row][0..3]);
    }

    // =========================================================================
    // Local array cache (manual DP - no DpCache struct)
    // =========================================================================
    println!("\n=== Local array cache (manual DP) ===");

    // Bottom-up DP with local 2D array
    println!("Running bottom-up DP (local 2D array)...");
    let start = Instant::now();
    let bottom_up_results: Vec<u32> = grids
        .iter()
        .map(|grid| min_path_sum_bottom_up::<GRID_SIZE, GRID_SIZE>(grid))
        .collect();
    let bottom_up_time = start.elapsed();
    println!("Bottom-up (local array):     {:?}", bottom_up_time);

    // Bottom-up DP with Vec
    println!("Running bottom-up DP (Vec)...");
    let start = Instant::now();
    let bottom_up_vec_results: Vec<u32> = grids
        .iter()
        .map(|grid| min_path_sum_bottom_up_vec(grid))
        .collect();
    let bottom_up_vec_time = start.elapsed();
    println!("Bottom-up (Vec):             {:?}", bottom_up_vec_time);

    // Bottom-up + par_iter (local array)
    println!("Running bottom-up + par_iter (local array)...");
    let start = Instant::now();
    let bottom_up_par_results: Vec<u32> = grids
        .par_iter()
        .map(|grid| min_path_sum_bottom_up::<GRID_SIZE, GRID_SIZE>(grid))
        .collect();
    let bottom_up_par_time = start.elapsed();
    println!("Bottom-up + par_iter:        {:?}", bottom_up_par_time);

    // Bottom-up + par_iter (Vec)
    println!("Running bottom-up Vec + par_iter...");
    let start = Instant::now();
    let bottom_up_vec_par_results: Vec<u32> = grids
        .par_iter()
        .map(|grid| min_path_sum_bottom_up_vec(grid))
        .collect();
    let bottom_up_vec_par_time = start.elapsed();
    println!("Bottom-up Vec + par_iter:    {:?}", bottom_up_vec_par_time);

    // =========================================================================
    // Big grids (50x50) - DpCache struct
    // =========================================================================
    println!("\n=== Big grids ({}x{}, {} grids) - DpCache struct ===", GRID_SIZE, GRID_SIZE, NUM_GRIDS);

    // Array2DBackend with DpCache (sequential) - zero allocation
    println!("Running Array2DBackend (sequential, zero-alloc)...");
    let start = Instant::now();
    let array2d_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = DpCache::builder()
                .backend(Array2DBackend::<u32, GRID_SIZE, GRID_SIZE>::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let array2d_time = start.elapsed();
    println!("Array2DBackend (sequential): {:?}", array2d_time);

    // Vec2DBackend with DpCache (sequential)
    println!("Running Vec2DBackend (sequential)...");
    let start = Instant::now();
    let vec2d_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = DpCache::builder()
                .backend(Vec2DBackend::with_capacity(GRID_SIZE, GRID_SIZE))
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let vec2d_time = start.elapsed();
    println!("Vec2DBackend (sequential):   {:?}", vec2d_time);

    // HashMapBackend with DpCache (sequential)
    println!("Running HashMapBackend (sequential)...");
    let start = Instant::now();
    let hashmap_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = DpCache::builder()
                .backend(HashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let hashmap_time = start.elapsed();
    println!("HashMapBackend (sequential): {:?}", hashmap_time);

    // =========================================================================
    // Big grids - Parallel backends (sequential iteration)
    // =========================================================================
    println!("\n=== Big grids - Parallel backends (sequential iteration) ===");

    // ParallelArray2DBackend with ParallelDpCache
    println!("Running ParallelArray2DBackend (parallel, zero-alloc)...");
    let start = Instant::now();
    let par_array2d_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(ParallelArray2DBackend::<u32, GRID_SIZE, GRID_SIZE>::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let par_array2d_time = start.elapsed();
    println!("ParallelArray2DBackend:      {:?}", par_array2d_time);

    // DashMapBackend with ParallelDpCache
    println!("Running DashMapBackend (parallel)...");
    let start = Instant::now();
    let dashmap_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let dashmap_time = start.elapsed();
    println!("DashMapBackend (parallel):   {:?}", dashmap_time);

    // RwLockHashMapBackend with ParallelDpCache
    println!("Running RwLockHashMapBackend (parallel)...");
    let start = Instant::now();
    let rwlock_results: Vec<u32> = grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let rwlock_time = start.elapsed();
    println!("RwLockHashMapBackend:        {:?}", rwlock_time);

    // =========================================================================
    // Big grids - Parallel iteration (par_iter)
    // =========================================================================
    println!("\n=== Big grids - Parallel iteration (par_iter) ===");

    // ParallelArray2DBackend + par_iter
    println!("Running ParallelArray2DBackend + par_iter...");
    let start = Instant::now();
    let par_array2d_par_results: Vec<u32> = grids
        .par_iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(ParallelArray2DBackend::<u32, GRID_SIZE, GRID_SIZE>::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let par_array2d_par_time = start.elapsed();
    println!("ParallelArray2DBackend+par:  {:?}", par_array2d_par_time);

    // DashMapBackend + par_iter
    println!("Running DashMapBackend + par_iter...");
    let start = Instant::now();
    let dashmap_par_results: Vec<u32> = grids
        .par_iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(DashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let dashmap_par_time = start.elapsed();
    println!("DashMapBackend + par_iter:   {:?}", dashmap_par_time);

    // RwLockHashMapBackend + par_iter
    println!("Running RwLockHashMapBackend + par_iter...");
    let start = Instant::now();
    let rwlock_par_results: Vec<u32> = grids
        .par_iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(RwLockHashMapBackend::new())
                .problem(problem)
                .build();
            cache.get(&(GRID_SIZE - 1, GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let rwlock_par_time = start.elapsed();
    println!("RwLockHashMapBackend + par:  {:?}", rwlock_par_time);

    // =========================================================================
    // Small grids (15x15) - Wrapper overhead comparison
    // =========================================================================
    println!("\n=== Small grids ({}x{}, {} grids) - Wrapper overhead comparison ===", 
             SMALL_GRID_SIZE, SMALL_GRID_SIZE, NUM_SMALL_GRIDS);

    // No cache - direct recursive computation (baseline)
    println!("Running no cache (direct recursive)...");
    let start = Instant::now();
    let no_cache_results: Vec<u32> = small_grids
        .iter()
        .map(|grid| min_path_sum_direct(grid, SMALL_GRID_SIZE - 1, SMALL_GRID_SIZE - 1))
        .collect();
    let no_cache_time = start.elapsed();
    println!("No cache (direct):           {:?}", no_cache_time);

    // NoCacheBackend with DpCache (sequential wrapper overhead)
    println!("Running NoCacheBackend (DpCache wrapper)...");
    let start = Instant::now();
    let nocache_backend_results: Vec<u32> = small_grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = DpCache::builder()
                .backend(NoCacheBackend::<(usize, usize), u32>::new())
                .problem(problem)
                .build();
            cache.get(&(SMALL_GRID_SIZE - 1, SMALL_GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let nocache_backend_time = start.elapsed();
    println!("NoCacheBackend (wrapper):    {:?}", nocache_backend_time);

    // ParallelNoCacheBackend with ParallelDpCache (parallel wrapper overhead)
    println!("Running ParallelNoCacheBackend (parallel wrapper)...");
    let start = Instant::now();
    let par_nocache_backend_results: Vec<u32> = small_grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<(usize, usize), u32>::new())
                .problem(problem)
                .build();
            cache.get(&(SMALL_GRID_SIZE - 1, SMALL_GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let par_nocache_backend_time = start.elapsed();
    println!("ParallelNoCacheBackend:      {:?}", par_nocache_backend_time);

    // ParallelNoCacheBackend + par_iter
    println!("Running ParallelNoCacheBackend + par_iter...");
    let start = Instant::now();
    let par_nocache_par_results: Vec<u32> = small_grids
        .par_iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = ParallelDpCache::builder()
                .backend(ParallelNoCacheBackend::<(usize, usize), u32>::new())
                .problem(problem)
                .build();
            cache.get(&(SMALL_GRID_SIZE - 1, SMALL_GRID_SIZE - 1)).unwrap()
        })
        .collect();
    let par_nocache_par_time = start.elapsed();
    println!("ParallelNoCacheBackend+par:  {:?}", par_nocache_par_time);

    // =========================================================================
    // Verification
    // =========================================================================
    println!("\nVerifying results...");

    // Verify big grid cached results match each other
    let mut all_match = true;
    let mut mismatches = 0;
    for i in 0..NUM_GRIDS {
        let expected = bottom_up_results[i]; // Use bottom-up as ground truth
        if expected != bottom_up_vec_results[i]
            || expected != bottom_up_par_results[i]
            || expected != bottom_up_vec_par_results[i]
            || expected != array2d_results[i]
            || expected != vec2d_results[i]
            || expected != hashmap_results[i]
            || expected != par_array2d_results[i]
            || expected != dashmap_results[i]
            || expected != rwlock_results[i]
            || expected != par_array2d_par_results[i]
            || expected != dashmap_par_results[i]
            || expected != rwlock_par_results[i]
        {
            if mismatches < 5 {
                println!(
                    "Mismatch at grid {}: bottom_up={}, Array2D={}, Vec2D={}",
                    i, expected, array2d_results[i], vec2d_results[i]
                );
            }
            all_match = false;
            mismatches += 1;
        }
    }

    // Verify small grid results against direct computation
    let small_cache_results: Vec<u32> = small_grids
        .iter()
        .map(|grid| {
            let problem = MinPathSum::new(grid);
            let cache = DpCache::builder()
                .backend(Array2DBackend::<u32, SMALL_GRID_SIZE, SMALL_GRID_SIZE>::new())
                .problem(problem)
                .build();
            cache.get(&(SMALL_GRID_SIZE - 1, SMALL_GRID_SIZE - 1)).unwrap()
        })
        .collect();

    for i in 0..NUM_SMALL_GRIDS {
        if no_cache_results[i] != small_cache_results[i]
            || no_cache_results[i] != nocache_backend_results[i]
            || no_cache_results[i] != par_nocache_backend_results[i]
            || no_cache_results[i] != par_nocache_par_results[i]
        {
            println!(
                "Mismatch at small grid {}: direct={}, nocache={}, par_nocache={}",
                i, no_cache_results[i], nocache_backend_results[i], par_nocache_backend_results[i]
            );
            all_match = false;
        }
    }

    if all_match {
        println!("✓ All backends produce identical results!");
    } else {
        println!("✗ {} mismatches found!", mismatches);
    }

    // Sample results
    println!("\nSample results:");
    for i in 0..3 {
        println!("  Grid {}: min_path_sum = {}", i, array2d_results[i]);
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Performance Summary ===");

    println!("\nLocal array cache (manual DP):");
    println!("  Bottom-up (local array):    {:?}", bottom_up_time);
    println!("  Bottom-up (Vec):            {:?}", bottom_up_vec_time);
    println!("  Bottom-up + par_iter:       {:?}", bottom_up_par_time);
    println!("  Bottom-up Vec + par_iter:   {:?}", bottom_up_vec_par_time);

    println!("\nBig grids ({}x{}) - DpCache struct:", GRID_SIZE, GRID_SIZE);
    println!("  Array2DBackend:             {:?}", array2d_time);
    println!("  Vec2DBackend:               {:?}", vec2d_time);
    println!("  HashMapBackend:             {:?}", hashmap_time);

    println!("\nBig grids - Parallel backends (sequential iteration):");
    println!("  ParallelArray2DBackend:     {:?}", par_array2d_time);
    println!("  DashMapBackend:             {:?}", dashmap_time);
    println!("  RwLockHashMapBackend:       {:?}", rwlock_time);

    println!("\nBig grids - Parallel iteration (par_iter):");
    println!("  ParallelArray2DBackend:     {:?}", par_array2d_par_time);
    println!("  DashMapBackend:             {:?}", dashmap_par_time);
    println!("  RwLockHashMapBackend:       {:?}", rwlock_par_time);

    println!("\nSmall grids ({}x{}) - Wrapper overhead:", SMALL_GRID_SIZE, SMALL_GRID_SIZE);
    println!("  No cache (direct):          {:?}", no_cache_time);
    println!("  NoCacheBackend (wrapper):   {:?}", nocache_backend_time);
    println!("  ParallelNoCacheBackend:     {:?}", par_nocache_backend_time);
    println!("  ParallelNoCacheBackend+par: {:?}", par_nocache_par_time);

    // Backend comparison
    let best_2d = array2d_time
        .min(vec2d_time)
        .min(par_array2d_time)
        .min(par_array2d_par_time);

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
    println!("  Best DpCache time: {:?}", best_2d);

    println!("\nWrapper overhead (small grids):");
    println!(
        "  NoCacheBackend vs direct: {:.2}x",
        nocache_backend_time.as_secs_f64() / no_cache_time.as_secs_f64()
    );
    println!(
        "  ParallelNoCacheBackend vs direct: {:.2}x",
        par_nocache_backend_time.as_secs_f64() / no_cache_time.as_secs_f64()
    );

    println!("\n=== Conclusion ===");
    println!(
        "For 2D grid DP, local bottom-up is {:.1}x faster than DpCache Array2DBackend.",
        array2d_time.as_secs_f64() / bottom_up_time.as_secs_f64()
    );
    println!("DpCache is useful when:");
    println!("  - Problem has complex/irregular dependencies");
    println!("  - You need the abstraction for code reuse");
    println!("  - Manual DP implementation is error-prone");
}
