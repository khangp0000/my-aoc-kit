# aoc-solver

A flexible and type-safe Rust framework for solving Advent of Code problems across multiple years and days.

## Features

- **Type-safe solver interface**: Each solver defines its own shared data type
- **Flexible part dependencies**: Parts can share data through mutations to shared state
- **Builder pattern**: Fluent API for registry construction with compile-time immutability guarantees
- **Plugin system**: Automatic solver discovery and registration using the `inventory` crate
- **Derive macro**: Zero-boilerplate solver registration with `#[derive(AutoRegisterSolver)]`
- **Flexible filtering**: Register solvers by tags, year, or custom predicates
- **Extensible**: Add new solvers without modifying the core library

## Quick Start

### 1. Define a Solver

```rust
use std::borrow::Cow;
use aoc_solver::{Solver, ParseError, SolveError};

pub struct Day1Solver;

impl Solver for Day1Solver {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
    
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError> {
        match part {
            1 => Ok(shared.iter().sum::<i32>().to_string()),
            2 => Ok(shared.iter().product::<i32>().to_string()),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

### 2. Register and Use (Builder Pattern)

```rust
use aoc_solver::{RegistryBuilder, register_solver};

fn main() {
    // Use the builder pattern for registry construction
    let mut builder = RegistryBuilder::new();
    register_solver!(builder, Day1Solver, 2023, 1);
    let registry = builder.build();  // Registry is now immutable
    
    let input = "1\n2\n3\n4\n5";
    let mut solver = registry.create_solver(2023, 1, input).unwrap();
    
    // Solve parts
    match solver.solve(1) {
        Ok(answer) => println!("Part 1: {}", answer),  // Part 1: 15
        Err(e) => eprintln!("Error: {}", e),
    }
    
    match solver.solve(2) {
        Ok(answer) => println!("Part 2: {}", answer),  // Part 2: 120
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

## Simplified Solver Implementation with `#[aoc_solver]`

The `#[aoc_solver]` attribute macro dramatically simplifies solver implementation by automatically generating the `Solver` trait implementation. Instead of manually implementing the trait with match statements, you just define types and part functions.

### Basic Usage

```rust
use aoc_solver::{ParseError, SolveError};
use aoc_solver_macros::aoc_solver;

struct Day1;  // Define the struct

#[aoc_solver(max_parts = 2)]
impl Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".into())))
            .collect()
    }
    
    fn part1(shared: &mut Vec<i32>) -> String {
        shared.iter().sum::<i32>().to_string()
    }
    
    fn part2(shared: &mut Vec<i32>) -> String {
        shared.iter().product::<i32>().to_string()
    }
}
```

The macro generates:
- The complete `Solver` trait implementation
- Proper return value wrapping
- Error handling for out-of-range parts

### Flexible Return Types

Part functions support two return types:

```rust
struct Day2;

#[aoc_solver(max_parts = 2)]
impl Day2 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    
    // Simple string return
    fn part1(shared: &mut Vec<i32>) -> String {
        "42".to_string()
    }
    
    // Result for error handling
    fn part2(shared: &mut Vec<i32>) -> Result<String, SolveError> {
        if shared.is_empty() {
            Err(SolveError::SolveFailed("Empty input".into()))
        } else {
            Ok("answer".to_string())
        }
    }
}
```

### Dependent Parts

For parts that share data, use mutable access to SharedData:

```rust
#[derive(Debug)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
    count: Option<usize>,
}

struct Day3;

#[aoc_solver(max_parts = 2)]
impl Day3 {
    type SharedData = SharedData;
    
    fn parse(input: &str) -> Result<SharedData, ParseError> {
        let numbers: Vec<i32> = input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".into())))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(SharedData {
            numbers,
            sum: None,
            count: None,
        })
    }
    
    // Part 1 stores data for Part 2
    fn part1(shared: &mut SharedData) -> String {
        let sum: i32 = shared.numbers.iter().sum();
        let count = shared.numbers.len();
        
        // Store for Part 2
        shared.sum = Some(sum);
        shared.count = Some(count);
        
        sum.to_string()
    }
    
    // Part 2 uses Part 1's data if available
    fn part2(shared: &mut SharedData) -> String {
        let (sum, count) = if let (Some(s), Some(c)) = (shared.sum, shared.count) {
            (s, c)  // Use Part 1's data
        } else {
            // Compute independently if Part 1 wasn't run
            (shared.numbers.iter().sum(), shared.numbers.len())
        };
        
        let avg = if count > 0 {
            sum as f64 / count as f64
        } else {
            0.0
        };
        format!("{:.2}", avg)
    }
}
```

### Combining with Plugin System

To use `#[aoc_solver]` with automatic registration, combine with `AutoRegisterSolver`:

```rust
use aoc_solver::AutoRegisterSolver;
use aoc_solver_macros::aoc_solver;

#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["example"])]
struct Day1;

#[aoc_solver(max_parts = 2)]
impl Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    fn part1(shared: &mut Vec<i32>) -> String { /* ... */ }
    fn part2(shared: &mut Vec<i32>) -> String { /* ... */ }
}

// Now it can be discovered automatically
let registry = RegistryBuilder::new()
    .register_all_plugins()
    .unwrap()
    .build();
```

### Compile-Time Validation

The macro provides helpful compile-time errors:

```rust
// Missing max_parts
#[aoc_solver]  // Error: missing required attribute 'max_parts'
impl Day1 { /* ... */ }

// Missing required type
#[aoc_solver(max_parts = 2)]
impl Day1 {
    // Error: missing required type 'SharedData'
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
}

// Missing part1
#[aoc_solver(max_parts = 2)]
impl Day1 {
    type SharedData = Vec<i32>;
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    // Error: at least 'part1' function is required
    fn part2(shared: &mut Vec<i32>) -> String { /* ... */ }
}

// Part exceeds max_parts
#[aoc_solver(max_parts = 2)]
impl Day1 {
    type SharedData = Vec<i32>;
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    fn part1(shared: &mut Vec<i32>) -> String { /* ... */ }
    fn part2(shared: &mut Vec<i32>) -> String { /* ... */ }
    fn part3(shared: &mut Vec<i32>) -> String { /* ... */ }  // Error: part3 exceeds max_parts = 2
}
```

## Plugin System (Automatic Registration)

The library supports automatic solver discovery using the `inventory` crate. This eliminates manual registration boilerplate and enables flexible filtering.

### Using the Derive Macro (Recommended)

The easiest way to register a solver is using the `#[derive(AutoRegisterSolver)]` macro:

```rust
use std::borrow::Cow;
use aoc_solver::{AutoRegisterSolver, Solver, ParseError, SolveError};

#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["easy", "2023"])]
pub struct Day1Solver;

impl Solver for Day1Solver {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
    
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError> {
        match part {
            1 => Ok(shared.iter().sum::<i32>().to_string()),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}

// That's it! No manual registration needed.
```

### Using Plugins

```rust
use aoc_solver::RegistryBuilder;

fn main() {
    // Register ALL discovered plugins
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .unwrap()
        .build();
    
    // Or register only specific plugins by tag
    let registry = RegistryBuilder::new()
        .register_solver_plugins(|plugin| plugin.tags.contains(&"easy"))
        .unwrap()
        .build();
    
    // Or register only 2023 solvers
    let registry = RegistryBuilder::new()
        .register_solver_plugins(|plugin| plugin.year == 2023)
        .unwrap()
        .build();
}
```

## Advanced: Dependent Parts

For problems where Part 2 depends on Part 1's computation, use mutable access to shared data via `Cow::to_mut()`:

```rust
use std::borrow::Cow;
use aoc_solver::{Solver, ParseError, SolveError};

#[derive(Debug, Clone)]
pub struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
    count: Option<usize>,
}

pub struct Day5Solver;

impl Solver for Day5Solver {
    type SharedData = SharedData;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        let numbers: Vec<i32> = input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(Cow::Owned(SharedData {
            numbers,
            sum: None,
            count: None,
        }))
    }
    
    fn solve_part(
        shared: &mut Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, SolveError> {
        match part {
            1 => {
                // Need to mutate, so call to_mut() to get owned data
                let data = shared.to_mut();
                let sum: i32 = data.numbers.iter().sum();
                let count = data.numbers.len();
                
                // Store for Part 2
                data.sum = Some(sum);
                data.count = Some(count);
                
                Ok(sum.to_string())
            }
            2 => {
                // Read-only access - no need to call to_mut()
                let (sum, count) = if let (Some(s), Some(c)) = (shared.sum, shared.count) {
                    (s, c)  // Use Part 1's data
                } else {
                    // Compute independently
                    (shared.numbers.iter().sum(), shared.numbers.len())
                };
                
                let average = if count > 0 {
                    sum as f64 / count as f64
                } else {
                    0.0
                };
                Ok(format!("{:.2}", average))
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

## Key Concepts

### Solver Trait

The `Solver` trait is the core interface that all problem solvers implement:

- `SharedData`: The data structure holding parsed input and intermediate results (must implement `ToOwned`)
- `parse()`: Transforms raw input into `Cow<SharedData>` for zero-copy support
- `solve_part()`: Solves a specific part with mutable access to `Cow<SharedData>`

### DynSolver Trait

The `DynSolver` trait provides a type-erased interface for working with any solver:

- `solve(part)`: Computes the solution for a specific part
- `year()` and `day()`: Get the solver's year and day

### Zero-Copy Design

The library uses `Cow<SharedData>` to enable zero-copy parsing:
- Read-only operations work directly with borrowed data
- Call `.to_mut()` only when mutation is needed (triggers clone if borrowed)
- Each solver controls its own memory strategy

### RegistryBuilder and SolverRegistry

The builder pattern ensures type-safe registry construction:

**RegistryBuilder** (mutable, for construction):
- `new()`: Create a new builder
- `register()`: Add a solver factory (returns `Result<Self, RegistrationError>`)
- `register_all_plugins()`: Register all discovered plugins
- `register_solver_plugins(filter)`: Register plugins matching a predicate
- `build()`: Finalize into an immutable `SolverRegistry`

**SolverRegistry** (immutable, for usage):
- `create_solver()`: Create a solver instance with input
- Cannot be modified after construction (compile-time guarantee)

## Examples

The `examples/` directory contains complete working examples:

```bash
# Run the independent parts example
cargo run --example independent_parts

# Run the dependent parts example
cargo run --example dependent_parts

# Run the plugin system example
cargo run --example plugin_system

# Run tests for the examples
cargo test --examples
```

- `independent_parts.rs`: Independent parts (sum and product)
- `dependent_parts.rs`: Dependent parts (sum/count and average)
- `plugin_system.rs`: Plugin system with automatic registration and filtering

## Error Handling

The library uses `Result` types for all fallible operations:

- `ParseError`: Input parsing failures
  - `InvalidFormat`: Input doesn't match expected structure
  - `MissingData`: Required data is absent
  - `Other`: Other parsing issues
  
- `SolveError`: Part solving failures
  - `PartNotImplemented(usize)`: The requested part is not implemented
  - `SolveFailed(String)`: Custom error from solver logic
  
- `SolverError`: Registry operations
  - `NotFound(year, day)`: Solver not found for the given year-day
  - `ParseError`: Wraps parsing errors
  - `SolveError`: Wraps solving errors

- `RegistrationError`: Builder operations
  - `DuplicateSolver(year, day)`: Attempted to register duplicate solver

All errors implement `std::error::Error` for easy integration.

## License

MIT
