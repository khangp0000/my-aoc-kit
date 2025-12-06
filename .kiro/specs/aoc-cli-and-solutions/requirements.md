# Requirements Document

## Introduction

This document specifies requirements for two new crates in the Advent of Code solver workspace:

1. **aoc-cli** - A command-line interface for running AoC solvers with input caching, parallel execution, and optional answer submission.
2. **aoc-solutions** - A crate holding actual solution implementations with automatic registration via the existing plugin system.

The CLI integrates with the existing `aoc-solver` (solver framework) and `aoc-http-client` (AoC website interaction) crates.

## Glossary

- **AoC**: Advent of Code, an annual programming puzzle event
- **CLI**: Command-Line Interface application
- **Session Key**: A cookie value from adventofcode.com used for authentication
- **User ID**: A numeric identifier for an AoC user account
- **Solver**: An implementation of puzzle logic for a specific year/day
- **Plugin**: A solver registered via the `inventory` crate for automatic discovery
- **Tag**: A string label attached to solvers for filtering (e.g., "2024", "easy")
- **Part**: A puzzle subdivision (1 or 2) within a day
- **Throttle**: Rate limiting imposed by the AoC website on submissions
- **Input Cache**: Local storage of puzzle inputs to avoid repeated fetches
- **Zeroizing**: Secure memory clearing to prevent sensitive data leakage

## Requirements

### Requirement 1: CLI Configuration

**User Story:** As a user, I want to configure the CLI via command-line options, so that I can customize execution without modifying code.

#### Acceptance Criteria

1. WHEN a user provides year, day, or part options THEN the CLI SHALL filter solver execution to match the specified criteria
2. WHEN a user omits year, day, or part options THEN the CLI SHALL run all registered solvers matching the unspecified criteria
3. WHEN a user provides a cache directory option THEN the CLI SHALL use that directory for input caching
4. WHEN a user omits the cache directory option THEN the CLI SHALL default to `~/.cache/aoc_solver`
5. WHEN a user provides tag filters THEN the CLI SHALL only register solvers matching those tags
6. WHEN a user provides a thread count option THEN the CLI SHALL limit parallelism to that number of threads
7. WHEN a user provides a parallelize-by option THEN the CLI SHALL parallelize execution at the specified level (year, day, or part)

### Requirement 2: Input Caching

**User Story:** As a user, I want puzzle inputs cached locally, so that I can run solvers offline and avoid redundant network requests.

#### Acceptance Criteria

1. WHEN the CLI needs input for a year/day/user combination THEN the CLI SHALL first check the local cache
2. WHEN cached input exists THEN the CLI SHALL use the cached input without network requests
3. WHEN cached input does not exist and a session key is available THEN the CLI SHALL fetch input from AoC and cache it
4. THE CLI SHALL organize cached inputs by user ID, year, and day in the cache directory
5. WHEN writing to the cache THEN the CLI SHALL create parent directories as needed

### Requirement 3: Session Key Handling

**User Story:** As a user, I want secure session key handling, so that my AoC credentials remain protected.

#### Acceptance Criteria

1. WHEN a session key is needed and not provided THEN the CLI SHALL prompt for it using hidden input
2. WHEN the `AOC_SESSION` environment variable is set THEN the CLI SHALL use that value as the session key
3. THE CLI SHALL wrap session keys in a Zeroizing container to clear memory after use
4. WHEN a user ID is provided via CLI THEN the CLI SHALL verify the session key matches that user ID
5. IF the fetched user ID does not match the provided user ID THEN the CLI SHALL terminate with an error
6. WHEN a session key is provided without a user ID THEN the CLI SHALL fetch and use the user ID from the session

### Requirement 4: Parallel Execution

**User Story:** As a user, I want solvers to run in parallel, so that I can get results faster on multi-core systems.

#### Acceptance Criteria

