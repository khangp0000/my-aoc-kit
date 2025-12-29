//! Example demonstrating using #[derive(AocSolver)] with manual plugin registration
//!
//! This example shows how to combine the AocSolver derive macro with the plugin
//! system for automatic solver discovery using manual inventory::submit!.
//!
//! Run with: cargo run --example combined_macros

use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError, SolverRegistryBuilder};

/// Example solver using the macro
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day1;

impl AocParser for Day1 {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect()
    }
}

impl PartSolver<1> for Day1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for Day1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

// Manually register the solver with the plugin system
aoc_solver::inventory::submit! {
    aoc_solver::SolverPlugin {
        year: 2023,
        day: 1,
        solver: &Day1,
        tags: &["example", "simple"],
    }
}

/// Another solver for day 2
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day2;

impl AocParser for Day2 {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect()
    }
}

impl PartSolver<1> for Day2 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared
            .iter()
            .filter(|&&x| x % 2 == 0)
            .sum::<i32>()
            .to_string())
    }
}

impl PartSolver<2> for Day2 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared
            .iter()
            .filter(|&&x| x % 2 != 0)
            .sum::<i32>()
            .to_string())
    }
}

// Register day 2
aoc_solver::inventory::submit! {
    aoc_solver::SolverPlugin {
        year: 2023,
        day: 2,
        solver: &Day2,
        tags: &["example", "filtering"],
    }
}

fn main() {
    println!("=== Combined Macros Example ===\n");

    // Build registry with all registered plugins
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    println!("Registered solvers:");

    // Solve Day 1
    let input1 = "1\n2\n3\n4\n5";
    println!("\nDay 1 Input: {}", input1.replace('\n', ", "));

    let mut solver1 = registry
        .create_solver(2023, 1, input1)
        .expect("Failed to create Day 1 solver");

    match solver1.solve(1) {
        Ok(result) => println!("Day 1, Part 1 (Sum): {} (took {:?})", result.answer, result.duration()),
        Err(e) => eprintln!("Error: {}", e),
    }

    match solver1.solve(2) {
        Ok(result) => println!("Day 1, Part 2 (Product): {} (took {:?})", result.answer, result.duration()),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("Parse took {:?}", solver1.parse_duration());

    // Solve Day 2
    let input2 = "1\n2\n3\n4\n5\n6";
    println!("\nDay 2 Input: {}", input2.replace('\n', ", "));

    let mut solver2 = registry
        .create_solver(2023, 2, input2)
        .expect("Failed to create Day 2 solver");

    match solver2.solve(1) {
        Ok(result) => println!("Day 2, Part 1 (Sum of evens): {} (took {:?})", result.answer, result.duration()),
        Err(e) => eprintln!("Error: {}", e),
    }

    match solver2.solve(2) {
        Ok(result) => println!("Day 2, Part 2 (Sum of odds): {} (took {:?})", result.answer, result.duration()),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("Parse took {:?}", solver2.parse_duration());

    println!("\n=== Benefits ===");
    println!("✓ AocParser + PartSolver<N> provide clean separation of concerns");
    println!("✓ #[derive(AocSolver)] generates Solver trait implementation");
    println!("✓ Manual plugin registration enables automatic discovery");
    println!("✓ Registry can filter by tags, year, or custom predicates");
}
