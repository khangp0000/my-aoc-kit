//! Property-based tests for the AocSolver derive macro
//!
//! These tests verify the correctness properties defined in the design document
//! for the trait-based solver redesign.

use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError, Solver};
use proptest::prelude::*;

// Test solver for property tests
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct TestSolver;

impl AocParser for TestSolver {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                l.parse()
                    .map_err(|_| ParseError::InvalidFormat("bad int".into()))
            })
            .collect()
    }
}

impl PartSolver<1> for TestSolver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for TestSolver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

// Property 1 (Parse delegation equivalence) is no longer needed since
// Solver: AocParser means there's no delegation - parse() is inherited directly.

/// **Feature: trait-based-solver-redesign, Property 1: Part dispatch correctness**
///
/// *For any* valid part number N in 1..=max_parts, calling `Solver::solve_part(shared, N)`
/// should produce the same result as calling `<Self as PartSolver<N>>::solve(shared)`.
///
/// **Validates: Requirements 3.3**
mod property_1_part_dispatch {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn solve_part_dispatches_to_correct_part_solver(
            numbers in prop::collection::vec(1i32..10, 1..5),
            part in 1u8..=2
        ) {
            let input = numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("\n");
            let mut shared1 = <TestSolver as AocParser>::parse(&input).unwrap();
            let mut shared2 = <TestSolver as AocParser>::parse(&input).unwrap();

            let solver_result = <TestSolver as Solver>::solve_part(&mut shared1, part);

            let direct_result = match part {
                1 => <TestSolver as PartSolver<1>>::solve(&mut shared2),
                2 => <TestSolver as PartSolver<2>>::solve(&mut shared2),
                _ => unreachable!(),
            };

            // Compare Ok values (both should succeed for valid parts)
            prop_assert_eq!(solver_result.unwrap(), direct_result.unwrap());
        }
    }
}

/// **Feature: trait-based-solver-redesign, Property 2: Invalid part rejection**
///
/// *For any* part number outside the valid range (0 or > max_parts),
/// `Solver::solve_part` should return `SolveError::PartNotImplemented`.
///
/// **Validates: Requirements 3.5**
mod property_2_invalid_part_rejection {
    use super::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn invalid_part_returns_not_implemented(invalid_part in prop_oneof![Just(0u8), 3u8..=255]) {
            let input = "1\n2\n3";
            let mut shared = <TestSolver as AocParser>::parse(input).unwrap();

            let result = <TestSolver as Solver>::solve_part(&mut shared, invalid_part);

            match result {
                Err(SolveError::PartNotImplemented(p)) => prop_assert_eq!(p, invalid_part),
                _ => prop_assert!(false, "Expected PartNotImplemented error for part {}", invalid_part),
            }
        }
    }
}

/// **Feature: trait-based-solver-redesign, Property 3: Read-only operations work correctly**
///
/// *For any* solver where part functions only read from shared data,
/// the solve operations should work correctly without mutation.
///
/// **Validates: Requirements 2.4, 1.2, 1.3**
mod property_3_read_only {
    use super::*;

    // Read-only solver
    #[derive(AocSolver)]
    #[aoc_solver(max_parts = 1)]
    struct ReadOnlySolver;

    impl AocParser for ReadOnlySolver {
        type SharedData<'a> = Vec<i32>;

        fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
            input
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| {
                    l.parse()
                        .map_err(|_| ParseError::InvalidFormat("bad int".into()))
                })
                .collect()
        }
    }

    impl PartSolver<1> for ReadOnlySolver {
        fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
            // Read-only access
            Ok(shared.iter().sum::<i32>().to_string())
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn read_only_solve_works_correctly(numbers in prop::collection::vec(1i32..100, 1..5)) {
            let input = numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("\n");
            let expected_sum: i32 = numbers.iter().sum();

            let mut shared = <ReadOnlySolver as AocParser>::parse(&input).unwrap();

            let result = <ReadOnlySolver as Solver>::solve_part(&mut shared, 1).unwrap();

            prop_assert_eq!(result, expected_sum.to_string());
        }
    }
}

/// **Feature: trait-based-solver-redesign, Property 4: Mutation works correctly**
///
/// *For any* solver where a part function mutates shared data,
/// the mutations should be visible to subsequent parts.
///
/// **Validates: Requirements 2.5**
mod property_4_mutation {
    use super::*;

    #[derive(Debug, Clone)]
    struct MutableData {
        numbers: Vec<i32>,
        cached_sum: Option<i32>,
    }

    #[derive(AocSolver)]
    #[aoc_solver(max_parts = 2)]
    struct MutatingSolver;

    impl AocParser for MutatingSolver {
        type SharedData<'a> = MutableData;

        fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
            let numbers: Vec<i32> = input
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| {
                    l.parse()
                        .map_err(|_| ParseError::InvalidFormat("bad int".into()))
                })
                .collect::<Result<_, _>>()?;
            Ok(MutableData {
                numbers,
                cached_sum: None,
            })
        }
    }

    impl PartSolver<1> for MutatingSolver {
        fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
            // Mutating access
            let sum: i32 = shared.numbers.iter().sum();
            shared.cached_sum = Some(sum);
            Ok(sum.to_string())
        }
    }

    impl PartSolver<2> for MutatingSolver {
        fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
            // Uses cached value from part 1
            let sum = shared.cached_sum.unwrap_or(0);
            Ok((sum * 2).to_string())
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn mutation_works_correctly(numbers in prop::collection::vec(1i32..100, 1..5)) {
            let input = numbers.iter().map(|n| n.to_string()).collect::<Vec<_>>().join("\n");
            let expected_sum: i32 = numbers.iter().sum();

            let mut shared = <MutatingSolver as AocParser>::parse(&input).unwrap();

            // Part 1 should compute and cache the sum
            let result1 = <MutatingSolver as Solver>::solve_part(&mut shared, 1).unwrap();
            prop_assert_eq!(result1, expected_sum.to_string());

            // Verify the cache was set
            prop_assert_eq!(shared.cached_sum, Some(expected_sum));

            // Part 2 should use the cached value
            let result2 = <MutatingSolver as Solver>::solve_part(&mut shared, 2).unwrap();
            prop_assert_eq!(result2, (expected_sum * 2).to_string());
        }
    }
}
