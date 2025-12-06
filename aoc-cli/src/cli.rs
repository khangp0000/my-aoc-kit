//! CLI argument parsing using clap

use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Parallelization level for solver execution
#[derive(Debug, Clone, Copy, Default, ValueEnum, PartialEq, Eq)]
pub enum ParallelizeBy {
    /// No parallelization; execute all solvers sequentially in order
    Sequential,
    /// Parallelize across years; days and parts run sequentially within each year
    Year,
    /// Parallelize across year/day combinations; parts run sequentially (default)
    #[default]
    Day,
    /// Parallelize across all year/day/part combinations
    Part,
}

/// Advent of Code solver runner
#[derive(Parser, Debug)]
#[command(name = "aoc", about = "Run Advent of Code solvers", version)]
pub struct Args {
    /// Year to run (runs all years if omitted)
    #[arg(short, long)]
    pub year: Option<u16>,

    /// Day to run (runs all days if omitted)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=25))]
    pub day: Option<u8>,

    /// Part to run (runs all parts if omitted)
    #[arg(short, long, value_parser = clap::value_parser!(u8).range(1..=2))]
    pub part: Option<u8>,

    /// Tags to filter solvers (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub tags: Vec<String>,

    /// Cache directory for puzzle inputs
    #[arg(long, default_value = "~/.cache/aoc_solver")]
    pub cache_dir: PathBuf,

    /// Number of threads for parallel execution
    #[arg(long)]
    pub threads: Option<usize>,

    /// Parallelization level: sequential, year, day, or part
    #[arg(long, value_enum, default_value = "day")]
    pub parallelize_by: ParallelizeBy,

    /// Submit answers to Advent of Code
    #[arg(long)]
    pub submit: bool,

    /// User ID for cache organization and verification
    #[arg(long)]
    pub user_id: Option<u64>,

    /// Auto-retry on throttle with parsed wait time
    #[arg(long, default_value = "false")]
    pub auto_retry: bool,

    /// Quiet mode - only output answers
    #[arg(short, long)]
    pub quiet: bool,
}