1. THE CLI SHALL support four parallelization levels: sequential, year, day, and part
2. WHEN parallelize-by is set to "sequential" THEN the CLI SHALL execute all solvers sequentially in year/day/part order
3. WHEN parallelize-by is set to "year" THEN the CLI SHALL parallelize across years while executing days and parts sequentially within each year
4. WHEN parallelize-by is set to "day" THEN the CLI SHALL parallelize across year/day combinations while executing parts sequentially within each solver
5. WHEN parallelize-by is set to "part" THEN the CLI SHALL parallelize across all year/day/part combinations
6. THE CLI SHALL default to "day" level parallelization to balance performance and shared data optimization
7. WHEN a thread count is specified THEN the CLI SHALL limit the thread pool to that size
8. THE CLI SHALL use a configurable default thread count when not specified

### Requirement 5: Answer Submission

**User Story:** As a user, I want to optionally submit answers to AoC, so that I can verify my solutions directly from the CLI.

#### Acceptance Criteria

1. WHEN the submit flag is present THEN the CLI SHALL submit computed answers to AoC
2. WHEN submitting answers THEN the CLI SHALL display the submission result (correct, incorrect, already completed, or throttled)
3. WHEN a submission is throttled with a parseable wait time THEN the CLI SHALL display the wait duration
4. WHEN auto-retry is enabled and a submission is throttled with a parseable wait time THEN the CLI SHALL wait and retry automatically
5. THE CLI SHALL disable auto-retry by default

### Requirement 6: Error Handling

**User Story:** As a user, I want clear error messages, so that I can diagnose and fix issues quickly.

#### Acceptance Criteria

1. THE CLI SHALL use `thiserror` for error type definitions
2. THE CLI SHALL use `thiserror-ext` for Arc-wrapped errors to support multi-threaded contexts
3. WHEN an error occurs THEN the CLI SHALL display a human-readable error message
4. WHEN multiple solvers fail THEN the CLI SHALL report all failures rather than stopping at the first

### Requirement 7: Solutions Crate Structure

**User Story:** As a developer, I want a dedicated crate for solutions, so that puzzle implementations are organized separately from infrastructure.

#### Acceptance Criteria

1. THE solutions crate SHALL use the `AutoRegisterSolver` derive macro for automatic plugin registration
2. THE solutions crate SHALL organize solutions by year in separate modules
3. WHEN a solution is compiled THEN the solution SHALL be automatically registered with the plugin system
4. THE solutions crate SHALL depend on `aoc-solver` for the solver traits and macros

### Requirement 8: Output Formatting

**User Story:** As a user, I want clear output formatting, so that I can easily read solver results.

#### Acceptance Criteria

1. WHEN displaying results THEN the CLI SHALL show year, day, part, and answer for each solution
2. WHEN displaying results THEN the CLI SHALL show execution time for each solver
3. WHEN a solver fails THEN the CLI SHALL display the error alongside the year/day/part identifier
4. THE CLI SHALL support a quiet mode that only outputs answers
5. WHEN displaying a summary THEN the CLI SHALL show both total solve time (sum of all solver durations) and actual elapsed wall-clock time

### Requirement 9: Result Aggregation and Streaming

**User Story:** As a user, I want results displayed progressively in chronological order, so that I can see early results while waiting for slower solvers.

#### Acceptance Criteria

1. THE CLI SHALL aggregate results from parallel solver executions
2. THE CLI SHALL display results in year/day/part order regardless of completion order
3. WHEN a result is ready and all preceding year/day/part combinations have completed THEN the CLI SHALL display that result immediately
4. WHEN a result is ready but preceding combinations are still running THEN the CLI SHALL buffer that result until predecessors complete
5. THE CLI SHALL maintain a sorted buffer of pending results for ordered output

### Requirement 10: Testing Configuration

**User Story:** As a developer, I want efficient test execution, so that compile times remain reasonable.

#### Acceptance Criteria

1. WHEN property-based tests are configured THEN the test suite SHALL run a maximum of 10 iterations per property
2. THE test dependencies SHALL be optional and gated behind a `test` feature
3. THE workspace SHALL use the latest minor versions of all dependencies
