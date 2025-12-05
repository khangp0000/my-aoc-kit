use aoc_solver::{ParseError, PartResult, Solver, SolveError};
use aoc_solver_macros::aoc_solver;

struct TestResultReturns;

#[aoc_solver(max_parts = 4)]
impl TestResultReturns {
    type Parsed = Vec<i32>;
    type PartialResult = i32;
    
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
    
    // Part 1: Returns String
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    // Part 2: Returns Result<String, SolveError>
    fn part2(parsed: &Vec<i32>) -> Result<String, SolveError> {
        if parsed.is_empty() {
            Err(SolveError::SolveFailed("Empty input".into()))
        } else {
            Ok(parsed.iter().product::<i32>().to_string())
        }
    }
    
    // Part 3: Returns PartResult<i32>
    fn part3(parsed: &Vec<i32>) -> PartResult<i32> {
        let sum: i32 = parsed.iter().sum();
        PartResult {
            answer: sum.to_string(),
            partial: Some(sum),
        }
    }
    
    // Part 4: Returns Result<PartResult<i32>, SolveError>
    fn part4(parsed: &Vec<i32>, prev: Option<&i32>) -> Result<PartResult<i32>, SolveError> {
        if let Some(&prev_sum) = prev {
            let product: i32 = parsed.iter().product();
            Ok(PartResult {
                answer: (prev_sum + product).to_string(),
                partial: Some(product),
            })
        } else {
            Err(SolveError::SolveFailed("No previous data".into()))
        }
    }
}

#[test]
fn test_string_return() {
    let parsed = vec![1, 2, 3];
    let result = TestResultReturns::solve_part(&parsed, 1, None).unwrap();
    assert_eq!(result.answer, "6");
    assert!(result.partial.is_none());
}

#[test]
fn test_result_string_return_ok() {
    let parsed = vec![2, 3, 4];
    let result = TestResultReturns::solve_part(&parsed, 2, None).unwrap();
    assert_eq!(result.answer, "24");
    assert!(result.partial.is_none());
}

#[test]
fn test_result_string_return_err() {
    let parsed = vec![];
    let result = TestResultReturns::solve_part(&parsed, 2, None);
    assert!(result.is_err());
}

#[test]
fn test_part_result_return() {
    let parsed = vec![1, 2, 3];
    let result = TestResultReturns::solve_part(&parsed, 3, None).unwrap();
    assert_eq!(result.answer, "6");
    assert_eq!(result.partial, Some(6));
}

#[test]
fn test_result_part_result_return_ok() {
    let parsed = vec![2, 3];
    
    // First get data from part3
    let result3 = TestResultReturns::solve_part(&parsed, 3, None).unwrap();
    assert_eq!(result3.partial, Some(5));
    
    // Then use it in part4
    let result4 = TestResultReturns::solve_part(&parsed, 4, result3.partial.as_ref()).unwrap();
    assert_eq!(result4.answer, "11"); // 5 + 6
    assert_eq!(result4.partial, Some(6));
}

#[test]
fn test_result_part_result_return_err() {
    let parsed = vec![2, 3];
    let result = TestResultReturns::solve_part(&parsed, 4, None);
    assert!(result.is_err());
}
