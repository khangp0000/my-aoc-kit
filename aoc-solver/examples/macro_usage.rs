//! Example demonstrating the #[aoc_solver] macro
//!
//! This example shows how to use the #[aoc_solver] attribute macro
//! to simplify solver implementation.
//!
//! Run with: cargo run --example macro_usage

use aoc_solver::{ParseError, PartResult};
use aoc_solver_macros::aoc_solver;

/// Example solver using the macro with independent parts
struct SimpleExample;

#[aoc_solver(max_parts = 2)]
impl SimpleExample {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
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
    
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    fn part2(parsed: &Vec<i32>) -> String {
        parsed.iter().product::<i32>().to_string()
    }
}

/// Data shared from Part 1 to Part 2
#[derive(Debug, Clone)]
struct SumCount {
    sum: i32,
    count: usize,
}

/// Example solver with dependent parts
struct DependentExample;

#[aoc_solver(max_parts = 2)]
impl DependentExample {
    type Parsed = Vec<i32>;
    type PartialResult = SumCount;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
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
    
    fn part1(parsed: &Vec<i32>) -> PartResult<SumCount> {
        let sum: i32 = parsed.iter().sum();
        let count = parsed.len();
        
        PartResult {
            answer: sum.to_string(),
            partial: Some(SumCount { sum, count }),
        }
    }
    
    fn part2(parsed: &Vec<i32>, prev: Option<&SumCount>) -> String {
        if let Some(data) = prev {
            // Use data from Part 1
            println!("Using Part 1 data: sum={}, count={}", data.sum, data.count);
            let avg = if data.count > 0 {
                data.sum as f64 / data.count as f64
            } else {
                0.0
            };
            format!("{:.2}", avg)
        } else {
            // Compute independently if Part 1 wasn't run
            println!("Computing independently (Part 1 not run)");
            let sum: i32 = parsed.iter().sum();
            let count = parsed.len();
            let avg = if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            };
            format!("{:.2}", avg)
        }
    }
}

fn main() {
    println!("=== Simple Example (Independent Parts) ===\n");
    
    let input1 = "1\n2\n3\n4\n5";
    println!("Input: {}", input1.replace('\n', ", "));
    
    let parsed1 = SimpleExample::parse(input1).expect("Failed to parse");
    println!("Part 1 (Sum): {}", SimpleExample::part1(&parsed1));
    println!("Part 2 (Product): {}", SimpleExample::part2(&parsed1));
    
    println!("\n=== Dependent Example ===\n");
    
    let input2 = "10\n20\n30\n40\n50";
    println!("Input: {}", input2.replace('\n', ", "));
    
    let parsed2 = DependentExample::parse(input2).expect("Failed to parse");
    
    // Part 1 returns data
    let part1_result = DependentExample::part1(&parsed2);
    println!("Part 1 (Sum): {}", part1_result.answer);
    
    // Part 2 uses Part 1's data
    let part2_answer = DependentExample::part2(&parsed2, part1_result.partial.as_ref());
    println!("Part 2 (Average): {}", part2_answer);
    
    println!("\n=== Using Solver Trait ===\n");
    
    // The macro generates the Solver trait implementation
    use aoc_solver::Solver;
    
    let input3 = "2\n4\n6";
    let parsed3 = SimpleExample::parse(input3).expect("Failed to parse");
    
    let result1 = SimpleExample::solve_part(&parsed3, 1, None).expect("Failed to solve part 1");
    println!("Part 1: {}", result1.answer);
    
    let result2 = SimpleExample::solve_part(&parsed3, 2, None).expect("Failed to solve part 2");
    println!("Part 2: {}", result2.answer);
    
    // Trying to solve part 3 returns PartOutOfRange error
    match SimpleExample::solve_part(&parsed3, 3, None) {
        Ok(_) => println!("Part 3: unexpected success"),
        Err(e) => println!("Part 3: {} (expected)", e),
    }
}
