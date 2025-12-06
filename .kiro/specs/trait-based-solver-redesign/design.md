# Trait-Based Solver Redesign

## Overview

This document describes a redesign of the AOC solver system using separate traits for parsing and solving, with const generics for type-safe part numbers. The design provides:

- Zero-copy parsing support via `Cow<SharedData>`
- Compile-time part number validation
- Clean separation between parsing, solving, and registration
- Simpler derive macros with single responsibilities

## Core Traits

### AocParser Trait

Defines the shared data type and parsing logic for a solver.

```rust
use std::borrow::Cow;
use aoc_solver::ParseError;

/// Trait for parsing AOC puzzle input into shared data
pub trait AocParser {
    /// The shared data structure that holds parsed input and intermediate results.
    /// Must implement ToOwned + Clone to support zero-copy via Cow.
    type SharedData: ToOwned + Clone;
    
    /// Parse the input string into the shared data structure.
    ///
    /// Returns `Cow::Owned` for transformed data, or `Cow::Borrowed` for zero-copy.
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError>;
}
```

**Key points:**
- `SharedData` must be `ToOwned + Clone` to work with `Cow`
- Returns `Cow<'_, SharedData>` enabling zero-copy when possible
- Separate from solving logic - single responsibility

### PartSolver Trait

Defines the solving logic for a specific part using const generics.

```rust
use std::borrow::Cow;
use aoc_solver::SolveError;

/// Trait for solving a specific part of an AOC puzzle.
/// 
/// The const generic `N` represents the part number (1, 2, etc.).
/// This provides compile-time validation that the part is implemented.
pub trait PartSolver<const N: usize>: AocParser {
    /// Solve this part of the puzzle.
    ///
    /// # Arguments
    /// * `shared` - Mutable reference to Cow containing shared data.
    ///   - For read-only operations: just read from `shared` (zero-copy)
    ///   - For mutations: call `shared.to_mut()` to get owned data (triggers clone if borrowed)
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError)` - An error occurred while solving
    fn solve(shared: &mut Cow<'_, Self::SharedData>) -> Result<String, SolveError>;
}
```

**Key points:**
- `const N: usize` provides compile-time part number
- Requires `AocParser` as supertrait (access to `SharedData` type)
- User controls cloning via `shared.to_mut()` when mutation needed
- Read-only operations work directly with borrowed data (zero-copy)

### Solver Trait

The core trait that extends `AocParser` and adds solving capabilities.

```rust
use std::borrow::Cow;
use aoc_solver::{ParseError, SolveError};

/// Core trait that all Advent of Code solvers must implement.
/// Extends AocParser to inherit SharedData and parse().
pub trait Solver: AocParser {
    /// Number of parts this solver implements
    const PARTS: u8;
    
    /// Solve a specific part of the problem
    ///
    /// # Arguments
    /// * `shared` - Mutable reference to Cow containing shared data
    /// * `part` - The part number (1, 2, etc.)
    ///
    /// # Returns
    /// * `Ok(String)` - The answer for this part
    /// * `Err(SolveError)` - An error occurred while solving
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: u8,
    ) -> Result<String, SolveError>;
}
```

**Key points:**
- `Solver: AocParser` - inherits `SharedData` type and `parse()` function
- Only defines `PARTS` constant and `solve_part()` method
- No duplication of parsing logic

## Derive Macros

### AocSolver Derive Macro

Generates the `Solver` trait implementation from `AocParser` + `PartSolver<N>` impls.

Since `Solver: AocParser`, the macro only needs to generate `PARTS` and `solve_part()` - the `SharedData` type and `parse()` function are inherited from `AocParser`.

**Attribute:** `#[aoc_solver(max_parts = N)]` - specifies how many parts to dispatch

```rust
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day1;
```

**Generates:**

```rust
impl Solver for Day1 {
    const PARTS: u8 = 2;
    
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: u8,
    ) -> Result<String, SolveError> {
        match part {
            1 => <Self as PartSolver<1>>::solve(shared),
            2 => <Self as PartSolver<2>>::solve(shared),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

**Compile-time checks:**
- If `AocParser` is not implemented, compilation fails (required supertrait)
- If `PartSolver<1>` is not implemented, compilation fails with clear error
- If `PartSolver<2>` is not implemented but `#[aoc_solver(max_parts = 2)]` specified, compilation fails
- All `PartSolver<M>` for M in 1..=N must be implemented

