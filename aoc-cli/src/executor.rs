//! Parallel executor for running solvers

use crate::cache::InputCache;
use crate::cli::ParallelizeBy;
use crate::config::Config;
use crate::error::{ArcExecutorError, ExecutorError};
use aoc_http_client::AocClient;
use aoc_solver::{DynSolver, SolverRegistry};
use chrono::{DateTime, Local};
use itertools::Itertools;
use rayon::prelude::*;
use std::ops::RangeInclusive;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

/// Submission outcome from AoC
#[derive(Debug, Clone)]
pub enum SubmissionOutcome {
    Correct,
    Incorrect,
    AlreadyCompleted,
    Throttled { wait_time: Option<Duration> },
    Error(String),
}

/// Result from a single solver execution
pub struct SolverResult {
    pub year: u16,
    pub day: u8,
    pub part: u8,
    pub answer: Result<String, aoc_solver::SolverError>,
    pub solve_duration: Duration,
    pub submitted_at: Option<DateTime<Local>>,
    pub submission: Option<SubmissionOutcome>,
    pub submission_wait: Option<Duration>,
}

/// Work item representing a solver to execute
pub struct WorkItem {
    pub year: u16,
    pub day: u8,
    pub parts: RangeInclusive<u8>,
}

/// Parallel executor for running solvers
pub struct Executor {
    sync_executor_config: SyncExecutorConfig,
    thread_pool: rayon::ThreadPool,
}

pub struct SyncExecutorConfig {
    registry: SolverRegistry,
    cache: InputCache,
    client: Option<AocClient>,
    session: Zeroizing<String>,
    submit: bool,
    auto_retry: bool,
    parallelize_by: ParallelizeBy,
    year_filter: Option<u16>,
    day_filter: Option<u8>,
    part_filter: Option<u8>,
}

impl Executor {
    /// Create a new executor from config
    pub fn new(registry: SolverRegistry, config: &Config) -> Result<Self, ExecutorError> {
        let client = if config.submit || !config.session.is_empty() {
            Some(AocClient::new().map_err(|e| ExecutorError::InputFetch {
                year: 0,
                day: 0,
                source: Box::new(e),
            })?)
        } else {
            None
        };

        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(config.thread_count)
            .build()
            .map_err(|e| ExecutorError::ThreadPool(e.to_string()))?;

        Ok(Self {
            sync_executor_config: SyncExecutorConfig {
                registry,
                cache: InputCache::new(config.cache_dir.as_path().into(), config.user_id),
                client,
                session: config.session.clone(),
                submit: config.submit,
                auto_retry: config.auto_retry,
                parallelize_by: config.parallelize_by,
                year_filter: config.year_filter,
                day_filter: config.day_filter,
                part_filter: config.part_filter,
            },
            thread_pool,
        })
    }

    /// Collect work items by filtering from registry metadata
    pub fn collect_work_items(&self) -> Vec<WorkItem> {
        let cfg = &self.sync_executor_config;
        cfg.registry
            .storage()
            .iter_info()
            .filter(|info| cfg.year_filter.is_none_or(|y| info.year == y))
            .filter(|info| cfg.day_filter.is_none_or(|d| info.day == d))
            .map(|info| WorkItem {
                year: info.year,
                day: info.day,
                parts: self.filter_parts(info.parts),
            })
            .filter(|w| !w.parts.is_empty())
            .collect()
    }

    /// Filter parts based on config.part_filter and solver's max parts
    #[allow(clippy::reversed_empty_ranges)]
    fn filter_parts(&self, max_parts: u8) -> RangeInclusive<u8> {
        match self.sync_executor_config.part_filter {
            Some(p) if p <= max_parts => p..=p,
            Some(_) => 1..=0, // Empty range - intentional
            None => 1..=max_parts,
        }
    }

    /// Execute all work items and send results to channel
    pub fn execute(&self, tx: Sender<SolverResult>) -> Result<(), ArcExecutorError> {
        let work_items = self.collect_work_items();

        match self.sync_executor_config.parallelize_by {
            ParallelizeBy::Sequential => {
                // No parallelization, execute all in order
                let mut collected_error: Option<ArcExecutorError> = None;
                for work in work_items {
                    if let Err(e) = self.run_solver(&work, &tx) {
                        collected_error = Some(ArcExecutorError::combine_opt(collected_error, e));
                    }
                }
                collected_error.map_or(Ok(()), Err)
            }
            ParallelizeBy::Year => {
                // Group by year, parallelize years using configured thread pool
                let by_year: Vec<Vec<WorkItem>> = work_items
                    .into_iter()
                    .chunk_by(|w| w.year)
                    .into_iter()
                    .map(|(_, group)| group.collect())
                    .collect();

                self.execute_parallel_grouped(by_year, &tx)
            }
            // Day and Part both parallelize across all work items (Part differs in run_solver_parallel behavior)
            ParallelizeBy::Day | ParallelizeBy::Part => self.execute_parallel(work_items, &tx),
        }
    }

