//! Example solver with dependent parts
//!
//! This example demonstrates a case where Part 2 can use data from Part 1.
//! Part 1 calculates the sum and count, and Part 2 uses that data to calculate
//! the average (or can compute independently if Part 1 wasn't run).
//!
//! Run with: cargo run --example dependent_parts

use aoc_solver::{
    AocParser, AocSolver, AutoRegisterSolver, ParseError, PartSolver, SolveError,
    SolverRegistryBuilder,
};
use std::borrow::Cow;

/// Shared data that can be mutated by parts to pass information
#[derive(Debug, Clone)]
pub struct SharedData {
    pub numbers: Vec<i32>,
    pub sum: Option<i32>,
    pub count: Option<usize>,
}

/// Example solver that processes lines of integers with dependent parts
///
/// - Part 1: Calculate sum and count, return the sum as the answer
/// - Part 2: Use Part 1's data to calculate average, or compute independently
#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 2, tags = ["example", "dependent"])]
pub struct ExampleDependent;

impl AocParser for ExampleDependent {
    type SharedData = SharedData;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Cow::Owned(SharedData {
            numbers,
            sum: None,
            count: None,
        }))
    }
}

impl PartSolver<1> for ExampleDependent {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        // Part 1: Calculate sum and count
        // Need to mutate, so call to_mut() to get owned data
        let data = shared.to_mut();
        let sum: i32 = data.numbers.iter().sum();
        let count = data.numbers.len();

        // Store for Part 2
        data.sum = Some(sum);
        data.count = Some(count);

        Ok(sum.to_string())
    }
}

impl PartSolver<2> for ExampleDependent {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        // Part 2: Calculate average using Part 1's data if available
        let average = if let (Some(sum), Some(count)) = (shared.sum, shared.count) {
            // Use the data from Part 1 (read-only access)
            println!("Using Part 1 data: sum={}, count={}", sum, count);
            if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            }
        } else {
            // Compute independently if Part 1 wasn't run (read-only access)
            println!("Computing independently (Part 1 not run)");
            let sum: i32 = shared.numbers.iter().sum();
            let count = shared.numbers.len();
            if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            }
        };

        Ok(format!("{:.2}", average))
    }
}

fn main() {
    println!("=== Dependent Parts Example ===\n");

    // Use the plugin system with automatic registration via derive macro
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    let input = "10\n20\n30\n40\n50";
    println!("Input: {}", input.replace('\n', ", "));
    println!();

    let mut solver = registry
        .create_solver(2023, 2, input)
        .expect("Failed to create solver");

    match solver.solve(1) {
        Ok(answer) => println!("Part 1 (Sum): {}", answer),
        Err(e) => eprintln!("Error solving part 1: {}", e),
    }
    println!();

    match solver.solve(2) {
        Ok(answer) => println!("Part 2 (Average): {}", answer),
        Err(e) => eprintln!("Error solving part 2: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        let input = "10\n20\n30";
        let cow = <ExampleDependent as AocParser>::parse(input).unwrap();
        let shared = cow.into_owned();
        assert_eq!(shared.numbers, vec![10, 20, 30]);
    }

    #[test]
    fn test_part1_stores_data() {
        let mut shared = Cow::Owned(SharedData {
            numbers: vec![10, 20, 30],
            sum: None,
            count: None,
        });
        let result = <ExampleDependent as PartSolver<1>>::solve(&mut shared).unwrap();

        assert_eq!(result, "60");
        assert_eq!(shared.sum, Some(60));
        assert_eq!(shared.count, Some(3));
    }

    #[test]
    fn test_part2_uses_part1_data() {
        let mut shared = Cow::Owned(SharedData {
            numbers: vec![10, 20, 30],
            sum: None,
            count: None,
        });

        // First solve Part 1 to populate shared data
        let _part1_result = <ExampleDependent as PartSolver<1>>::solve(&mut shared).unwrap();

        // Now solve Part 2 which uses Part 1's data
        let part2_result = <ExampleDependent as PartSolver<2>>::solve(&mut shared).unwrap();

        // Average of 10, 20, 30 is 20.00
        assert_eq!(part2_result, "20.00");
    }

    #[test]
    fn test_part2_solves_independently() {
        let mut shared = Cow::Owned(SharedData {
            numbers: vec![10, 20, 30],
            sum: None,
            count: None,
        });

        // Solve Part 2 without Part 1's data
        let result = <ExampleDependent as PartSolver<2>>::solve(&mut shared).unwrap();

        // Should still compute the correct average
        assert_eq!(result, "20.00");
    }

    #[test]
    fn test_empty_input() {
        let mut shared = Cow::Owned(SharedData {
            numbers: vec![],
            sum: None,
            count: None,
        });

        let part1_result = <ExampleDependent as PartSolver<1>>::solve(&mut shared).unwrap();
        assert_eq!(part1_result, "0");

        let part2_result = <ExampleDependent as PartSolver<2>>::solve(&mut shared).unwrap();
        assert_eq!(part2_result, "0.00");
    }
}
