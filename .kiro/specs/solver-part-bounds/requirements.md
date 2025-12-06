# Requirements Document

## Introduction

Add part count awareness to the solver system, enabling runtime validation of part numbers before solving. This allows callers to query how many parts a solver supports and get clear errors when requesting out-of-range parts.

## Glossary

- **Solver**: A type implementing the `Solver` trait that solves AOC puzzle parts
- **SolverExt**: Extension trait providing range-validated solving via blanket impl
- **DynSolver**: Type-erased trait for dynamic dispatch of solvers
- **Part**: A numbered section of an AOC puzzle (typically 1 or 2)
- **Part Bounds**: The valid range of parts a solver supports (1 to PARTS)

## Requirements

### Requirement 1

**User Story:** As a library user, I want to know how many parts a solver supports, so that I can validate inputs before attempting to solve.

#### Acceptance Criteria

1. WHEN a solver is defined THEN the Solver trait SHALL expose a const `PARTS: u8` indicating the number of supported parts
2. WHEN querying a DynSolver THEN the system SHALL provide a `parts()` method returning the part count

### Requirement 2

**User Story:** As a library user, I want range-validated solving, so that I get clear errors for invalid part numbers.

#### Acceptance Criteria

1. WHEN calling `solve(0)` via DynSolver THEN the system SHALL return `SolveError::PartOutOfRange(0)`
2. WHEN calling `solve(part)` where part > PARTS THEN the system SHALL return `SolveError::PartOutOfRange(part)`
3. WHEN calling `solve(part)` where 1 <= part <= PARTS THEN the system SHALL delegate to `Solver::solve_part(part)`

### Requirement 3

**User Story:** As a library maintainer, I want the validation logic to be non-overridable, so that the behavior is consistent across all solvers.

#### Acceptance Criteria

1. WHEN a user implements Solver THEN the system SHALL automatically provide `solve_part_checked_range` via SolverExt blanket impl
2. WHEN a user attempts to override `solve_part_checked_range` THEN the Rust compiler SHALL reject the code due to conflicting impls
