//! Example solver with independent parts
//!
//! This example demonstrates a simple case where Part 1 and Part 2
//! are completely independent and don't share any data.
//!
//! Run with: cargo run --example independent_parts

use aoc_solver::{AutoRegisterSolver, ParseError, RegistryBuilder, SolveError, Solver};
use std::borrow::Cow;

/// Example solver that processes lines of integers
///
/// - Part 1: Sum all numbers
/// - Part 2: Product of all numbers
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["example", "independent"])]
pub struct ExampleIndependent;

impl Solver for ExampleIndependent {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|v| Cow::Owned(v))
    }

    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError> {
        match part {
            1 => {
                // Part 1: Sum all numbers (read-only, no need to call to_mut())
                let sum: i32 = shared.iter().sum();
                Ok(sum.to_string())
            }
            2 => {
                // Part 2: Product of all numbers (read-only, no need to call to_mut())
                let product: i32 = shared.iter().product();
                Ok(product.to_string())
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

fn main() {
    println!("=== Independent Parts Example ===\n");

    // Use the plugin system with automatic registration via derive macro
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    let input = "1\n2\n3\n4\n5";
    println!("Input: {}", input.replace('\n', ", "));

    let mut solver = registry
        .create_solver(2023, 1, input)
        .expect("Failed to create solver");

    match solver.solve(1) {
        Ok(answer) => println!("Part 1 (Sum): {}", answer),
        Err(e) => eprintln!("Error solving part 1: {}", e),
    }

    match solver.solve(2) {
        Ok(answer) => println!("Part 2 (Product): {}", answer),
        Err(e) => eprintln!("Error solving part 2: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        let input = "1\n2\n3\n4\n5";
        let shared = ExampleIndependent::parse(input).unwrap();
        assert_eq!(*shared, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let input = "  1  \n  2  \n  3  ";
        let shared = ExampleIndependent::parse(input).unwrap();
        assert_eq!(*shared, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "1\nabc\n3";
        let result = ExampleIndependent::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_part1_sum() {
        let mut shared = Cow::Owned(vec![1, 2, 3, 4, 5]);
        let result = ExampleIndependent::solve_part(&mut shared, 1).unwrap();
        assert_eq!(result, "15");
    }

    #[test]
    fn test_part2_product() {
        let mut shared = Cow::Owned(vec![1, 2, 3, 4, 5]);
        let result = ExampleIndependent::solve_part(&mut shared, 2).unwrap();
        assert_eq!(result, "120");
    }

    #[test]
    fn test_invalid_part() {
        let mut shared = Cow::Owned(vec![1, 2, 3]);
        let result = ExampleIndependent::solve_part(&mut shared, 3);
        assert!(result.is_err());
        assert!(matches!(result, Err(SolveError::PartNotImplemented(3))));
    }
}
