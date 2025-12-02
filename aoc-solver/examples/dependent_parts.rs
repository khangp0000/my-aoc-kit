//! Example solver with dependent parts
//!
//! This example demonstrates a case where Part 2 can use data from Part 1.
//! Part 1 calculates the sum and count, and Part 2 uses that data to calculate
//! the average (or can compute independently if Part 1 wasn't run).
//!
//! Run with: cargo run --example dependent_parts

use aoc_solver::{
    AutoRegisterSolver, ParseError, PartResult, RegistryBuilder, SolveError, Solver,
};

/// Example solver that processes lines of integers with dependent parts
///
/// - Part 1: Calculate sum and count, return the sum as the answer
/// - Part 2: Use Part 1's data to calculate average, or compute independently
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 2, tags = ["example", "dependent"])]
pub struct ExampleDependent;

/// Data shared from Part 1 to Part 2
#[derive(Debug, Clone)]
pub struct Part1Data {
    pub sum: i32,
    pub count: usize,
}

impl Solver for ExampleDependent {
    type Parsed = Vec<i32>;
    type PartialResult = Part1Data;
    
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
        previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => {
                // Part 1: Calculate sum and count
                let sum: i32 = parsed.iter().sum();
                let count = parsed.len();
                
                Ok(PartResult {
                    answer: sum.to_string(),
                    partial: Some(Part1Data { sum, count }),
                })
            }
            2 => {
                // Part 2: Calculate average using Part 1's data if available
                let average = if let Some(part1_data) = previous_partial {
                    // Use the data from Part 1
                    println!(
                        "Using Part 1 data: sum={}, count={}",
                        part1_data.sum, part1_data.count
                    );
                    if part1_data.count > 0 {
                        part1_data.sum as f64 / part1_data.count as f64
                    } else {
                        0.0
                    }
                } else {
                    // Compute independently if Part 1 wasn't run
                    println!("Computing independently (Part 1 not run)");
                    let sum: i32 = parsed.iter().sum();
                    let count = parsed.len();
                    if count > 0 {
                        sum as f64 / count as f64
                    } else {
                        0.0
                    }
                };
                
                Ok(PartResult {
                    answer: format!("{:.2}", average),
                    partial: None, // No more parts after this
                })
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

fn main() {
    println!("=== Dependent Parts Example ===\n");
    
    // Use the plugin system with automatic registration via derive macro
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();
    
    let input = "10\n20\n30\n40\n50";
    println!("Input: {}", input.replace('\n', ", "));
    println!();
    
    let mut solver = registry.create_solver(2023, 2, input)
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
    
    println!("\nCached results: {:?}", solver.results());
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valid_input() {
        let input = "10\n20\n30";
        let parsed = ExampleDependent::parse(input).unwrap();
        assert_eq!(parsed, vec![10, 20, 30]);
    }

    #[test]
    fn test_part1_produces_partial_result() {
        let parsed = vec![10, 20, 30];
        let result = ExampleDependent::solve_part(&parsed, 1, None).unwrap();
        
        assert_eq!(result.answer, "60");
        assert!(result.partial.is_some());
        
        let partial = result.partial.unwrap();
        assert_eq!(partial.sum, 60);
        assert_eq!(partial.count, 3);
    }

    #[test]
    fn test_part2_uses_part1_data() {
        let parsed = vec![10, 20, 30];
        
        // First solve Part 1 to get the partial data
        let part1_result = ExampleDependent::solve_part(&parsed, 1, None).unwrap();
        let part1_data = part1_result.partial.as_ref();
        
        // Now solve Part 2 with Part 1's data
        let part2_result = ExampleDependent::solve_part(&parsed, 2, part1_data).unwrap();
        
        // Average of 10, 20, 30 is 20.00
        assert_eq!(part2_result.answer, "20.00");
        assert!(part2_result.partial.is_none());
    }

    #[test]
    fn test_part2_solves_independently() {
        let parsed = vec![10, 20, 30];
        
        // Solve Part 2 without Part 1's data
        let result = ExampleDependent::solve_part(&parsed, 2, None).unwrap();
        
        // Should still compute the correct average
        assert_eq!(result.answer, "20.00");
    }

    #[test]
    fn test_empty_input() {
        let parsed = vec![];
        
        let part1_result = ExampleDependent::solve_part(&parsed, 1, None).unwrap();
        assert_eq!(part1_result.answer, "0");
        
        let part2_result = ExampleDependent::solve_part(&parsed, 2, None).unwrap();
        assert_eq!(part2_result.answer, "0.00");
    }
}
