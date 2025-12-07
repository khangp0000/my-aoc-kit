//! Example demonstrating the #[derive(AocSolver)] macro
//!
//! This example shows how to use the AocSolver derive macro with
//! AocParser and PartSolver traits to simplify solver implementation.
//!
//! Run with: cargo run --example macro_usage

use aoc_solver::{
    AocParser, AocSolver, AutoRegisterSolver, ParseError, PartSolver, SolveError, Solver,
    SolverRegistryBuilder,
};

/// Example solver using the macro with independent parts
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct SimpleExample;

impl AocParser for SimpleExample {
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

impl PartSolver<1> for SimpleExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for SimpleExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
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
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct DependentExample;

impl AocParser for DependentExample {
    type SharedData<'a> = SharedData;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
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
}

impl PartSolver<1> for DependentExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        let sum: i32 = shared.numbers.iter().sum();
        let count = shared.numbers.len();

        // Store for Part 2
        shared.sum = Some(sum);
        shared.count = Some(count);

        Ok(sum.to_string())
    }
}

impl PartSolver<2> for DependentExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        if let (Some(sum), Some(count)) = (shared.sum, shared.count) {
            // Use data from Part 1
            println!("Using Part 1 data: sum={}, count={}", sum, count);
            let avg = if count > 0 {
                sum as f64 / count as f64
            } else {
                0.0
            };
            Ok(format!("{:.2}", avg))
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
            Ok(format!("{:.2}", avg))
        }
    }
}

// ============================================================================
// Zero-copy example: SharedData = str (no parsing, just borrow the input)
// Uses AutoRegisterSolver for automatic plugin registration
// ============================================================================

/// Example solver that uses `&str` as SharedData for true zero-copy parsing.
/// This is useful when the input doesn't need transformation.
#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2024, day = 25, tags = ["zero-copy", "str"])]
struct ZeroCopyStrExample;

impl AocParser for ZeroCopyStrExample {
    // Using `&'a str` as SharedData - no allocation, just borrow the input!
    type SharedData<'a> = &'a str;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        // Zero-copy: just return a borrowed reference to the input
        Ok(input)
    }
}

impl PartSolver<1> for ZeroCopyStrExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        // Count lines in the input
        let line_count = shared.lines().count();
        Ok(line_count.to_string())
    }
}

impl PartSolver<2> for ZeroCopyStrExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        // Count total characters (excluding newlines)
        let char_count: usize = shared.lines().map(|l| l.len()).sum();
        Ok(char_count.to_string())
    }
}

fn main() {
    println!("=== Simple Example (Independent Parts) ===\n");

    let input1 = "1\n2\n3\n4\n5";
    println!("Input: {}", input1.replace('\n', ", "));

    let mut shared1 = <SimpleExample as AocParser>::parse(input1).expect("Failed to parse");
    println!(
        "Part 1 (Sum): {}",
        <SimpleExample as Solver>::solve_part(&mut shared1, 1).unwrap()
    );
    println!(
        "Part 2 (Product): {}",
        <SimpleExample as Solver>::solve_part(&mut shared1, 2).unwrap()
    );

    println!("\n=== Dependent Example ===\n");

    let input2 = "10\n20\n30\n40\n50";
    println!("Input: {}", input2.replace('\n', ", "));

    let mut shared2 = <DependentExample as AocParser>::parse(input2).expect("Failed to parse");

    // Part 1 stores data
    let part1_answer = <DependentExample as Solver>::solve_part(&mut shared2, 1).unwrap();
    println!("Part 1 (Sum): {}", part1_answer);

    // Part 2 uses Part 1's data
    let part2_answer = <DependentExample as Solver>::solve_part(&mut shared2, 2).unwrap();
    println!("Part 2 (Average): {}", part2_answer);

    println!("\n=== Using PartSolver Traits Directly ===\n");

    let input3 = "2\n4\n6";
    let mut shared3 = <SimpleExample as AocParser>::parse(input3).expect("Failed to parse");

    let result1 =
        <SimpleExample as PartSolver<1>>::solve(&mut shared3).expect("Failed to solve part 1");
    println!("Part 1: {}", result1);

    let result2 =
        <SimpleExample as PartSolver<2>>::solve(&mut shared3).expect("Failed to solve part 2");
    println!("Part 2: {}", result2);

    // Trying to solve part 3 via Solver trait returns PartNotImplemented error
    match <SimpleExample as Solver>::solve_part(&mut shared3, 3) {
        Ok(_) => println!("Part 3: unexpected success"),
        Err(e) => println!("Part 3: {} (expected)", e),
    }

    println!("\n=== Zero-Copy str Example (with AutoRegisterSolver) ===\n");

    let input4 = "hello\nworld\nfoo\nbar";
    println!("Input: {:?}", input4);

    // Direct usage via traits
    let mut shared4 = <ZeroCopyStrExample as AocParser>::parse(input4).expect("Failed to parse");

    // With &str as SharedData, it's always zero-copy (just a reference)
    println!("Zero-copy: true (using &str directly)");

    let lines = <ZeroCopyStrExample as Solver>::solve_part(&mut shared4, 1).unwrap();
    println!("Part 1 (Line count): {}", lines);

    let chars = <ZeroCopyStrExample as Solver>::solve_part(&mut shared4, 2).unwrap();
    println!("Part 2 (Char count): {}", chars);

    // Also demonstrate using via the registry (auto-registered)
    println!("\n--- Using via Registry (auto-registered) ---");
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    if let Ok(mut solver) = registry.create_solver(2024, 25, input4) {
        println!("Found solver for 2024 day 25 (ZeroCopyStrExample)");
        println!("Part 1: {}", solver.solve(1).unwrap());
        println!("Part 2: {}", solver.solve(2).unwrap());
    }
}
