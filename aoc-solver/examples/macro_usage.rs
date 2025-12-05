//! Example demonstrating the #[aoc_solver] macro
//!
//! This example shows how to use the #[aoc_solver] attribute macro
//! to simplify solver implementation.
//!
//! Run with: cargo run --example macro_usage

use aoc_solver::ParseError;
use aoc_solver_macros::aoc_solver;

/// Example solver using the macro with independent parts
struct SimpleExample;

#[aoc_solver(max_parts = 2)]
impl SimpleExample {
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

/// Shared data that can be mutated by parts to pass information
#[derive(Debug, Clone)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
    count: Option<usize>,
}

/// Example solver with dependent parts
struct DependentExample;

#[aoc_solver(max_parts = 2)]
impl DependentExample {
    type SharedData = SharedData;

    fn parse(input: &str) -> Result<SharedData, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|line| {
                line.trim().parse::<i32>().map_err(|_| {
                    ParseError::InvalidFormat(format!("Expected integer, got: {}", line))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SharedData {
            numbers,
            sum: None,
            count: None,
        })
    }

    fn part1(shared: &mut SharedData) -> String {
        let sum: i32 = shared.numbers.iter().sum();
        let count = shared.numbers.len();

        // Store for Part 2
        shared.sum = Some(sum);
        shared.count = Some(count);

        sum.to_string()
    }

    fn part2(shared: &mut SharedData) -> String {
        if let (Some(sum), Some(count)) = (shared.sum, shared.count) {
            // Use data from Part 1
            println!("Using Part 1 data: sum={}, count={}", sum, count);
            let avg = if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            };
            format!("{:.2}", avg)
        } else {
            // Compute independently if Part 1 wasn't run
            println!("Computing independently (Part 1 not run)");
            let sum: i32 = shared.numbers.iter().sum();
            let count = shared.numbers.len();
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

    let mut shared1 = SimpleExample::parse(input1).expect("Failed to parse");
    println!("Part 1 (Sum): {}", SimpleExample::part1(&mut shared1));
    println!("Part 2 (Product): {}", SimpleExample::part2(&mut shared1));

    println!("\n=== Dependent Example ===\n");

    let input2 = "10\n20\n30\n40\n50";
    println!("Input: {}", input2.replace('\n', ", "));

    let mut shared2 = DependentExample::parse(input2).expect("Failed to parse");

    // Part 1 stores data
    let part1_answer = DependentExample::part1(&mut shared2);
    println!("Part 1 (Sum): {}", part1_answer);

    // Part 2 uses Part 1's data
    let part2_answer = DependentExample::part2(&mut shared2);
    println!("Part 2 (Average): {}", part2_answer);

    println!("\n=== Using Solver Trait ===\n");

    // The macro generates the Solver trait implementation
    use aoc_solver::Solver;

    let input3 = "2\n4\n6";
    let mut cow = <SimpleExample as Solver>::parse(input3).expect("Failed to parse");

    let result1 = SimpleExample::solve_part(&mut cow, 1).expect("Failed to solve part 1");
    println!("Part 1: {}", result1);

    let result2 = SimpleExample::solve_part(&mut cow, 2).expect("Failed to solve part 2");
    println!("Part 2: {}", result2);

    // Trying to solve part 3 returns PartNotImplemented error
    match SimpleExample::solve_part(&mut cow, 3) {
        Ok(_) => println!("Part 3: unexpected success"),
        Err(e) => println!("Part 3: {} (expected)", e),
    }
}
