use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError, Solver};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
    count: Option<usize>,
}

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct TestDependentSolver;

impl AocParser for TestDependentSolver {
    type SharedData = SharedData;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Cow::Owned(SharedData {
            numbers,
            sum: None,
            count: None,
        }))
    }
}

impl PartSolver<1> for TestDependentSolver {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        let data = shared.to_mut();
        let sum: i32 = data.numbers.iter().sum();
        let count = data.numbers.len();

        // Store for part2
        data.sum = Some(sum);
        data.count = Some(count);

        Ok(sum.to_string())
    }
}

impl PartSolver<2> for TestDependentSolver {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        // Use data from part1 if available, otherwise compute
        let sum = shared.sum.unwrap_or_else(|| shared.numbers.iter().sum());
        let count = shared.count.unwrap_or_else(|| shared.numbers.len());

        let avg = if count > 0 {
            sum as f64 / count as f64
        } else {
            0.0
        };
        Ok(format!("{:.2}", avg))
    }
}

#[test]
fn test_dependent_parts_compiles() {
    // Test that the macro generates valid code
    let input = "10\n20\n30";
    let cow = <TestDependentSolver as AocParser>::parse(input).unwrap();
    let shared = cow.into_owned();
    assert_eq!(shared.numbers, vec![10, 20, 30]);
}

#[test]
fn test_part1_stores_data() {
    let input = "10\n20\n30";
    let mut cow = <TestDependentSolver as AocParser>::parse(input).unwrap();

    let result = <TestDependentSolver as Solver>::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "60");

    // Check that data was stored
    assert_eq!(cow.sum, Some(60));
    assert_eq!(cow.count, Some(3));
}

#[test]
fn test_part2_uses_part1_data() {
    let input = "10\n20\n30";
    let mut cow = <TestDependentSolver as AocParser>::parse(input).unwrap();

    // First solve Part 1 to populate shared data
    let _part1_result = <TestDependentSolver as Solver>::solve_part(&mut cow, 1).unwrap();

    // Now solve Part 2 which uses Part 1's data
    let part2_result = <TestDependentSolver as Solver>::solve_part(&mut cow, 2).unwrap();

    // Average of 10, 20, 30 is 20.00
    assert_eq!(part2_result, "20.00");
}

#[test]
fn test_part2_solves_independently() {
    let input = "10\n20\n30";
    let mut cow = <TestDependentSolver as AocParser>::parse(input).unwrap();

    // Solve Part 2 without Part 1 (shared.sum and shared.count are None)
    let result = <TestDependentSolver as Solver>::solve_part(&mut cow, 2).unwrap();

    // Should still compute the correct average
    assert_eq!(result, "20.00");
}
