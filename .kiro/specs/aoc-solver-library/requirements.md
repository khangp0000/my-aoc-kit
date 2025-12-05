# Requirements Document

## Introduction

This document specifies the requirements for an Advent of Code solver library. The library provides a flexible framework for solving Advent of Code problems across multiple years and days. Each problem (identified by year and day) has its own solver with a custom input parser and can produce results for multiple parts. The system is designed to be extensible, allowing new solvers to be added easily while maintaining a simple and consistent interface.

## Glossary

- **Solver**: A component that handles a specific Advent of Code problem identified by a year and day combination
- **Input Parser**: A function within a Solver that transforms raw input text into an intermediate data structure
- **SharedData**: The data structure holding input data and intermediate results that can be mutated by parts
- **Part**: A sub-problem within a day's challenge (typically Part 1 and Part 2)
- **Part Result**: The solution output for a specific Part, returned as a String
- **AOC Library**: The Advent of Code solver library system

## Requirements

### Requirement 1

**User Story:** As a developer, I want to create a solver for a specific year and day combination, so that I can solve Advent of Code problems in an organized manner.

#### Acceptance Criteria

1. WHEN a developer creates a Solver with year, day, and input string THEN the AOC Library SHALL instantiate a new Solver instance with the specified parameters
2. WHEN a Solver is created THEN the AOC Library SHALL invoke the Input Parser to transform the raw input string into the SharedData structure
3. WHEN multiple Solvers are created for different year-day combinations THEN the AOC Library SHALL maintain each Solver independently without interference
4. WHEN a Solver is created with invalid input THEN the AOC Library SHALL handle the error gracefully and provide meaningful feedback

### Requirement 2

**User Story:** As a developer, I want each solver to have its own input parser, so that I can handle different input formats for different problems.

#### Acceptance Criteria

1. WHEN a Solver defines an Input Parser THEN the AOC Library SHALL use that parser to process the input string during Solver creation
2. WHEN different Solvers define different SharedData types THEN the AOC Library SHALL support type-safe parsing for each Solver
3. WHEN an Input Parser processes valid input THEN the AOC Library SHALL store the resulting SharedData within the Solver
4. WHEN an Input Parser encounters malformed input THEN the AOC Library SHALL propagate the parsing error to the caller

### Requirement 3

**User Story:** As a developer, I want to solve specific parts of a problem by calling a function with the part number, so that I can compute solutions incrementally.

#### Acceptance Criteria

1. WHEN a developer calls the solve function with a part number THEN the AOC Library SHALL execute the solution logic for that specific part
2. WHEN a part is solved successfully THEN the AOC Library SHALL return the computed result
3. WHEN a part has not been implemented THEN the AOC Library SHALL return None for that part
4. WHEN an invalid part number is provided THEN the AOC Library SHALL handle the error appropriately

### Requirement 4

**User Story:** As a developer, I want part results stored in a vector of Options, so that I can track which parts have been solved.

#### Acceptance Criteria

1. WHEN parts are solved THEN the AOC Library SHALL store each result in a vector at the index corresponding to the part number
2. WHEN a part has not been solved THEN the AOC Library SHALL store None at that part's index
3. WHEN retrieving part results THEN the AOC Library SHALL provide access to the complete vector of Option values
4. WHEN multiple parts exist THEN the AOC Library SHALL maintain the ordering and indexing consistently

### Requirement 5

**User Story:** As a developer, I want to easily add new year-day combinations, so that I can extend the library as new Advent of Code problems are released.

#### Acceptance Criteria

1. WHEN a developer implements a new Solver for a year-day combination THEN the AOC Library SHALL integrate it without requiring changes to the core library code
2. WHEN new Solvers are added THEN the AOC Library SHALL discover and register them automatically or through a simple registration mechanism
3. WHEN a Solver is requested for a specific year and day THEN the AOC Library SHALL locate and instantiate the appropriate Solver implementation
4. WHEN no Solver exists for a requested year-day combination THEN the AOC Library SHALL indicate that the Solver is not available

### Requirement 6

**User Story:** As a developer, I want a simple and consistent interface for all solvers, so that I can use the library without learning different APIs for each problem.

