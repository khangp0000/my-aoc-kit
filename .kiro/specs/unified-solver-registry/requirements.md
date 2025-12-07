# Requirements Document

## Introduction

This document specifies the requirements for consolidating the dual registry system in the `aoc-solver` crate. Currently, there are two separate registry implementations: `SolverRegistry` with `RegistryBuilder` (HashMap-based) and `SolverFactoryRegistry` with `FactoryRegistryBuilder` (flat Vec-based). This refactoring will unify them into a single `SolverRegistry` with `SolverRegistryStorage` as the internal storage mechanism, eliminating redundancy and simplifying the public API.

## Glossary

- **SolverRegistry**: The unified registry that manages solver factories and provides solver creation capabilities
- **SolverRegistryBuilder**: Builder for constructing SolverRegistry instances
- **SolverRegistryStorage**: The internal storage mechanism using a flat Vec for efficient year/day indexing
- **SolverFactory**: A thread-safe function that creates solver instances from input strings (always Send + Sync)
- **SolverInfo**: Metadata about a registered solver (year, day, parts count)
- **SolverFactoryEntry**: Internal storage entry containing a factory and its parts count
- **RegisterableSolver**: Trait for types that can register themselves with the builder
- **DynSolver**: A type-erased solver instance that can solve puzzle parts

## Requirements

### Requirement 1

**User Story:** As a library user, I want a single unified registry type, so that I don't have to choose between multiple registry implementations.

#### Acceptance Criteria

1. WHEN a user imports the aoc-solver library THEN the library SHALL export exactly one registry type named `SolverRegistry`
2. WHEN a user creates a registry THEN the SolverRegistry SHALL use `SolverRegistryStorage` as its internal storage mechanism
3. WHEN a user attempts to use the old `SolverFactoryRegistry` type THEN the compiler SHALL report that the type does not exist
4. WHEN a user attempts to use the old `FactoryRegistryBuilder` type THEN the compiler SHALL report that the type does not exist

### Requirement 2

**User Story:** As a library user, I want to build registries using a fluent builder pattern, so that I can register solvers in a clean, chainable way.

#### Acceptance Criteria

1. WHEN a user calls `SolverRegistryBuilder::new()` THEN the builder SHALL create an empty builder with pre-allocated storage for years 2015-2034 and days 1-25
2. WHEN a user calls `register_factory()` on the builder THEN the builder SHALL store the factory with its parts count and return a mutable reference for chaining
3. WHEN a user calls `register_factory()` with a duplicate year/day THEN the builder SHALL return a `RegistrationError::DuplicateSolverFactory` error
4. WHEN a user calls `register_factory()` with an out-of-bounds year or day THEN the builder SHALL return a `RegistrationError::InvalidYearDay` error
5. WHEN a user calls `build()` on the builder THEN the builder SHALL produce an immutable `SolverRegistry` instance

### Requirement 3

**User Story:** As a library user, I want to access storage metadata and iterate over registered solvers, so that I can discover available solvers and their capabilities.

#### Acceptance Criteria

1. WHEN a user calls `storage()` on SolverRegistry THEN the registry SHALL return a reference to the internal `SolverRegistryStorage`
2. WHEN a user calls `iter_info()` on storage THEN the storage SHALL yield `SolverInfo` items in ascending (year, day) order
3. WHEN a user calls `get_info(year, day)` on storage THEN the storage SHALL return `Some(SolverInfo)` if registered or `None` if not
4. WHEN a user calls `contains(year, day)` on storage THEN the storage SHALL return `true` if a factory exists for that year/day
5. WHEN a user calls `len()` on storage THEN the storage SHALL return the count of registered factories
6. WHEN a user calls `is_empty()` on storage THEN the storage SHALL return `true` only when no factories are registered

### Requirement 4

**User Story:** As a library user, I want to create solver instances from the registry, so that I can solve puzzle inputs.

#### Acceptance Criteria

1. WHEN a user calls `create_solver(year, day, input)` on SolverRegistry THEN the registry SHALL invoke the registered factory and return the solver instance
2. WHEN a user calls `create_solver()` with an unregistered year/day THEN the registry SHALL return `SolverError::NotFound`
3. WHEN a user calls `create_solver()` with an out-of-bounds year/day THEN the registry SHALL return `SolverError::InvalidYearDay`
4. WHEN the factory function returns a parse error THEN the registry SHALL wrap it in `SolverError::ParseError`

### Requirement 5

**User Story:** As a library user, I want backward compatibility with the plugin system, so that existing solver registrations continue to work.

#### Acceptance Criteria

1. WHEN a user calls `register_all_plugins()` on the builder THEN the builder SHALL register all solvers submitted via `inventory::submit!`
2. WHEN a user calls `register_solver_plugins(filter)` on the builder THEN the builder SHALL register only plugins matching the filter predicate
3. WHEN the `RegisterableSolver` trait is used THEN the trait SHALL continue to work with the new `SolverRegistryBuilder`
4. WHEN the `SolverPlugin` struct is used with inventory THEN the plugin system SHALL function identically to before

### Requirement 6

**User Story:** As a library user, I want the `register_solver!` macro to continue working, so that existing code using the macro doesn't break.

#### Acceptance Criteria

1. WHEN a user invokes `register_solver!(builder, SolverType, year, day)` THEN the macro SHALL register the solver with the builder
2. WHEN the macro is used with the new SolverRegistryBuilder THEN the macro SHALL produce identical behavior to the previous implementation