    /// Execute work items in parallel, collecting errors
    fn execute_parallel(
        &self,
        work_items: Vec<WorkItem>,
        tx: &Sender<SolverResult>,
    ) -> Result<(), ArcExecutorError> {
        let sync_executor_config = &self.sync_executor_config;

        self.thread_pool.install(|| {
            work_items
                .into_par_iter()
                .map(|work| run_solver_parallel(&work, tx, sync_executor_config).err())
                .reduce_with(|err1, err2| {
                    err1.map(|err1| ArcExecutorError::combine_opt(err2, err1))
                })
                .unwrap_or_default()
                .map_or(Ok(()), Err)
        })
    }

    /// Execute grouped work items in parallel (for year-level parallelism)
    fn execute_parallel_grouped(
        &self,
        groups: Vec<Vec<WorkItem>>,
        tx: &Sender<SolverResult>,
    ) -> Result<(), ArcExecutorError> {
        let sync_executor_config = &self.sync_executor_config;

        self.thread_pool.install(|| {
            groups
                .into_par_iter()
                .map(|items| {
                    let mut err = None;
                    for work in items {
                        if let Err(e) = run_solver_parallel(&work, tx, sync_executor_config) {
                            err = Some(ArcExecutorError::combine_opt(err, e))
                        }
                    }
                    err
                })
                .reduce_with(|err1, err2| {
                    err1.map(|err1| ArcExecutorError::combine_opt(err2, err1))
                })
                .unwrap_or_default()
                .map_or(Ok(()), Err)
        })
    }

    /// Run a single solver for specified parts (used for sequential mode)
    fn run_solver(
        &self,
        work: &WorkItem,
        tx: &Sender<SolverResult>,
    ) -> Result<(), ArcExecutorError> {
        run_solver_parallel(work, tx, &self.sync_executor_config)
    }
}

/// Create an error result for a failed input fetch
fn make_error_result(year: u16, day: u8, part: u8, error: &str) -> SolverResult {
    SolverResult {
        year,
        day,
        part,
        answer: Err(aoc_solver::SolverError::ParseError(
            aoc_solver::ParseError::InvalidFormat(error.to_string()),
        )),
        solve_duration: Duration::ZERO,
        submitted_at: None,
        submission: None,
        submission_wait: None,
    }
}

/// Send result with optional submission
fn send_result(
    tx: &Sender<SolverResult>,
    mut result: SolverResult,
    client: Option<&AocClient>,
    session: &str,
    submit: bool,
    auto_retry: bool,
) -> Result<(), ArcExecutorError> {
    if submit {
        submit_result_internal(&mut result, client, session, auto_retry);
    }
    tx.send(result)
        .map_err(|_| ExecutorError::ChannelSend.into())
}

/// Free function for parallel solver execution
fn run_solver_parallel(
    work: &WorkItem,
    tx: &Sender<SolverResult>,
    sync_executor_config: &SyncExecutorConfig,
) -> Result<(), ArcExecutorError> {
    let parallelize_by = sync_executor_config.parallelize_by;

    let input = match get_input_parallel(work, sync_executor_config) {
        Ok(input) => input,
        Err(e) => {
            // Send error result for each part
            let error_msg = e.to_string();
            for part in work.parts.clone() {
                tx.send(make_error_result(work.year, work.day, part, &error_msg))
                    .map_err(|_| ArcExecutorError::from(ExecutorError::ChannelSend))?;
            }
            return Ok(());
        }
    };

    if matches!(parallelize_by, ParallelizeBy::Part) {
        run_solver_parts_parallel(work, &input, tx, sync_executor_config)
    } else {
        run_solver_sequential(work, &input, tx, sync_executor_config)
    }
}

/// Run solver with part-level parallelism, buffering results to emit in order
fn run_solver_parts_parallel(
    work: &WorkItem,
    input: &str,
    tx: &Sender<SolverResult>,
    sync_executor_config: &SyncExecutorConfig,
) -> Result<(), ArcExecutorError> {
    let (result_tx, result_rx) = std::sync::mpsc::channel();
    let (year, day) = (work.year, work.day);
    let registry = &sync_executor_config.registry;
    let session = &sync_executor_config.session;
    let client = &sync_executor_config.client;
    let submit = sync_executor_config.submit;
    let auto_retry = sync_executor_config.auto_retry;

    // Solve parts in parallel
    work.parts
        .clone()
        .into_par_iter()
        .for_each_with(result_tx, |rtx, part| {
            let mut solver = registry.create_solver(year, day, input).unwrap();
            rtx.send(solve_part_internal(year, day, part, &mut *solver))
                .ok();
        });

    // Buffer and emit results in part order
    let mut buffer: [Option<SolverResult>; 2] = [None, None];
    let start_part = *work.parts.start();
    let mut next_part = start_part;

    for result in result_rx {
        let idx = (result.part - start_part) as usize;
        if idx < buffer.len() {
            buffer[idx] = Some(result);
        }
        // Emit buffered results in order
        while let Some(result) = buffer
            .get_mut((next_part - start_part) as usize)
            .and_then(Option::take)
        {
            send_result(tx, result, client.as_ref(), session, submit, auto_retry)?;
            next_part += 1;
        }
    }
    Ok(())
}

