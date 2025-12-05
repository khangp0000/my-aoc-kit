# Requirements Document

## Introduction

This document specifies requirements for an attribute macro that simplifies the implementation of Advent of Code solvers by automatically generating the `Solver` trait implementation. The macro reduces boilerplate code while maintaining type safety and providing excellent compile-time error messages.

## Glossary

- **Solver**: A type that implements the `Solver` trait from the `aoc-solver` library
- **Macro**: The `#[aoc_solver]` attribute macro that generates trait implementations
- **Parsed**: The intermediate representation type after parsing input
- **PartialResult**: The type of data shared between dependent parts
- **Part Function**: A function named `part1`, `part2`, etc. that solves a specific part
- **Independent Parts**: Parts that do not share data (PartialResult = ())
- **Dependent Parts**: Parts that share data via PartialResult
- **User Code**: The impl block annotated with `#[aoc_solver]`
- **Generated Code**: The code produced by the macro expansion

## Requirements

### Requirement 1

**User Story:** As a solver developer, I want to use an attribute macro to generate the Solver trait implementation, so that I can focus on solving logic without writing boilerplate.

#### Acceptance Criteria

1. WHEN an impl block is annotated with `#[aoc_solver(max_parts = N)]` THEN the system SHALL generate a complete `Solver` trait implementation
2. WHEN the user defines required types and functions THEN the system SHALL forward them to the trait implementation
3. WHEN the user defines part functions THEN the system SHALL generate a `solve_part` method that dispatches to them

### Requirement 2

**User Story:** As a solver developer, I want to declare required types in the impl block, so that the macro knows what types to use for the trait implementation.

#### Acceptance Criteria

1. WHEN `type Parsed = T` is defined THEN the system SHALL use T as Solver::Parsed
2. WHEN `type PartialResult = T` is defined THEN the system SHALL use T as Solver::PartialResult
3. WHEN Parsed or PartialResult is missing THEN the system SHALL emit a compile error with example syntax

### Requirement 3

**User Story:** As a solver developer, I want to define a parse function in the impl block, so that the macro can generate the trait's parse method.

#### Acceptance Criteria

1. WHEN `fn parse(input: &str) -> Result<Parsed, ParseError>` is defined THEN the system SHALL forward it to Solver::parse
2. WHEN parse is missing THEN the system SHALL emit a compile error with example syntax
3. WHEN parse has incorrect signature THEN the system SHALL produce a type error

### Requirement 4

**User Story:** As a solver developer, I want to define part functions with flexible return types, so that I can write concise code for simple cases and detailed code for complex cases.

#### Acceptance Criteria

1. WHEN a part function returns `String` THEN the system SHALL wrap it in `PartResult { answer, partial: None }`
2. WHEN a part function returns `Result<String, SolveError>` THEN the system SHALL unwrap with `?` and wrap the result
3. WHEN a part function returns `PartResult<PartialResult>` THEN the system SHALL use it directly
4. WHEN a part function returns `Result<PartResult<PartialResult>, SolveError>` THEN the system SHALL use it directly
5. WHEN a part function returns an unsupported type THEN the system SHALL emit a compile error

### Requirement 5

**User Story:** As a solver developer, I want to specify the maximum part number, so that the macro validates all expected parts are implemented.

#### Acceptance Criteria

1. WHEN the macro has attribute `#[aoc_solver(max_parts = N)]` THEN the system SHALL verify parts 1 through N exist
2. WHEN max_parts is missing THEN the system SHALL emit a compile error
3. WHEN a part from 1 to N is missing THEN the system SHALL emit a compile error naming the missing part
4. WHEN a part number exceeds N THEN the system SHALL emit a compile error
5. WHEN max_parts is less than 1 THEN the system SHALL emit a compile error

### Requirement 6

**User Story:** As a solver developer, I want to support independent parts with simple signatures, so that I can write clean code when parts don't share data.

#### Acceptance Criteria

1. WHEN a part function has signature `fn partN(parsed: &Parsed) -> ReturnType` THEN the system SHALL call it with only parsed data
2. WHEN a part returns String or Result<String, SolveError> THEN the system SHALL set partial to None

### Requirement 7

**User Story:** As a solver developer, I want to support dependent parts that share data, so that I can pass computation results between parts efficiently.

#### Acceptance Criteria

1. WHEN a part has signature `fn partN(parsed: &Parsed, prev: Option<&PartialResult>) -> ReturnType` THEN the system SHALL pass previous_partial
2. WHEN a part returns `PartResult<PartialResult>` with `partial: Some(data)` THEN the system SHALL make data available to subsequent parts

### Requirement 8

**User Story:** As a solver developer, I want clear compile-time error messages when I make mistakes, so that I can quickly identify and fix issues.

#### Acceptance Criteria

1. WHEN required types or functions are missing THEN the system SHALL emit compile_error with example syntax
2. WHEN a function signature is incorrect THEN the system SHALL produce a type error
3. WHEN a return type is unsupported THEN the system SHALL emit compile_error listing supported types

### Requirement 9

**User Story:** As a solver developer, I want to define the struct myself before using the macro, so that I can add derives and attributes like AutoRegisterSolver.

#### Acceptance Criteria

1. WHEN the user defines a struct before the impl block THEN the system SHALL use that struct for the Solver trait implementation
2. WHEN the user applies both AutoRegisterSolver and aoc_solver to the same struct THEN the system SHALL compile without conflicts

### Requirement 10

**User Story:** As a solver developer, I want the generated code to use fully qualified paths, so that it works regardless of import statements.

#### Acceptance Criteria

1. WHEN the macro generates code THEN the system SHALL use fully qualified paths like `::aoc_solver::Solver`
2. WHEN the generated code is compiled without explicit imports THEN the system SHALL compile successfully

### Requirement 11

**User Story:** As a solver developer, I want the macro to distinguish between parts that don't exist versus parts not yet implemented, so that automation can handle completion correctly.

#### Acceptance Criteria

1. WHEN solve_part receives a part number within 1 to max_parts THEN the system SHALL call the corresponding part function
2. WHEN solve_part receives a part number greater than max_parts THEN the system SHALL return `Err(SolveError::PartOutOfRange(part))`

### Requirement 12

**User Story:** As a library maintainer, I want a new error variant for out-of-range parts, so that the system can distinguish between non-existent and unimplemented parts.

#### Acceptance Criteria

1. WHEN the SolveError enum is extended THEN the system SHALL add a `PartOutOfRange(usize)` variant
2. WHEN PartOutOfRange is displayed THEN the system SHALL show a message like "Part N is out of range"
3. WHEN existing code uses SolveError THEN the system SHALL remain backward compatible with PartNotImplemented

### Requirement 13

**User Story:** As a solver developer, I want to use `#[aoc_solver]` together with `#[derive(AutoRegisterSolver)]`, so that I can both generate the trait implementation and register the solver automatically.

#### Acceptance Criteria

1. WHEN both `#[derive(AutoRegisterSolver)]` and `#[aoc_solver]` are used THEN the system SHALL generate both implementations
2. WHEN both macros are used together THEN the system SHALL compile without conflicts
