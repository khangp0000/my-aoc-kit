//! Example solver with independent parts
//!
//! This example demonstrates a simple case where Part 1 and Part 2
//! are completely independent and don't share any data.
//!
//! Run with: cargo run --example independent_parts

use aoc_solver::{
    AocSolver, ParseError, PartResult, RegistryBuilder, SolveError, Solver,
};

/// Example solver that processes lines of integers
///
/// - Part 1: Sum all numbers
/// - Part 2: Product of all numbers
#[derive(AocSolver)]
#[aoc(year = 2023, day = 1, tags = ["example", "independent"])]
pub struct ExampleIndependent;

impl Solver for ExampleIndependent {
    type Parsed = Vec<i32>;
    type PartialResult = (); // No data shared between parts
    
    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat(
                        format!("Expected integer, got: {}", line)
                    ))
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
                // Part 1: Sum all numbers
                let sum: i32 = parsed.iter().sum();
                Ok(PartResult {
                    answer: sum.to_string(),
                    partial: None, // No data to share
                })
            }
            2 => {
                // Part 2: Product of all numbers
                let product: i32 = parsed.iter().product();
                Ok(PartResult {
                    answer: product.to_string(),
                    partial: None, // No data to share
                })
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
    
    let mut solver = registry.create_solver(2023, 1, input)
        .expect("Failed to create solver");
    
    match solver.solve(1) {
        Ok(answer) => println!("Part 1 (Sum): {}", answer),
        Err(e) => eprintln!("Error solving part 1: {}", e),
    }
    
    match solver.solve(2) {
        Ok(answer) => println!("Part 2 (Product): {}", answer),
        Err(e) => eprintln!("Error solving part 2: {}", e),
    }
    
    println!("Cached results: {:?}", solver.results());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        let input = "1\n2\n3\n4\n5";
        let parsed = ExampleIndependent::parse(input).unwrap();
        assert_eq!(parsed, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let input = "  1  \n  2  \n  3  ";
        let parsed = ExampleIndependent::parse(input).unwrap();
        assert_eq!(parsed, vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_invalid_input() {
        let input = "1\nabc\n3";
        let result = ExampleIndependent::parse(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_part1_sum() {
        let parsed = vec![1, 2, 3, 4, 5];
        let result = ExampleIndependent::solve_part(&parsed, 1, None).unwrap();
        assert_eq!(result.answer, "15");
        assert!(result.partial.is_none());
    }

    #[test]
    fn test_part2_product() {
        let parsed = vec![1, 2, 3, 4, 5];
        let result = ExampleIndependent::solve_part(&parsed, 2, None).unwrap();
        assert_eq!(result.answer, "120");
        assert!(result.partial.is_none());
    }

    #[test]
    fn test_invalid_part() {
        let parsed = vec![1, 2, 3];
        let result = ExampleIndependent::solve_part(&parsed, 3, None);
        assert!(result.is_err());
        assert!(matches!(result, Err(SolveError::PartNotImplemented(3))));
    }
}
