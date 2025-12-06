use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError, Solver};
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
}

#[derive(AocSolver)]
#[aoc_solver(max_parts = 4)]
struct TestResultReturns;

impl AocParser for TestResultReturns {
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

        Ok(Cow::Owned(SharedData { numbers, sum: None }))
    }
}

// Part 1: Simple sum, stores result
impl PartSolver<1> for TestResultReturns {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        let data = shared.to_mut();
        let sum: i32 = data.numbers.iter().sum();
        data.sum = Some(sum);
        Ok(sum.to_string())
    }
}

// Part 2: Product with error handling
impl PartSolver<2> for TestResultReturns {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        if shared.numbers.is_empty() {
            Err(SolveError::SolveFailed("Empty input".into()))
        } else {
            Ok(shared.numbers.iter().product::<i32>().to_string())
        }
    }
}

// Part 3: Sum (stores for part4)
impl PartSolver<3> for TestResultReturns {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        let data = shared.to_mut();
        let sum: i32 = data.numbers.iter().sum();
        data.sum = Some(sum);
        Ok(sum.to_string())
    }
}

// Part 4: Uses sum from part3
impl PartSolver<4> for TestResultReturns {
    fn solve(shared: &mut Cow<'_, SharedData>) -> Result<String, SolveError> {
        if let Some(prev_sum) = shared.sum {
            let product: i32 = shared.numbers.iter().product();
            Ok((prev_sum + product).to_string())
        } else {
            Err(SolveError::SolveFailed("No previous data".into()))
        }
    }
}

#[test]
fn test_string_return() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![1, 2, 3],
        sum: None,
    });
    let result = <TestResultReturns as Solver>::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_result_string_return_ok() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3, 4],
        sum: None,
    });
    let result = <TestResultReturns as Solver>::solve_part(&mut cow, 2).unwrap();
    assert_eq!(result, "24");
}

#[test]
fn test_result_string_return_err() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![],
        sum: None,
    });
    let result = <TestResultReturns as Solver>::solve_part(&mut cow, 2);
    assert!(result.is_err());
}

#[test]
fn test_part3_stores_sum() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![1, 2, 3],
        sum: None,
    });
    let result = <TestResultReturns as Solver>::solve_part(&mut cow, 3).unwrap();
    assert_eq!(result, "6");
    assert_eq!(cow.sum, Some(6));
}

#[test]
fn test_part4_uses_part3_data() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3],
        sum: None,
    });

    // First run part3 to populate sum
    let _result3 = <TestResultReturns as Solver>::solve_part(&mut cow, 3).unwrap();
    assert_eq!(cow.sum, Some(5));

    // Then run part4 which uses the sum
    let result4 = <TestResultReturns as Solver>::solve_part(&mut cow, 4).unwrap();
    assert_eq!(result4, "11"); // 5 + 6
}

#[test]
fn test_part4_without_part3_data() {
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3],
        sum: None,
    });
    let result = <TestResultReturns as Solver>::solve_part(&mut cow, 4);
    assert!(result.is_err());
}
