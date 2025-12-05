use aoc_solver::{ParseError, Solver, SolveError};
use aoc_solver_macros::aoc_solver;

struct TestSolver;

#[aoc_solver(max_parts = 2)]
impl TestSolver {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
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
    
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    fn part2(parsed: &Vec<i32>) -> String {
        parsed.iter().product::<i32>().to_string()
    }
}

#[test]
fn test_independent_parts_compiles() {
    // Test that the macro generates valid code
    let input = "1\n2\n3\n4\n5";
    let parsed = TestSolver::parse(input).unwrap();
    assert_eq!(parsed, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_solver_trait_implemented() {
    // Test that Solver trait is implemented
    let input = "1\n2\n3";
    let parsed = TestSolver::parse(input).unwrap();
    
    let result1 = TestSolver::solve_part(&parsed, 1, None).unwrap();
    assert_eq!(result1.answer, "6");
    assert!(result1.partial.is_none());
    
    let result2 = TestSolver::solve_part(&parsed, 2, None).unwrap();
    assert_eq!(result2.answer, "6");
    assert!(result2.partial.is_none());
}

#[test]
fn test_part_out_of_range() {
    let input = "1\n2\n3";
    let parsed = TestSolver::parse(input).unwrap();
    
    let result = TestSolver::solve_part(&parsed, 3, None);
    assert!(result.is_err());
    assert!(matches!(result, Err(SolveError::PartOutOfRange(3))));
}