#### Acceptance Criteria

1. WHEN interacting with any Solver THEN the AOC Library SHALL provide a uniform interface for creation and solving
2. WHEN solving parts across different Solvers THEN the AOC Library SHALL use the same function signature and calling convention
3. WHEN accessing Solver functionality THEN the AOC Library SHALL expose only the necessary methods and hide implementation details
4. WHEN working with the library THEN the AOC Library SHALL require minimal boilerplate code for common operations

### Requirement 7

**User Story:** As a developer, I want the library to be type-safe, so that I can catch errors at compile time rather than runtime.

#### Acceptance Criteria

1. WHEN defining Solvers with different SharedData types THEN the AOC Library SHALL enforce type safety through the Rust type system
2. WHEN calling solve functions THEN the AOC Library SHALL ensure type correctness for inputs and outputs at compile time
3. WHEN composing Solvers THEN the AOC Library SHALL prevent type mismatches through static type checking
4. WHEN using generic functionality THEN the AOC Library SHALL maintain type safety across all generic operations

### Requirement 8

**User Story:** As a developer, I want Part 2 to access Part 1's result when needed, so that I can implement problems where Part 2 depends on Part 1's solution.

#### Acceptance Criteria

1. WHEN solving a part THEN the AOC Library SHALL provide access to all previously solved part results
2. WHEN Part 2 depends on Part 1's result THEN the AOC Library SHALL allow Part 2 to retrieve and use Part 1's answer
3. WHEN parts are independent THEN the AOC Library SHALL allow solvers to ignore previous results
4. WHEN a part accesses previous results THEN the AOC Library SHALL provide them in a consistent indexed format

### Requirement 9

**User Story:** As a developer, I want to automatically register multiple solvers without manually calling register for each one, so that I can manage large collections of solvers efficiently.

#### Acceptance Criteria

1. WHEN a solver implements a registration trait THEN the AOC Library SHALL allow that solver to register itself with a registry given year and day parameters
2. WHEN multiple solvers are collected in a plugin system THEN the AOC Library SHALL support automatic discovery and registration of all plugins
3. WHEN a plugin is submitted with year, day, solver information, and tags THEN the AOC Library SHALL store that plugin with its metadata for later registration
4. WHEN a mass registration function is called THEN the AOC Library SHALL iterate through all collected plugins and register each solver with the registry
5. WHEN a filtered registration function is called with a predicate THEN the AOC Library SHALL register only the plugins that satisfy the predicate function
6. WHEN plugins have tags THEN the AOC Library SHALL allow filtering based on those tags to selectively register subsets of solvers

### Requirement 10

**User Story:** As a developer, I want a fluent builder API for constructing the registry, so that I can chain registration calls and ensure the registry is immutable after construction.

#### Acceptance Criteria

1. WHEN using a registry builder THEN the AOC Library SHALL provide a builder type that consumes and returns self for method chaining
2. WHEN registering a solver with a year-day combination that already exists THEN the AOC Library SHALL return an error indicating the duplicate registration
3. WHEN the builder is finalized THEN the AOC Library SHALL produce an immutable registry that cannot be modified
4. WHEN using the immutable registry THEN the AOC Library SHALL only allow solver lookup and creation operations
5. WHEN registration methods are chained THEN the AOC Library SHALL maintain a fluent interface that enables readable construction code

### Requirement 11

**User Story:** As a developer, I want to use a derive macro to automatically register my solvers, so that I can eliminate boilerplate code and reduce the chance of registration errors.

#### Acceptance Criteria

1. WHEN a solver struct is annotated with a derive macro THEN the AOC Library SHALL automatically generate the plugin submission code
2. WHEN the derive macro is used with year and day attributes THEN the AOC Library SHALL extract those values and include them in the plugin submission
3. WHEN the derive macro is used with optional tags THEN the AOC Library SHALL include those tags in the plugin metadata
4. WHEN a solver is annotated with the derive macro THEN the AOC Library SHALL ensure the solver implements the Solver trait before generating registration code
5. WHEN multiple solvers use the derive macro THEN the AOC Library SHALL generate unique plugin submissions for each solver without conflicts
