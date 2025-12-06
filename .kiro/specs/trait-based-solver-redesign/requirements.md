# Requirements Document

## Introduction

This document specifies the requirements for replacing the `#[aoc_solver]` attribute macro with a stronger-typed trait-based approach using `AocParser` and `PartSolver<const N>` traits. The redesign provides compile-time part number validation through const generics and cleaner separation of concerns between parsing and solving.

## Glossary

- **AocParser**: A trait that defines the shared data type and parsing logic for a solver
- **PartSolver<N>**: A const-generic trait that defines the solving logic for a specific part number N
- **AocSolver**: A derive macro that generates the `Solver` trait implementation from `AocParser` + `PartSolver<N>` implementations, configured with `#[aoc_solver(max_parts = N)]` helper attribute
- **SharedData**: The parsed input data structure shared between parsing and solving phases
- **Const Generic**: A Rust feature allowing compile-time constant values as type parameters

## Requirements

### Requirement 1

**User Story:** As a solver implementer, I want to define parsing logic separately from solving logic, so that I have cleaner separation of concerns.

#### Acceptance Criteria

1. WHEN a user implements `AocParser` for a solver type THEN the system SHALL accept a `SharedData` associated type and a `parse` function
2. WHEN the `parse` function returns `Cow::Owned` THEN the system SHALL support transformed/parsed data
3. WHEN the `parse` function returns `Cow::Borrowed` THEN the system SHALL support zero-copy parsing scenarios

### Requirement 2

**User Story:** As a solver implementer, I want to implement each part as a separate trait implementation, so that I get compile-time validation that all required parts are implemented.

#### Acceptance Criteria

1. WHEN a user implements `PartSolver<N>` for a solver type THEN the system SHALL require `AocParser` as a supertrait for any value of N
2. WHEN a user implements `PartSolver<N>` THEN the system SHALL provide access to `SharedData` via `&mut Cow<'_, Self::SharedData>`
3. WHEN a user specifies `#[aoc_solver(max_parts = N)]` but does not implement any `PartSolver<M>` for M in 1..=N THEN the compiler SHALL produce a clear error message indicating the missing implementation
4. WHEN a part function only reads shared data THEN the system SHALL allow zero-copy access without cloning
5. WHEN a part function needs to mutate shared data THEN the system SHALL allow mutation via `to_mut()` which triggers clone-on-write

### Requirement 3

**User Story:** As a solver implementer, I want a derive macro that generates the `Solver` trait from my `AocParser` and `PartSolver<N>` implementations, so that I don't have to write boilerplate code.

#### Acceptance Criteria

1. WHEN a user applies `#[derive(AocSolver)]` with `#[aoc_solver(max_parts = N)]` THEN the system SHALL generate a `Solver` trait implementation
2. WHEN the macro generates `Solver` THEN the system SHALL NOT generate redundant `SharedData` type or `parse()` delegation since `Solver: AocParser`
3. WHEN the macro generates `Solver::solve_part` THEN the system SHALL dispatch to the appropriate `PartSolver<N>::solve` based on the part number
4. WHEN the macro generates `Solver::PARTS` THEN the system SHALL set it to the value specified in `#[aoc_solver(max_parts = N)]`
5. WHEN `solve_part` receives a part number outside the valid range THEN the system SHALL return `SolveError::PartNotImplemented`
6. WHEN `#[aoc_solver(max_parts = N)]` is specified THEN the macro SHALL verify at compile-time that `PartSolver<M>` is implemented for all M in 1..=N

### Requirement 4

**User Story:** As a solver implementer, I want `Solver` to extend `AocParser`, so that parsing logic is defined once without duplication.

#### Acceptance Criteria

1. WHEN the `Solver` trait is defined THEN the trait SHALL have `AocParser` as a supertrait
2. WHEN a type implements `Solver` THEN the type SHALL also implement `AocParser` (required by supertrait)
3. WHEN a user manually implements `Solver` THEN the user SHALL split the implementation into `AocParser` (for `SharedData` and `parse()`) and `Solver` (for `PARTS` and `solve_part()`)

### Requirement 5

**User Story:** As a maintainer, I want to remove the `#[aoc_solver]` attribute macro, so that we have a single, cleaner approach to solver implementation.

#### Acceptance Criteria

1. WHEN the redesign is complete THEN the system SHALL remove the `#[aoc_solver]` attribute macro from the codebase
2. WHEN the `#[aoc_solver]` macro is removed THEN the system SHALL update all examples to use the new `AocParser` + `PartSolver<N>` approach
3. WHEN the `#[aoc_solver]` macro is removed THEN the system SHALL update all tests to use the new approach
