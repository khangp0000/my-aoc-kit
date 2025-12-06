# Implementation Plan

## Phase 1: Registry Extensions (aoc-solver)

- [x] 1. Extend aoc-solver registry with flat Vec storage and parts metadata
  - [x] 1.1 Add `FactoryInfo`, `SolverFactoryEntry`, and storage constants
  - [x] 1.2 Implement `SolverFactoryStorage` with iteration and lookup
  - [x] 1.3 Implement `FactoryRegistryBuilder` with validation
  - [x] 1.4 Implement `SolverFactoryRegistry` wrapper
  - [x] 1.5 Update `RegisterableSolver` blanket impl to use `register_factory`
  - [ ]* 1.6 Write property test for registry index calculation

- [x] 2. Checkpoint - Ensure all tests pass

## Phase 2: CLI Crate Setup (aoc-cli)

- [x] 3. Create aoc-cli crate with CLI parser
  - [x] 3.1 Initialize crate structure
  - [x] 3.2 Implement CLI argument parsing with clap
    - `ParallelizeBy` enum (Sequential, Year, Day, Part) with default Day
    - All CLI options implemented
  - [x] 3.3 Implement error types
  - [ ]* 3.4 Write property test for error message formatting

## Phase 3: Configuration and Cache

- [x] 4. Implement configuration resolution
  - [x] 4.1 Implement `Config` struct and `from_args`
    - `parallelize_by: ParallelizeBy` field implemented
  - [x] 4.2 Implement session and user ID resolution
  - [ ]* 4.3 Write property test for user ID verification

- [x] 5. Implement input cache
  - [x] 5.1 Implement `InputCache` struct
  - [x] 5.2 Implement cache read/write operations
  - [ ]* 5.3 Write property tests for cache

- [x] 6. Checkpoint - Ensure all tests pass

## Phase 4: Executor Implementation

- [x] 7. Implement parallel executor
  - [x] 7.1 Implement `WorkItem` and `SolverResult` types
  - [x] 7.2 Implement `Executor` struct and work item collection
  - [x] 7.3 Implement input fetching with cache
  - [x] 7.4 Implement `execute` with parallelization levels
  - [x] 7.5 Implement `run_solver` for part-level parallel mode
  - [x] 7.6 Implement `run_solver` for sequential mode
  - [ ]* 7.7 Write property test for parallelization level behavior
  - [x] 7.8 Implement submission with retry
  - [ ]* 7.9 Write property test for filter matching

## Phase 5: Result Aggregation and Output

- [x] 8. Implement result aggregator
  - [x] 8.1 Implement `ResultKey` with ordering
  - [x] 8.2 Implement `ResultAggregator` with min-heaps
  - [ ]* 8.3 Write property test for result ordering

- [x] 9. Implement output formatter
  - [x] 9.1 Implement `OutputFormatter` struct
  - [ ]* 9.2 Write property test for summary timing
  - [ ]* 9.3 Write property tests for output formatting
  - [ ]* 9.4 Write property test for submission result handling

- [x] 10. Checkpoint - Ensure all tests pass

## Phase 6: Main Entry Point and Integration

- [x] 11. Implement main entry point
  - [x] 11.1 Wire up main function
  - [x] 11.2 Implement execution loop with aggregator
  - [ ]* 11.3 Write property test for error aggregation

- [x] 12. Checkpoint - Ensure all tests pass

## Phase 7: Solutions Crate

- [x] 13. Create aoc-solutions crate structure
  - [x] 13.1 Initialize crate
  - [x] 13.2 Add example solution with auto-registration

- [x] 14. Final Checkpoint - Ensure all tests pass

## Phase 8: Integration Testing

- [-] 15. Create mock AoC server crate for testing
  - [x] 15.1 Initialize aoc-mock-server crate
  - [ ] 15.2 Implement mock endpoints

- [ ] 16. Create stress test with 100 mock solvers
  - [x] 16.1 Generate 100 test solvers in aoc-solutions
  - [ ] 16.2 Run integration test without submission
  - [ ] 16.3 Run integration test with mock submission

- [ ] 17. Final Integration Checkpoint
