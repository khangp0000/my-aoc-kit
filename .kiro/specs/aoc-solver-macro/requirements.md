# Requirements Document

## Introduction

This document specifies requirements for an attribute macro that simplifies the implementation of Advent of Code solvers by automatically generating the `Solver` trait implementation. The macro reduces boilerplate code while maintaining type safety and providing excellent compile-time error messages.

## Glossary

- **Solver**: A type that implements the `Solver` trait from the `aoc-solver` library
- **Macro**: The `#[aoc_solver]` attribute macro that generates trait implementations
- **SharedData**: The data structure holding parsed input and intermediate results that can be mutated by parts
- **Part Function**: A function named `part1`, `part2`, etc. that solves a specific part
- **Independent Parts**: Parts that don't modify shared data beyond reading it
- **Dependent Parts**: Parts that store data in SharedData for later parts to read
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

**User Story:** As a solver developer, I want to declare the SharedData type in the impl block, so that the macro knows what type to use for the trait implementation.

#### Acceptance Criteria

1. WHEN `type SharedData = T` is defined THEN the system SHALL use T as Solver::SharedData
2. WHEN SharedData is missing THEN the system SHALL emit a compile error with example syntax

### Requirement 3

**User Story:** As a solver developer, I want to define a parse function in the impl block, so that the macro can generate the trait's parse method.

#### Acceptance Criteria

1. WHEN `fn parse(input: &str) -> Result<SharedData, ParseError>` is defined THEN the system SHALL forward it to Solver::parse
2. WHEN parse is missing THEN the system SHALL emit a compile error with example syntax
3. WHEN parse has incorrect signature THEN the system SHALL produce a type error

### Requirement 4

**User Story:** As a solver developer, I want to define part functions with flexible return types, so that I can write concise code for simple cases and detailed code for complex cases.

#### Acceptance Criteria

1. WHEN a part function returns `String` THEN the system SHALL return it directly
2. WHEN a part function returns `Result<String, SolveError>` THEN the system SHALL return it directly
3. WHEN a part function returns an unsupported type THEN the system SHALL emit a compile error

### Requirement 5

**User Story:** As a solver developer, I want to specify the maximum part number, so that the macro validates all expected parts are implemented.

#### Acceptance Criteria

1. WHEN the macro has attribute `#[aoc_solver(max_parts = N)]` THEN the system SHALL verify parts 1 through N exist
2. WHEN max_parts is missing THEN the system SHALL emit a compile error
3. WHEN a part from 1 to N is missing THEN the system SHALL emit a compile error naming the missing part
4. WHEN a part number exceeds N THEN the system SHALL emit a compile error
5. WHEN max_parts is less than 1 THEN the system SHALL emit a compile error

### Requirement 6

**User Story:** As a solver developer, I want all part functions to receive mutable access to shared data, so that I can read and modify it as needed.

#### Acceptance Criteria

1. WHEN a part function has signature `fn partN(shared: &mut SharedData) -> ReturnType` THEN the system SHALL call it with mutable access to shared data
2. WHEN a part function has incorrect signature THEN the system SHALL emit a compile error

### Requirement 7

**User Story:** As a solver developer, I want to support dependent parts that share data, so that I can pass computation results between parts efficiently.

#### Acceptance Criteria

1. WHEN Part 1 stores data in SharedData fields THEN Part 2 SHALL be able to read those fields
2. WHEN Part 2 reads SharedData fields that Part 1 didn't populate THEN Part 2 SHALL handle missing data gracefully (e.g., using Option types)

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

**User Story:** As a solver developer, I want the macro to handle invalid part numbers correctly, so that errors are clear.

#### Acceptance Criteria

1. WHEN solve_part receives a part number within 1 to max_parts THEN the system SHALL call the corresponding part function
2. WHEN solve_part receives a part number greater than max_parts THEN the system SHALL return `Err(SolveError::PartNotImplemented(part))`

### Requirement 13

**User Story:** As a solver developer, I want to use `#[aoc_solver]` together with `#[derive(AutoRegisterSolver)]`, so that I can both generate the trait implementation and register the solver automatically.

#### Acceptance Criteria

1. WHEN both `#[derive(AutoRegisterSolver)]` and `#[aoc_solver]` are used THEN the system SHALL generate both implementations
2. WHEN both macros are used together THEN the system SHALL compile without conflicts
