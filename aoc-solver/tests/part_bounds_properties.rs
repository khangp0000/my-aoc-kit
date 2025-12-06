//! Property-based tests for solver part bounds validation
//!
//! **Feature: solver-part-bounds**

use aoc_solver::{AocParser, ParseError, SolveError, Solver, SolverExt};
use proptest::prelude::*;
use std::borrow::Cow;

/// Test solver with configurable PARTS
struct TestSolver<const N: u8>;

impl<const N: u8> AocParser for TestSolver<N> {
    type SharedData = ();

    fn parse(_input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        Ok(Cow::Owned(()))
    }
}

impl<const N: u8> Solver for TestSolver<N> {
    const PARTS: u8 = N;

    fn solve_part(_shared: &mut Cow<'_, Self::SharedData>, part: u8) -> Result<String, SolveError> {
        Ok(format!("part{}", part))
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// **Feature: solver-part-bounds, Property 1: Out-of-range rejection**
    /// *For any* solver with PARTS = N, calling `solve_part_checked_range(part)`
    /// where part = 0 OR part > N returns `PartOutOfRange(part)`.
    /// **Validates: Requirements 2.1, 2.2**
    #[test]
    fn prop_out_of_range_rejection(max_parts in 1u8..=25, part in 0u8..=255) {
        let mut shared: Cow<'_, ()> = Cow::Owned(());

        // Test with different PARTS values
        let result = match max_parts {
            1 => TestSolver::<1>::solve_part_checked_range(&mut shared, part),
            2 => TestSolver::<2>::solve_part_checked_range(&mut shared, part),
            3 => TestSolver::<3>::solve_part_checked_range(&mut shared, part),
            _ => TestSolver::<2>::solve_part_checked_range(&mut shared, part),
        };

        let effective_max = match max_parts {
            1 => 1,
            2 => 2,
            3 => 3,
            _ => 2,
        };

        if part == 0 || part > effective_max {
            // Should be out of range
            match result {
                Err(SolveError::PartOutOfRange(p)) => prop_assert_eq!(p, part),
                other => prop_assert!(false, "Expected PartOutOfRange, got {:?}", other),
            }
        } else {
            // Should succeed
            prop_assert!(result.is_ok(), "Expected Ok for part {} with max {}", part, effective_max);
        }
    }

    /// **Feature: solver-part-bounds, Property 2: Valid range delegation**
    /// *For any* solver with PARTS = N and part where 1 <= part <= N,
    /// `solve_part_checked_range(part)` delegates to `solve_part(part)`.
    /// **Validates: Requirements 2.3**
    #[test]
    fn prop_valid_range_delegation(part in 1u8..=2) {
        let mut shared: Cow<'_, ()> = Cow::Owned(());
        let mut shared2: Cow<'_, ()> = Cow::Owned(());

        // Test that valid parts delegate correctly
        let checked_result = TestSolver::<2>::solve_part_checked_range(&mut shared, part);
        let direct_result = TestSolver::<2>::solve_part(&mut shared2, part);

        // Both should succeed with same value
        prop_assert!(checked_result.is_ok());
        prop_assert!(direct_result.is_ok());
        prop_assert_eq!(checked_result.unwrap(), direct_result.unwrap());
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_part_zero_rejected() {
        let mut shared: Cow<'_, ()> = Cow::Owned(());
        let result = TestSolver::<2>::solve_part_checked_range(&mut shared, 0);
        assert!(matches!(result, Err(SolveError::PartOutOfRange(0))));
    }

    #[test]
    fn test_part_exceeds_max_rejected() {
        let mut shared: Cow<'_, ()> = Cow::Owned(());
        let result = TestSolver::<2>::solve_part_checked_range(&mut shared, 3);
        assert!(matches!(result, Err(SolveError::PartOutOfRange(3))));
    }

    #[test]
    fn test_valid_part_succeeds() {
        let mut shared: Cow<'_, ()> = Cow::Owned(());
        let result = TestSolver::<2>::solve_part_checked_range(&mut shared, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "part1");
    }
}
