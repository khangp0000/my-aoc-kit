# Implementation Plan

- [x] 1. Normalize numeric types across codebase
  - [x] 1.1 Change `year` from `u32` to `u16` in all traits and structs
  - [x] 1.2 Change `day` from `u32` to `u8` in all traits and structs
  - [x] 1.3 Change `PARTS` and `parts()` to use `u8`
  - [x] 1.4 Update all usages in registry, instance, and examples
  - _Requirements: 1.1, 1.2_

- [x] 2. Add PARTS const to Solver trait
  - [x] 2.1 Add `const PARTS: u8` to Solver trait definition
  - [x] 2.2 Update existing Solver implementations to include PARTS
  - _Requirements: 1.1_

- [x] 3. Implement SolverExt trait
  - [x] 3.1 Create SolverExt trait with `solve_part_checked_range` method
  - [x] 3.2 Add blanket impl `impl<T: Solver + ?Sized> SolverExt for T {}`
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2_

- [x] 3.3 Write property test for range validation
  - **Property 1: Out-of-range rejection**
  - **Validates: Requirements 2.1, 2.2**

- [x] 4. Update DynSolver trait
  - [x] 4.1 Add `fn parts(&self) -> u8` to DynSolver trait
  - [x] 4.2 Update SolverInstanceCow impl to use `solve_part_checked_range`
  - [x] 4.3 Implement `parts()` returning `S::PARTS`
  - _Requirements: 1.2, 2.1, 2.2, 2.3_

- [x] 4.4 Write property test for delegation
  - **Property 2: Valid range delegation**
  - **Validates: Requirements 2.3**

- [x] 5. Update examples and tests
  - [x] 5.1 Update all example solvers to include `const PARTS`
  - [x] 5.2 Fix any broken tests due to type changes
  - _Requirements: 1.1, 1.2_

- [x] 6. Final Checkpoint
  - Ensure all tests pass, ask the user if questions arise.
