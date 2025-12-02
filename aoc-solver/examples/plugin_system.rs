//! Example demonstrating the plugin system and builder pattern
//!
//! This example shows how to use the inventory-based plugin system for
//! automatic solver registration, along with the fluent builder API.
//!
//! Run with: cargo run --example plugin_system

use aoc_solver::{
    AutoRegisterSolver, ParseError, PartResult, RegistryBuilder, SolveError, Solver, SolverPlugin,
};

// ============================================================================
// Plugin Day 1: Simple solver tagged as "easy" and "2023"
// Using the derive macro (RECOMMENDED)
// ============================================================================

#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["easy", "2023"])]
pub struct PluginDay1;

impl Solver for PluginDay1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();

    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat(format!("Expected integer: {}", line)))
            })
            .collect()
    }

    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        _previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => Ok(PartResult {
                answer: parsed.iter().sum::<i32>().to_string(),
                partial: None,
            }),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

// ============================================================================
// Plugin Day 2: Another solver tagged as "hard" and "2023"
// Using manual inventory::submit! (for comparison)
// ============================================================================

pub struct PluginDay2;

impl Solver for PluginDay2 {
    type Parsed = Vec<i32>;
    type PartialResult = ();

    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat(format!("Expected integer: {}", line)))
            })
            .collect()
    }

    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        _previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => Ok(PartResult {
                answer: parsed.iter().product::<i32>().to_string(),
                partial: None,
            }),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

// Register PluginDay2 with tags (manual approach)
inventory::submit! {
    SolverPlugin {
        year: 2023,
        day: 2,
        solver: &PluginDay2,
        tags: &["hard", "2023"],
    }
}

// ============================================================================
// Plugin Day 3: Solver for 2024 tagged as "easy" and "2024"
// Using manual inventory::submit! (for comparison)
// ============================================================================

pub struct PluginDay3;

impl Solver for PluginDay3 {
    type Parsed = Vec<i32>;
    type PartialResult = ();

    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat(format!("Expected integer: {}", line)))
            })
            .collect()
    }

    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        _previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => {
                let max = parsed.iter().max().copied().unwrap_or(0);
                Ok(PartResult {
                    answer: max.to_string(),
                    partial: None,
                })
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

// Register PluginDay3 with tags (manual approach)
inventory::submit! {
    SolverPlugin {
        year: 2024,
        day: 3,
        solver: &PluginDay3,
        tags: &["easy", "2024"],
    }
}

// ============================================================================
// Plugin Day 4: Using the derive macro (RECOMMENDED APPROACH)
// ============================================================================

#[derive(AutoRegisterSolver)]
#[aoc(year = 2024, day = 4, tags = ["derive", "easy"])]
pub struct PluginDay4Derive;

impl Solver for PluginDay4Derive {
    type Parsed = Vec<i32>;
    type PartialResult = ();

    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat(format!("Expected integer: {}", line)))
            })
            .collect()
    }

    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        _previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => {
                let min = parsed.iter().min().copied().unwrap_or(0);
                Ok(PartResult {
                    answer: min.to_string(),
                    partial: None,
                })
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

// Note: No manual inventory::submit! needed - the derive macro handles it!

// ============================================================================
// Main function demonstrating different registration scenarios
// ============================================================================

fn main() {
    println!("=== Plugin System and Builder Pattern Example ===\n");

    let input = "1\n2\n3\n4\n5";

    // Scenario 1: Register ALL plugins
    println!("--- Scenario 1: Register All Plugins ---");
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    // Test all registered solvers
    if let Ok(mut solver) = registry.create_solver(2023, 1, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 1 Part 1: {}", answer);
        }
    }
    if let Ok(mut solver) = registry.create_solver(2023, 2, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 2 Part 1: {}", answer);
        }
    }
    if let Ok(mut solver) = registry.create_solver(2024, 3, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2024 Day 3 Part 1: {}", answer);
        }
    }
    if let Ok(mut solver) = registry.create_solver(2024, 4, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2024 Day 4 Part 1 (derive macro): {}", answer);
        }
    }

    // Scenario 2: Register only "easy" solvers
    println!("\n--- Scenario 2: Register Only 'Easy' Solvers ---");
    let registry = RegistryBuilder::new()
        .register_solver_plugins(|plugin| plugin.tags.contains(&"easy"))
        .expect("Failed to register plugins")
        .build();

    // Only easy solvers should be registered (Day 1 and Day 3)
    if let Ok(mut solver) = registry.create_solver(2023, 1, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 1 Part 1 (easy): {}", answer);
        }
    }
    match registry.create_solver(2023, 2, input) {
        Ok(_) => println!("2023 Day 2 was registered (unexpected!)"),
        Err(_) => println!("2023 Day 2 not registered (expected - it's 'hard')"),
    }
    if let Ok(mut solver) = registry.create_solver(2024, 3, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2024 Day 3 Part 1 (easy): {}", answer);
        }
    }

    // Scenario 3: Register only 2023 solvers
    println!("\n--- Scenario 3: Register Only 2023 Solvers ---");
    let registry = RegistryBuilder::new()
        .register_solver_plugins(|plugin| plugin.year == 2023)
        .expect("Failed to register plugins")
        .build();

    // Only 2023 solvers should be registered
    if let Ok(mut solver) = registry.create_solver(2023, 1, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 1 Part 1: {}", answer);
        }
    }
    if let Ok(mut solver) = registry.create_solver(2023, 2, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 2 Part 1: {}", answer);
        }
    }
    match registry.create_solver(2024, 3, input) {
        Ok(_) => println!("2024 Day 3 was registered (unexpected!)"),
        Err(_) => println!("2024 Day 3 not registered (expected - it's 2024)"),
    }

    // Scenario 4: Mix manual registration with plugin registration
    println!("\n--- Scenario 4: Mix Manual and Plugin Registration ---");
    let registry = RegistryBuilder::new()
        .register(2022, 1, |input: &str| {
            // Manual registration for a custom solver
            let parsed: Vec<i32> = input
                .lines()
                .filter_map(|line| line.trim().parse().ok())
                .collect();
            Ok(Box::new(aoc_solver::SolverInstance::<PluginDay1>::new(
                2022, 1, parsed,
            )))
        })
        .expect("Failed to register manual solver")
        .register_solver_plugins(|plugin| plugin.tags.contains(&"easy"))
        .expect("Failed to register plugins")
        .build();

    // Both manual and plugin solvers should work
    if let Ok(mut solver) = registry.create_solver(2022, 1, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2022 Day 1 Part 1 (manual): {}", answer);
        }
    }
    if let Ok(mut solver) = registry.create_solver(2023, 1, input) {
        if let Ok(answer) = solver.solve(1) {
            println!("2023 Day 1 Part 1 (plugin): {}", answer);
        }
    }

    println!("\n=== Benefits of the Plugin System ===");
    println!("✓ No manual registration calls needed");
    println!("✓ Solvers can be defined in separate modules/crates");
    println!("✓ Automatic discovery at runtime");
    println!("✓ Flexible filtering by tags, year, or custom predicates");
    println!("✓ Fluent builder API for readable construction");
    println!("✓ Compile-time guarantee of immutability after build");
    println!("✓ Derive macro eliminates boilerplate (see PluginDay4Derive)");
}
