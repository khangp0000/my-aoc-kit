use aoc_solver::{ParseError, SolveError, Solver};
use aoc_solver_macros::aoc_solver;

#[derive(Debug, Clone)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
}

struct TestResultReturns;

#[aoc_solver(max_parts = 4)]
impl TestResultReturns {
    type SharedData = SharedData;

    fn parse(input: &str) -> Result<SharedData, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SharedData { numbers, sum: None })
    }

    // Part 1: Returns String
    fn part1(shared: &mut SharedData) -> String {
        let sum: i32 = shared.numbers.iter().sum();
        shared.sum = Some(sum);
        sum.to_string()
    }

    // Part 2: Returns Result<String, SolveError>
    fn part2(shared: &mut SharedData) -> Result<String, SolveError> {
        if shared.numbers.is_empty() {
            Err(SolveError::SolveFailed("Empty input".into()))
        } else {
            Ok(shared.numbers.iter().product::<i32>().to_string())
        }
    }

    // Part 3: Returns String (stores sum for part4)
    fn part3(shared: &mut SharedData) -> String {
        let sum: i32 = shared.numbers.iter().sum();
        shared.sum = Some(sum);
        sum.to_string()
    }

    // Part 4: Returns Result<String, SolveError> (uses sum from part3)
    fn part4(shared: &mut SharedData) -> Result<String, SolveError> {
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
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![1, 2, 3],
        sum: None,
    });
    let result = TestResultReturns::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "6");
}

#[test]
fn test_result_string_return_ok() {
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3, 4],
        sum: None,
    });
    let result = TestResultReturns::solve_part(&mut cow, 2).unwrap();
    assert_eq!(result, "24");
}

#[test]
fn test_result_string_return_err() {
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![],
        sum: None,
    });
    let result = TestResultReturns::solve_part(&mut cow, 2);
    assert!(result.is_err());
}

#[test]
fn test_part3_stores_sum() {
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![1, 2, 3],
        sum: None,
    });
    let result = TestResultReturns::solve_part(&mut cow, 3).unwrap();
    assert_eq!(result, "6");
    assert_eq!(cow.sum, Some(6));
}

#[test]
fn test_part4_uses_part3_data() {
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3],
        sum: None,
    });

    // First run part3 to populate sum
    let _result3 = TestResultReturns::solve_part(&mut cow, 3).unwrap();
    assert_eq!(cow.sum, Some(5));

    // Then run part4 which uses the sum
    let result4 = TestResultReturns::solve_part(&mut cow, 4).unwrap();
    assert_eq!(result4, "11"); // 5 + 6
}

#[test]
fn test_part4_without_part3_data() {
    use std::borrow::Cow;
    let mut cow = Cow::Owned(SharedData {
        numbers: vec![2, 3],
        sum: None,
    });
    let result = TestResultReturns::solve_part(&mut cow, 4);
    assert!(result.is_err());
}
