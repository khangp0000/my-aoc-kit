# Implementation Plan

- [x] 1. Add new traits to aoc-solver library
  - [x] 1.1 Add `AocParser` trait to `aoc-solver/src/solver.rs`
    - Define `SharedData` associated type with `ToOwned` bounds
    - Define `parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError>` method
    - _Requirements: 1.1, 1.2, 1.3_
  - [x] 1.2 Add `PartSolver<const N: u8>` trait to `aoc-solver/src/solver.rs`
    - Require `AocParser` as supertrait
    - Define `solve(shared: &mut Cow<'_, Self::SharedData>) -> Result<String, SolveError>` method
    - _Requirements: 2.1, 2.2_
  - [x] 1.3 Update `Solver` trait to extend `AocParser`
    - Add `AocParser` as supertrait: `trait Solver: AocParser`
    - Remove duplicate `SharedData` type and `parse()` method from `Solver`
    - Keep only `PARTS` constant and `solve_part()` method
    - _Requirements: 4.1, 4.2_
  - [x] 1.4 Export new traits from `aoc-solver/src/lib.rs`
    - Add `AocParser` and `PartSolver` to public exports
    - _Requirements: 1.1, 2.1_

- [x] 2. Implement `AocSolver` derive macro
  - [x] 2.1 Add `AocSolver` derive macro to `aoc-solver-macros/src/lib.rs`
    - Parse `#[aoc_solver(max_parts = N)]` helper attribute
    - Generate `Solver` trait implementation (no `SharedData` or `parse()` - inherited from `AocParser` supertrait)
    - Generate `solve_part` match arms dispatching to `PartSolver<1>`, `PartSolver<2>`, etc.
    - Set `const PARTS` to the `max_parts` value
    - Return `SolveError::PartNotImplemented` for invalid part numbers
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_
  - [x] 2.2 Export `AocSolver` derive macro from `aoc-solver/src/lib.rs`
    - Re-export from aoc-solver-macros
    - _Requirements: 3.1_
  - [x] 2.3 Write property test for part dispatch
    - **Property 1: Part dispatch correctness**
    - **Validates: Requirements 3.3**
  - [x] 2.4 Write property test for invalid part rejection
    - **Property 2: Invalid part rejection**
    - **Validates: Requirements 3.5**

- [x] 3. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Add zero-copy and clone-on-write tests
  - [x] 4.1 Write property test for zero-copy read preservation
    - **Property 3: Zero-copy read preservation**
    - **Validates: Requirements 2.4, 1.2, 1.3**
  - [x] 4.2 Write property test for clone-on-write mutation
    - **Property 4: Clone-on-write mutation**
    - **Validates: Requirements 2.5**

- [x] 5. Update examples to use new trait-based approach
  - [x] 5.1 Update `aoc-solver/examples/independent_parts.rs`
    - Convert to use `AocParser`, `PartSolver<1>`, `PartSolver<2>`, and `#[derive(AocSolver)]`
    - _Requirements: 5.2_
  - [x] 5.2 Update `aoc-solver/examples/dependent_parts.rs`
    - Convert to use new trait-based approach with caching pattern
    - _Requirements: 5.2_
  - [x] 5.3 Update `aoc-solver/examples/macro_usage.rs`
    - Convert from `#[aoc_solver]` attribute macro to `#[derive(AocSolver)]`
    - _Requirements: 5.2_

- [x] 6. Update tests to use new trait-based approach
  - [x] 6.1 Update `aoc-solver-macros/tests/independent_parts.rs`
    - Convert to use `AocParser`, `PartSolver<N>`, and `#[derive(AocSolver)]`
    - _Requirements: 5.3_
  - [x] 6.2 Update `aoc-solver-macros/tests/dependent_parts.rs`
    - Convert to use new trait-based approach
    - _Requirements: 5.3_
  - [x] 6.3 Update `aoc-solver-macros/tests/result_return_types.rs`
    - Convert to use new trait-based approach
    - _Requirements: 5.3_
  - [x] 6.4 Update `aoc-solver-macros/tests/auto_register_compat.rs`
    - Verify `AutoRegisterSolver` works with new `#[derive(AocSolver)]`
    - _Requirements: 4.2, 5.3_

- [x] 7. Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 8. Remove old `#[aoc_solver]` attribute macro
  - [x] 8.1 Remove `aoc_solver` attribute macro from `aoc-solver-macros/src/lib.rs`
    - Delete the `#[proc_macro_attribute] pub fn aoc_solver` function and all helper functions
    - _Requirements: 5.1_
  - [x] 8.2 Remove `aoc_solver` re-export from `aoc-solver/src/lib.rs` if present
    - _Requirements: 5.1_
  - [x] 8.3 Update `aoc-solver/README.md` documentation
    - Document new `AocParser`, `PartSolver<N>`, and `#[derive(AocSolver)]` approach
    - _Requirements: 5.1_

- [x] 9. Final Checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