/// Run solver sequentially in background, submit as results arrive
fn run_solver_sequential(
    work: &WorkItem,
    input: &str,
    tx: &Sender<SolverResult>,
    sync_executor_config: &SyncExecutorConfig,
) -> Result<(), ArcExecutorError> {
    let (solve_tx, solve_rx) = std::sync::mpsc::channel();
    let (year, day) = (work.year, work.day);
    let parts = work.parts.clone();
    let registry = &sync_executor_config.registry;
    let session = &sync_executor_config.session;
    let client = &sync_executor_config.client;
    let submit = sync_executor_config.submit;
    let auto_retry = sync_executor_config.auto_retry;
    std::thread::scope(|s| {
        s.spawn(move || {
            let mut solver = registry.create_solver(year, day, input).unwrap();
            for part in parts {
                if solve_tx
                    .send(solve_part_internal(year, day, part, &mut *solver))
                    .is_err()
                {
                    break;
                }
            }
        });

        for result in solve_rx {
            send_result(tx, result, client.as_ref(), session, submit, auto_retry)?
        }
        Ok(())
    })
}

/// Get input for a year/day, using cache or fetching (free function version)
fn get_input_parallel(
    work: &WorkItem,
    sync_executor_config: &SyncExecutorConfig,
) -> Result<String, ExecutorError> {
    let (year, day) = (work.year, work.day);
    let cache = &sync_executor_config.cache;
    let session = &sync_executor_config.session;
    let client = sync_executor_config.client.as_ref();
    // Check cache first
    if let Some(input) = cache
        .get(year, day)
        .map_err(|e| ExecutorError::InputFetch {
            year,
            day,
            source: Box::new(e),
        })?
    {
        return Ok(input);
    }

    // Fetch from AoC
    let client = client.ok_or_else(|| ExecutorError::InputFetch {
        year,
        day,
        source: Box::new(std::io::Error::other("No HTTP client available")),
    })?;

    let input = client
        .get_input(year, day, session)
        .map_err(|e| ExecutorError::InputFetch {
            year,
            day,
            source: Box::new(e),
        })?;

    // Cache the input (warn on failure, don't fail the operation)
    if let Err(e) = cache.put(year, day, &input) {
        eprintln!(
            "Warning: {}",
            ExecutorError::CacheWrite {
                year,
                day,
                message: e.to_string(),
            }
        );
    }

    Ok(input)
}

/// Solve a single part (free function)
fn solve_part_internal(year: u16, day: u8, part: u8, solver: &mut dyn DynSolver) -> SolverResult {
    let start = Instant::now();
    let answer = solver.solve(part);

    SolverResult {
        year,
        day,
        part,
        answer: answer.map_err(Into::into),
        solve_duration: start.elapsed(),
        submitted_at: None,
        submission: None,
        submission_wait: None,
    }
}

/// Submit a result (free function version)
fn submit_result_internal(
    result: &mut SolverResult,
    client: Option<&AocClient>,
    session: &str,
    auto_retry: bool,
) {
    if let Ok(ref ans) = result.answer {
        let (outcome, wait) = submit_with_retry_internal(
            result.year,
            result.day,
            result.part,
            ans,
            client,
            session,
            auto_retry,
        );
        result.submitted_at = Some(Local::now());
        result.submission = outcome;
        result.submission_wait = wait;
    }
}

/// Submit answer with optional retry on throttle (free function version)
fn submit_with_retry_internal(
    year: u16,
    day: u8,
    part: u8,
    answer: &str,
    client: Option<&AocClient>,
    session: &str,
    auto_retry: bool,
) -> (Option<SubmissionOutcome>, Option<Duration>) {
    let client = match client {
        Some(c) => c,
        None => {
            return (
                Some(SubmissionOutcome::Error("No HTTP client".into())),
                None,
            );
        }
    };

    let mut total_wait = Duration::ZERO;

    loop {
        match client.submit_answer(year, day, part, answer, session) {
            Ok(aoc_http_client::SubmissionResult::Correct) => {
                return (Some(SubmissionOutcome::Correct), Some(total_wait));
            }
            Ok(aoc_http_client::SubmissionResult::Incorrect) => {
                return (Some(SubmissionOutcome::Incorrect), Some(total_wait));
            }
            Ok(aoc_http_client::SubmissionResult::AlreadyCompleted) => {
                return (Some(SubmissionOutcome::AlreadyCompleted), Some(total_wait));
            }
            Ok(aoc_http_client::SubmissionResult::Throttled { wait_time }) => {
                if auto_retry && let Some(wait) = wait_time {
                    std::thread::sleep(wait);
                    total_wait += wait;
                    continue;
                }
                return (
                    Some(SubmissionOutcome::Throttled { wait_time }),
                    Some(total_wait),
                );
            }
            Err(e) => {
                return (
                    Some(SubmissionOutcome::Error(e.to_string())),
                    Some(total_wait),
                );
            }
        }
    }
}