### AutoRegisterSolver Derive Macro (Existing)

Handles plugin registration with the inventory system. Unchanged from current design.

**Attributes:** `#[aoc(year = N, day = N, tags = [...])]`

```rust
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["easy"])]
struct Day1;
```

**Generates:**

```rust
inventory::submit! {
    SolverPlugin {
        year: 2023,
        day: 1,
        solver: &Day1,
        tags: &["easy"],
    }
}
```

## Usage Examples

### Independent Parts (Read-Only, Zero-Copy)

```rust
use std::borrow::Cow;
use aoc_solver::{AocParser, PartSolver, ParseError, SolveError};
use aoc_solver_macros::{AocSolver, AutoRegisterSolver};

#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 1)]
struct Day1;

impl AocParser for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
            .collect::<Result<_, _>>()?;
        Ok(Cow::Owned(numbers))
    }
}

impl PartSolver<1> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        // Read-only - no clone happens
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        // Read-only - no clone happens
        Ok(shared.iter().product::<i32>().to_string())
    }
}
```

### Dependent Parts (Part 1 Caches for Part 2)

```rust
use std::borrow::Cow;
use aoc_solver::{AocParser, PartSolver, ParseError, SolveError};
use aoc_solver_macros::{AocSolver, AutoRegisterSolver};

#[derive(Clone)]
struct Day5Data {
    numbers: Vec<i32>,
    part1_sum: Option<i32>,  // Cached by Part 1
}

#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 5)]
struct Day5;

impl AocParser for Day5 {
    type SharedData = Day5Data;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|l| l.parse().map_err(|_| ParseError::InvalidFormat("bad int".into())))
            .collect::<Result<_, _>>()?;
        Ok(Cow::Owned(Day5Data {
            numbers,
            part1_sum: None,
        }))
    }
}

impl PartSolver<1> for Day5 {
    fn solve(shared: &mut Cow<'_, Day5Data>) -> Result<String, SolveError> {
        // Need to mutate - call to_mut() to get owned data
        let data = shared.to_mut();
        let sum: i32 = data.numbers.iter().sum();
        data.part1_sum = Some(sum);  // Cache for Part 2
        Ok(sum.to_string())
    }
}

impl PartSolver<2> for Day5 {
    fn solve(shared: &mut Cow<'_, Day5Data>) -> Result<String, SolveError> {
        // Read-only - use cached value if available
        let sum = shared.part1_sum.unwrap_or_else(|| shared.numbers.iter().sum());
        Ok((sum * 2).to_string())
    }
}
```

### Manual Registration (Without AutoRegisterSolver)

```rust
use aoc_solver::{RegistryBuilder, SolverInstanceCow};

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day1;

// ... AocParser and PartSolver impls ...

fn main() {
    let registry = RegistryBuilder::new()
        .register(2023, 1, |input| {
            let shared = Day1::parse(input)?;
            Ok(Box::new(SolverInstanceCow::<Day1>::new(2023, 1, shared)))
        })
        .unwrap()
        .build();
    
    let mut solver = registry.create_solver(2023, 1, "1\n2\n3").unwrap();
    println!("Part 1: {}", solver.solve(1).unwrap());
}
```

## Design Benefits

| Aspect | Benefit |
|--------|---------|
| **Type Safety** | `PartSolver<3>` is a compile-time check - missing impl = compile error |
| **Zero-Copy** | User controls cloning via `.to_mut()` - read-only parts never clone |
| **Single Responsibility** | `AocParser` parses, `PartSolver<N>` solves, `AutoRegisterSolver` registers |
| **Composable** | Use macros independently or together |
| **Minimal Attributes** | `#[aoc_solver(max_parts = N)]` for solver, `#[aoc(...)]` for registration |
| **Clear Semantics** | Each trait has one job, easy to understand |

## Migration from Current Design

### Before (Current)

```rust
impl Solver for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        // ...
    }
    
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError> {
        match part {
            1 => { /* ... */ }
            2 => { /* ... */ }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

### After (New Design)

```rust
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day1;

impl AocParser for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        // Same as before
    }
}

impl PartSolver<1> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        // Part 1 logic extracted
    }
}

