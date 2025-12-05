use aoc_solver::{ParseError, PartResult, Solver};
use aoc_solver_macros::aoc_solver;

#[derive(Debug, Clone)]
struct SumCount {
    sum: i32,
    count: usize,
}

struct TestDependentSolver;

#[aoc_solver(max_parts = 2)]
impl TestDependentSolver {
    type Parsed = Vec<i32>;
    type PartialResult = SumCount;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
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
            // Use data from part1
            let avg = if data.count > 0 {
                data.sum as f64 / data.count as f64
            } else {
                0.0
            };
            format!("{:.2}", avg)
        } else {
            // Compute independently
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

#[test]
fn test_dependent_parts_compiles() {
    // Test that the macro generates valid code
    let input = "10\n20\n30";
    let parsed = TestDependentSolver::parse(input).unwrap();
    assert_eq!(parsed, vec![10, 20, 30]);
}

#[test]
fn test_part1_produces_partial_result() {
    let input = "10\n20\n30";
    let parsed = TestDependentSolver::parse(input).unwrap();
    
    let result = TestDependentSolver::solve_part(&parsed, 1, None).unwrap();
    assert_eq!(result.answer, "60");
    assert!(result.partial.is_some());
    
    let partial = result.partial.unwrap();
    assert_eq!(partial.sum, 60);
    assert_eq!(partial.count, 3);
}

#[test]
fn test_part2_uses_part1_data() {
    let input = "10\n20\n30";
    let parsed = TestDependentSolver::parse(input).unwrap();
    
    // First solve Part 1 to get the partial data
    let part1_result = TestDependentSolver::solve_part(&parsed, 1, None).unwrap();
    let part1_data = part1_result.partial.as_ref();
    
    // Now solve Part 2 with Part 1's data
    let part2_result = TestDependentSolver::solve_part(&parsed, 2, part1_data).unwrap();
    
    // Average of 10, 20, 30 is 20.00
    assert_eq!(part2_result.answer, "20.00");
}

#[test]
fn test_part2_solves_independently() {
    let input = "10\n20\n30";
    let parsed = TestDependentSolver::parse(input).unwrap();
    
    // Solve Part 2 without Part 1's data
    let result = TestDependentSolver::solve_part(&parsed, 2, None).unwrap();
    
    // Should still compute the correct average
    assert_eq!(result.answer, "20.00");
}
