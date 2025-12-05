use aoc_solver::{ParseError, SolveError, Solver};
use aoc_solver_macros::aoc_solver;

struct TestSolver;

#[aoc_solver(max_parts = 2)]
impl TestSolver {
    type SharedData = Vec<i32>;

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

    fn part1(shared: &mut Vec<i32>) -> String {
        shared.iter().sum::<i32>().to_string()
    }

    fn part2(shared: &mut Vec<i32>) -> String {
        shared.iter().product::<i32>().to_string()
    }
}

#[test]
fn test_independent_parts_compiles() {
    // Test that the macro generates valid code
    let input = "1\n2\n3\n4\n5";
    let cow = <TestSolver as Solver>::parse(input).unwrap();
    assert_eq!(*cow, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_solver_trait_implemented() {
    // Test that Solver trait is implemented
    let input = "1\n2\n3";
    let mut cow = <TestSolver as Solver>::parse(input).unwrap();

    let result1 = TestSolver::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result1, "6");

    let result2 = TestSolver::solve_part(&mut cow, 2).unwrap();
    assert_eq!(result2, "6");
}

#[test]
fn test_part_out_of_range() {
    let input = "1\n2\n3";
    let mut cow = <TestSolver as Solver>::parse(input).unwrap();

    let result = TestSolver::solve_part(&mut cow, 3);
    assert!(result.is_err());
    assert!(matches!(result, Err(SolveError::PartNotImplemented(3))));
}
