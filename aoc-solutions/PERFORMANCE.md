# DP Cache Performance Guide

This document summarizes benchmark results and provides guidance on when and how to use the DP cache system.

## Quick Summary

| Approach | Best Use Case | Overhead vs Manual |
|----------|---------------|-------------------|
| Manual iterative | Simple linear dependencies | 1x (baseline) |
| Manual bottom-up array | Known bounds, regular grid | 2-3x slower than iterative |
| Manual memoized recursive | Complex dependencies | 7-10x slower than iterative |
| DpCache + ArrayBackend | Reusable abstractions | 30-100x slower than manual |
| DpCache + par_iter | Parallel workloads | Best for multi-core |

## When to Use DpCache

**Use DpCache when:**
- Problem has complex/irregular dependencies that are hard to order manually
- You need code reuse across different DP problems
- Manual DP implementation would be error-prone
- You're processing many independent problems in parallel

**Don't use DpCache when:**
- A simple iterative solution exists (like Fibonacci)
- Performance is critical and you can write manual DP
- The problem has simple, predictable dependencies

## Benchmark Results

### Fibonacci (1D DP, 1000 queries, n: 0-185)

| Implementation | Time | vs Iterative |
|----------------|------|--------------|
| Iterative (optimal) | 63µs | 1x |
| Bottom-up local array | 146µs | 2.3x slower |
| Memoized recursive | 483µs | 7.6x slower |
| DpCache ArrayBackend | 4.9ms | 78x slower |
| DpCache + par_iter (best) | 3.7ms | 59x slower |

**Key insight:** For Fibonacci, iterative is 59-78x faster than any DpCache approach. The DpCache abstraction adds significant overhead from trait dispatch, builder pattern, and generic code.

### Grid Path (2D DP, 100 grids of 50x50)

| Implementation | Time | vs Manual |
|----------------|------|-----------|
| Bottom-up local array | 181µs | 1x |
| Bottom-up Vec | 401µs | 2.2x slower |
| DpCache Array2DBackend | 17.9ms | 99x slower |
| DpCache Vec2DBackend | 20.1ms | 111x slower |
| DpCache + par_iter (best) | 6.6ms | 36x slower |

**Key insight:** For 2D grid problems, manual bottom-up is 99x faster than DpCache. The overhead comes from recursive memoization vs simple nested loops.

### Collatz (Sparse keys, 50000 queries)

| Implementation | Time | Notes |
|----------------|------|-------|
| No cache (direct) | 12ms | Baseline |
| HashMapBackend | 192ms | 16x slower |
| No cache + par_iter | 2ms | Best overall |
| DashMapBackend + par_iter | 48ms | 4x slower than no-cache parallel |

**Key insight:** For Collatz, caching provides no benefit because each query has unique subproblems. Direct computation with parallelization wins.

### LCS - Longest Common Subsequence (2D DP, 100 pairs of 99-char strings)

| Implementation | Time | vs Manual |
|----------------|------|-----------|
| Bottom-up local array | 1.16ms | 1x |
| Bottom-up Vec | 1.82ms | 1.6x slower |
| DpCache Array2DBackend | 38.3ms | 33x slower |
| DpCache Vec2DBackend | 49.1ms | 42x slower |
| DpCache + par_iter (best) | 18.7ms | 16x slower |

**Key insight:** LCS has regular 2D dependencies, making manual DP straightforward. DpCache adds 33x overhead.

### Pattern (Simple computation, 1100 queries)

| Implementation | Time | Notes |
|----------------|------|-------|
| No cache (direct) | 15µs | Baseline |
| ArrayBackend | 2.3ms | 160x slower |
| No cache + par_iter | 857µs | Parallel overhead |

**Key insight:** For simple O(n) computations, caching adds pure overhead with no benefit.

## Backend Selection Guide

### Single-threaded

| Backend | Best For | Performance |
|---------|----------|-------------|
| ArrayBackend | Known bounds, dense keys | Fastest (zero allocation) |
| Array2DBackend | 2D grids with known size | Fastest for 2D |
| VecBackend | Unknown bounds, dense keys | ~1.2x slower than Array |
| Vec2DBackend | 2D grids, dynamic size | ~1.2x slower than Array2D |
| HashMapBackend | Sparse/irregular keys | ~2x slower than Array |

### Parallel (with par_iter)

| Backend | Best For | Notes |
|---------|----------|-------|
| DashMapBackend | General parallel use | Best overall parallel performance |
| ParallelArrayBackend | Known bounds | Good for dense keys |
| RwLockHashMapBackend | Avoid | Severe lock contention |

**Critical:** Always use `par_iter()` with parallel backends. Sequential iteration with parallel backends is 30-100x slower due to lock contention.

## Performance Hierarchy

```
Fastest ──────────────────────────────────────────────► Slowest

Manual      Manual       Manual        DpCache      DpCache
Iterative → Bottom-up → Memoized  →  Sequential → Parallel
            Array        Recursive                  (no par_iter)
   1x         2-3x         7-10x       30-100x      100-1000x
```

## Recommendations

1. **Default to manual DP** for performance-critical code
2. **Use DpCache** when abstraction/reusability matters more than raw speed
3. **Always use par_iter** with parallel backends
4. **Prefer ArrayBackend** for single-threaded, DashMapBackend for parallel
5. **Avoid RwLockHashMapBackend** - it has severe contention issues

## Running Benchmarks

```bash
# Run all benchmarks
cargo run --example fibonacci_benchmark --release
cargo run --example grid_path_benchmark --release
cargo run --example lcs_benchmark --release
cargo run --example collatz_benchmark --release
cargo run --example pattern_benchmark --release
```
