use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError, Solver};
use std::borrow::Cow;

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct TestSolver;

impl AocParser for TestSolver {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
}

impl PartSolver<1> for TestSolver {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for TestSolver {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

#[test]
fn test_independent_parts_compiles() {
    // Test that the macro generates valid code
    let input = "1\n2\n3\n4\n5";
    let cow = <TestSolver as AocParser>::parse(input).unwrap();
    assert_eq!(*cow, vec![1, 2, 3, 4, 5]);
}

#[test]
fn test_solver_trait_implemented() {
    // Test that Solver trait is implemented
    let input = "1\n2\n3";
    let mut cow = <TestSolver as AocParser>::parse(input).unwrap();

    let result1 = <TestSolver as Solver>::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result1, "6");

    let result2 = <TestSolver as Solver>::solve_part(&mut cow, 2).unwrap();
    assert_eq!(result2, "6");
}

#[test]
fn test_part_out_of_range() {
    let input = "1\n2\n3";
    let mut cow = <TestSolver as AocParser>::parse(input).unwrap();

    let result = <TestSolver as Solver>::solve_part(&mut cow, 3);
    assert!(result.is_err());
    assert!(matches!(result, Err(SolveError::PartNotImplemented(3))));
}
