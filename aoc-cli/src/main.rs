//! AOC CLI - Command-line interface for running Advent of Code solvers

mod aggregator;
mod cache;
mod cli;
mod config;
mod error;
mod executor;
mod output;

// Import aoc-solutions to link the solver plugins
use aoc_solutions as _;

use aoc_solver::SolverRegistryBuilder;
use clap::Parser;
use cli::Args;
use config::Config;
use executor::Executor;
use output::OutputFormatter;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(args: Args) -> Result<(), error::CliError> {
    // Build config from args (may not have session yet)
    let config = Config::from_args(args)?;

    // Build registry with tag filtering (only once)
    let registry = build_registry(&config.tags)?;

    // Create executor
    let mut executor =
        Executor::new(registry, &config).map_err(|e| error::CliError::Config(e.to_string()))?;

    // Collect work items
    let work_items = executor.collect_work_items();
    if work_items.is_empty() {
        println!("No solvers found matching the specified filters.");
        return Ok(());
    }

    // Check for missing inputs early
    let missing_inputs = check_missing_inputs(&work_items, &config);
    if !missing_inputs.is_empty() {
        println!("Missing {} input file(s):", missing_inputs.len());
        for (year, day) in &missing_inputs {
            println!("  - {}/day{:02}", year, day);
        }

        // If no session, prompt for one
        if config.session.is_empty() {
            println!();
            let session = config::prompt_session(
                "Session token required to fetch missing inputs from adventofcode.com",
            )?;

            // Verify and get user ID (check match if user_id was explicitly provided)
            let expected = if config.user_id_provided {
                Some(config.user_id)
            } else {
                None
            };
            let actual_user_id = config::verify_session(&session, expected)?;

            // Update executor with new session and user_id
            executor
                .update_session(session, actual_user_id)
                .map_err(|e| error::CliError::Config(e.to_string()))?;
        } else {
            println!("Will fetch missing inputs using provided session...");
        }
    }

    run_executor(executor, config.quiet)
}

/// Check which inputs are missing from cache
fn check_missing_inputs(work_items: &[executor::WorkItem], config: &Config) -> Vec<(u16, u8)> {
    let cache = cache::InputCache::new(config.cache_dir.clone(), config.user_id);
    work_items
        .iter()
        .filter(|w| !cache.contains(w.year, w.day))
        .map(|w| (w.year, w.day))
        .collect()
}

/// Run the executor and collect results
fn run_executor(executor: Executor, quiet: bool) -> Result<(), error::CliError> {
    let work_items = executor.collect_work_items();
    println!("Running {} solver(s)...", work_items.len());

    // Build expected keys for result aggregation
    let expected_keys: Vec<aggregator::ResultKey> = work_items
        .iter()
        .flat_map(|w| {
            w.parts.clone().map(move |p| aggregator::ResultKey {
                year: w.year,
                day: w.day,
                part: p,
            })
        })
        .collect();

    // Set up result channel
    let (tx, rx) = std::sync::mpsc::channel();

    // Run executor in background thread
    let executor_handle = std::thread::spawn(move || executor.execute(tx));

    // Collect and display results in order using aggregator
    let formatter = OutputFormatter::new(quiet);
    let mut aggregator = aggregator::ResultAggregator::new(expected_keys);
    let mut results = Vec::new();

    for result in rx {
        // Add to aggregator and print any results that are ready (in order)
        for ready in aggregator.add(result) {
            formatter.print_result(&ready);
            results.push(ready);
        }
    }

    // Drain any remaining buffered results (shouldn't happen if all results arrived)
    for ready in aggregator.drain() {
        formatter.print_result(&ready);
        results.push(ready);
    }

    // Verify all expected results were received
    if !aggregator.is_complete() {
        eprintln!("Warning: Not all expected results were received");
    }

    // Wait for executor to finish
    executor_handle
        .join()
        .map_err(|_| error::CliError::Config("Executor thread panicked".to_string()))?
        .map_err(error::CliError::Executor)?;

    // Print summary
    formatter.print_summary(&results);

    Ok(())
}

/// Build registry with tag filtering
fn build_registry(tags: &[String]) -> Result<aoc_solver::SolverRegistry, error::CliError> {
    let builder = SolverRegistryBuilder::new();

    let builder = if tags.is_empty() {
        builder.register_all_plugins()?
    } else {
        builder.register_solver_plugins(|plugin| {
            tags.iter().all(|tag| plugin.tags.contains(&tag.as_str()))
        })?
    };

    Ok(builder.build())
}
