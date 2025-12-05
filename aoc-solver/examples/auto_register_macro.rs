//! Example demonstrating #[aoc_solver] with #[derive(AutoRegisterSolver)]
//!
//! This example shows how to combine both macros for maximum convenience:
//! - #[aoc_solver] generates the Solver trait implementation
//! - #[derive(AutoRegisterSolver)] automatically registers with the plugin system
//!
//! Run with: cargo run --example auto_register_macro

use aoc_solver::{AutoRegisterSolver, ParseError, RegistryBuilder};
use aoc_solver_macros::aoc_solver;

/// Day 1 solver with automatic registration
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["example", "simple"])]
struct Day1;

#[aoc_solver(max_parts = 2)]
impl Day1 {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect()
    }

    fn part1(shared: &mut Vec<i32>) -> String {
        shared.iter().sum::<i32>().to_string()
    }

    fn part2(shared: &mut Vec<i32>) -> String {
        shared.iter().product::<i32>().to_string()
    }
}

/// Day 2 solver with automatic registration
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 2, tags = ["example", "filtering"])]
struct Day2;

#[aoc_solver(max_parts = 2)]
impl Day2 {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect()
    }

    fn part1(shared: &mut Vec<i32>) -> String {
        shared
            .iter()
            .filter(|&&x| x % 2 == 0)
            .sum::<i32>()
            .to_string()
    }

    fn part2(shared: &mut Vec<i32>) -> String {
        shared
            .iter()
            .filter(|&&x| x % 2 != 0)
            .sum::<i32>()
            .to_string()
    }
}

fn main() {
    println!("=== Auto-Register with Macro Example ===\n");

    // Build registry - all solvers are automatically discovered!
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    println!("✓ All solvers automatically registered via #[derive(AutoRegisterSolver)]\n");

    // Solve Day 1
    let input1 = "1\n2\n3\n4\n5";
    println!("Day 1 Input: {}", input1.replace('\n', ", "));

    let mut solver1 = registry
        .create_solver(2023, 1, input1)
        .expect("Failed to create Day 1 solver");

    match solver1.solve(1) {
        Ok(answer) => println!("Day 1, Part 1 (Sum): {}", answer),
        Err(e) => eprintln!("Error: {}", e),
    }

    match solver1.solve(2) {
        Ok(answer) => println!("Day 1, Part 2 (Product): {}", answer),
        Err(e) => eprintln!("Error: {}", e),
    }

    // Solve Day 2
    let input2 = "1\n2\n3\n4\n5\n6";
    println!("\nDay 2 Input: {}", input2.replace('\n', ", "));

    let mut solver2 = registry
        .create_solver(2023, 2, input2)
        .expect("Failed to create Day 2 solver");

    match solver2.solve(1) {
        Ok(answer) => println!("Day 2, Part 1 (Sum of evens): {}", answer),
        Err(e) => eprintln!("Error: {}", e),
    }

    match solver2.solve(2) {
        Ok(answer) => println!("Day 2, Part 2 (Sum of odds): {}", answer),
        Err(e) => eprintln!("Error: {}", e),
    }

    println!("\n=== Benefits ===");
    println!("✓ #[aoc_solver] eliminates Solver trait boilerplate");
    println!("✓ #[derive(AutoRegisterSolver)] enables automatic discovery");
    println!("✓ Both macros work together seamlessly");
    println!("✓ Zero manual registration code needed!");
}