impl PartSolver<2> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        // Part 2 logic extracted
    }
}
```

## Migration Guide

Since `Solver` now extends `AocParser`, users implementing `Solver` directly must split their implementation:

### Before (Old Design)

```rust
impl Solver for Day1 {
    type SharedData = Vec<i32>;
    const PARTS: u8 = 2;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        // parsing logic
    }
    
    fn solve_part(shared: &mut Cow<'_, Self::SharedData>, part: u8) -> Result<String, SolveError> {
        // solving logic
    }
}
```

### After (New Design)

```rust
// Step 1: Move SharedData and parse() to AocParser
impl AocParser for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        // same parsing logic
    }
}

// Step 2: Solver only has PARTS and solve_part()
impl Solver for Day1 {
    const PARTS: u8 = 2;
    
    fn solve_part(shared: &mut Cow<'_, Self::SharedData>, part: u8) -> Result<String, SolveError> {
        // same solving logic
    }
}
```

The migration is mechanical - just moving code between impl blocks. The `solve_part()` logic remains unchanged.

## Implementation Notes

### Trait Hierarchy

```
AocParser (defines SharedData + parse)
    ↑
    ├── PartSolver<1> (requires AocParser)
    ├── PartSolver<2> (requires AocParser)
    ├── ...
    │
    └── Solver: AocParser (adds PARTS + solve_part, inherits SharedData + parse)
            ↑
        #[derive(AocSolver)] generates Solver impl
```

The key insight is that `Solver` extends `AocParser` as a supertrait. This means:
- `Solver` inherits `SharedData` type and `parse()` from `AocParser`
- `Solver` only defines `PARTS` constant and `solve_part()` method
- The `AocSolver` macro only generates `PARTS` and `solve_part()`, not redundant delegations

### Const Generic Bounds

Rust allows any `usize` for `const N: usize`. To restrict to valid AOC parts (1-25), we could add a sealed trait pattern, but it's likely unnecessary complexity.

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Part dispatch correctness
*For any* valid part number N in 1..=max_parts, calling `Solver::solve_part(shared, N)` should produce the same result as calling `<Self as PartSolver<N>>::solve(shared)`.
**Validates: Requirements 3.3**

### Property 2: Invalid part rejection
*For any* part number outside the valid range (0 or > max_parts), `Solver::solve_part` should return `SolveError::PartNotImplemented`.
**Validates: Requirements 3.5**

### Property 3: Zero-copy read preservation
*For any* solver where part functions only read from shared data (no `to_mut()` calls), the underlying data should not be cloned during solve operations.
**Validates: Requirements 2.4, 1.2, 1.3**

### Property 4: Clone-on-write mutation
*For any* solver where a part function calls `to_mut()` on borrowed data, the data should be cloned exactly once, and subsequent mutations should work correctly.
**Validates: Requirements 2.5**

### Error Messages

When `PartSolver<N>` is not implemented but referenced in the generated match:

```
error[E0277]: the trait bound `Day1: PartSolver<2>` is not satisfied
  --> src/main.rs:5:10
   |
5  | #[derive(AocSolver)]
   |          ^^^^^^^^^ the trait `PartSolver<2>` is not implemented for `Day1`
   |
   = help: implement `PartSolver<2>` for `Day1`
```

This is a clear, actionable error message.

## Testing Strategy

### Unit Tests
- Test that `AocParser` implementations compile and work correctly
- Test that `PartSolver<N>` implementations can access `SharedData`
- Test that `#[derive(AocSolver)]` generates valid `Solver` implementations
- Test backward compatibility with manual `Solver` implementations
- Test `AutoRegisterSolver` works with the new trait-based approach

### Property-Based Tests
Property-based tests will use the `proptest` crate to verify correctness properties across many inputs.

- **Property 1**: Generate random part numbers 1..=max_parts, verify dispatch correctness
- **Property 2**: Generate random invalid part numbers (0, > max_parts), verify error returned
- **Property 3**: Track clone calls during read-only operations, verify zero clones
- **Property 4**: Track clone calls during mutation, verify exactly one clone on first `to_mut()`

Each property-based test will be configured to run a minimum of 100 iterations and will be tagged with the format: `**Feature: trait-based-solver-redesign, Property N: <property_text>**`
