# aoc-solver

A flexible and type-safe Rust framework for solving Advent of Code problems across multiple years and days.

## Features

- **Type-safe solver interface**: Each solver defines its own parsed data type and partial result type
- **Flexible part dependencies**: Support for both independent and dependent parts
- **Builder pattern**: Fluent API for registry construction with compile-time immutability guarantees
- **Plugin system**: Automatic solver discovery and registration using the `inventory` crate
- **Derive macro**: Zero-boilerplate solver registration with `#[derive(AutoRegisterSolver)]`
- **Flexible filtering**: Register solvers by tags, year, or custom predicates
- **Caching**: Results are cached to avoid redundant computation
- **Extensible**: Add new solvers without modifying the core library

## Quick Start

### 1. Define a Solver

```rust
use aoc_solver::{Solver, ParseError, PartResult, SolveError};

pub struct Day1Solver;

impl Solver for Day1Solver {
    type Parsed = Vec<i32>;
    type PartialResult = ();  // No data shared between parts
    
    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect()
    }
    
    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        _previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => Ok(PartResult {
                answer: parsed.iter().sum::<i32>().to_string(),
                partial: None,
            }),
            2 => Ok(PartResult {
                answer: parsed.iter().product::<i32>().to_string(),
                partial: None,
            }),
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
    
    // Access cached results
    let all_results = solver.results();
    println!("All results: {:?}", all_results);
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
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".into())))
            .collect()
    }
    
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    fn part2(parsed: &Vec<i32>) -> String {
        parsed.iter().product::<i32>().to_string()
    }
}
```

The macro generates:
- The complete `Solver` trait implementation
- Proper return value wrapping
- Error handling for out-of-range parts

### Flexible Return Types

Part functions support multiple return types:

```rust
struct Day2;

#[aoc_solver(max_parts = 3)]
impl Day2 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    
    // Simple string return
    fn part1(parsed: &Vec<i32>) -> String {
        "42".to_string()
    }
    
    // Result for error handling
    fn part2(parsed: &Vec<i32>) -> Result<String, SolveError> {
        Ok("answer".to_string())
    }
    
    // Full control with PartResult
    fn part3(parsed: &Vec<i32>) -> PartResult<()> {
        PartResult {
            answer: "answer".to_string(),
            partial: None,
        }
    }
}
```

### Dependent Parts

For parts that share data, use `PartResult` and add a `prev` parameter:

```rust
use aoc_solver::PartResult;

#[derive(Clone)]
struct SumCount {
    sum: i32,
    count: usize,
}

struct Day3;

#[aoc_solver(max_parts = 2)]
impl Day3 {
    type Parsed = Vec<i32>;
    type PartialResult = SumCount;
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    
    // Part 1 returns data for Part 2
    fn part1(parsed: &Vec<i32>) -> PartResult<SumCount> {
        let sum: i32 = parsed.iter().sum();
        PartResult {
            answer: sum.to_string(),
            partial: Some(SumCount { sum, count: parsed.len() }),
        }
    }
    
    // Part 2 receives Part 1's data
    fn part2(parsed: &Vec<i32>, prev: Option<&SumCount>) -> String {
        if let Some(data) = prev {
            let avg = data.sum as f64 / data.count as f64;
            format!("{:.2}", avg)
        } else {
            // Can still compute independently
            let sum: i32 = parsed.iter().sum();
            let avg = sum as f64 / parsed.len() as f64;
            format!("{:.2}", avg)
        }
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
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    fn part1(parsed: &Vec<i32>) -> String { /* ... */ }
    fn part2(parsed: &Vec<i32>) -> String { /* ... */ }
}

// Now it can be discovered automatically
let registry = RegistryBuilder::new()
    .register_all_plugins()
    .unwrap()
    .build();
```

**Note**: Now that users define the struct separately, you can easily combine `#[derive(AutoRegisterSolver)]` with `#[aoc_solver]`!

### Compile-Time Validation

The macro provides helpful compile-time errors:

```rust
// Missing max_parts
#[aoc_solver]  // Error: missing required attribute 'max_parts'
impl Day1 { /* ... */ }

// Missing required types
#[aoc_solver(max_parts = 2)]
impl Day1 {
    // Error: missing required type 'Parsed'
    // Error: missing required type 'PartialResult'
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
}

// Missing part1
#[aoc_solver(max_parts = 2)]
impl Day1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    // Error: at least 'part1' function is required
    fn part2(parsed: &Vec<i32>) -> String { /* ... */ }
}

// Part exceeds max_parts
#[aoc_solver(max_parts = 2)]
impl Day1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { /* ... */ }
    fn part1(parsed: &Vec<i32>) -> String { /* ... */ }
    fn part2(parsed: &Vec<i32>) -> String { /* ... */ }
    fn part3(parsed: &Vec<i32>) -> String { /* ... */ }  // Error: part3 exceeds max_parts = 2
}
```

## Plugin System (Automatic Registration)

The library supports automatic solver discovery using the `inventory` crate. This eliminates manual registration boilerplate and enables flexible filtering.

### Using the Derive Macro (Recommended)

The easiest way to register a solver is using the `#[derive(AutoRegisterSolver)]` macro:

```rust
use aoc_solver::{AutoRegisterSolver, Solver, ParseError, PartResult, SolveError};

#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["easy", "2023"])]
pub struct Day1Solver;

impl Solver for Day1Solver {
    // ... implementation ...
}

// That's it! No manual registration needed.
```

The derive macro automatically generates the plugin registration code, eliminating boilerplate.

### Manual Plugin Registration

You can also register plugins manually if needed:

```rust
use aoc_solver::{Solver, SolverPlugin, ParseError, PartResult, SolveError};

pub struct Day1Solver;

impl Solver for Day1Solver {
    // ... implementation ...
}

// Manually register the solver with tags
inventory::submit! {
    SolverPlugin {
        year: 2023,
        day: 1,
        solver: &Day1Solver,
        tags: &["easy", "2023"],
    }
}
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
    
    // Or mix manual and plugin registration
    let registry = RegistryBuilder::new()
        .register(2022, 1, |input| { /* custom factory */ })
        .unwrap()
        .register_all_plugins()
        .unwrap()
        .build();
}
```

### Derive Macro vs Manual Registration

**Before (Manual):**
```rust
pub struct Day1Solver;
impl Solver for Day1Solver { /* ... */ }

inventory::submit! {
    SolverPlugin {
        year: 2023,
        day: 1,
        solver: &Day1Solver,
        tags: &["easy"],
    }
}
```

**After (Derive Macro):**
```rust
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1, tags = ["easy"])]
pub struct Day1Solver;

impl Solver for Day1Solver { /* ... */ }
```

The derive macro is cleaner, less error-prone, and keeps metadata close to the struct definition.

### Benefits of the Plugin System

- **No manual registration**: Solvers register themselves automatically
- **Derive macro**: Eliminates boilerplate with `#[derive(AutoRegisterSolver)]`
- **Modular**: Define solvers in separate modules or crates
- **Flexible filtering**: Register subsets based on tags, year, or custom predicates
- **Environment-specific**: Different solver sets for dev, test, and production
- **Scalable**: Handles hundreds of solvers without boilerplate

## Advanced: Dependent Parts

For problems where Part 2 depends on Part 1's computation:

```rust
use aoc_solver::{Solver, ParseError, PartResult, SolveError};

pub struct Day5Solver;

#[derive(Debug, Clone)]
pub struct Part1Data {
    pub sum: i32,
    pub count: usize,
}

impl Solver for Day5Solver {
    type Parsed = Vec<i32>;
    type PartialResult = Part1Data;
    
    fn parse(input: &str) -> Result<Self::Parsed, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect()
    }
    
    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        previous_partial: Option<&Self::PartialResult>,
    ) -> Result<PartResult<Self::PartialResult>, SolveError> {
        match part {
            1 => {
                let sum: i32 = parsed.iter().sum();
                let count = parsed.len();
                Ok(PartResult {
                    answer: sum.to_string(),
                    partial: Some(Part1Data { sum, count }),
                })
            }
            2 => {
                let average = if let Some(data) = previous_partial {
                    // Use Part 1's data
                    data.sum as f64 / data.count as f64
                } else {
                    // Compute independently
                    let sum: i32 = parsed.iter().sum();
                    sum as f64 / parsed.len() as f64
                };
                Ok(PartResult {
                    answer: format!("{:.2}", average),
                    partial: None,
                })
            }
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

## Key Concepts

### Solver Trait

The `Solver` trait is the core interface that all problem solvers implement:

- `Parsed`: The intermediate representation of parsed input
- `PartialResult`: Data that can be shared between parts (use `()` for independent parts)
- `parse()`: Transforms raw input into the parsed representation
- `solve_part()`: Solves a specific part, optionally using data from previous parts

### DynSolver Trait

The `DynSolver` trait provides a type-erased interface for working with any solver:

- `solve(part)`: **Recomputes** the solution and caches it
- `results()`: Returns **cached** results without recomputation
- `year()` and `day()`: Get the solver's year and day

**Important**: Call `solve()` once per part to compute, then use `results()` to access answers without redundant computation.

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

**Benefits**:
- Fluent API with method chaining
- Duplicate detection during registration
- Immutability after construction
- Clear separation between setup and usage phases

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
  - `PartOutOfRange(usize)`: The requested part number exceeds max_parts (used by `#[aoc_solver]` macro)
  - `SolveFailed(Box<dyn Error + Send + Sync>)`: Custom error from solver logic
  
- `SolverError`: Registry operations
  - `NotFound(year, day)`: Solver not found for the given year-day
  - `ParseError`: Wraps parsing errors
  - `SolveError`: Wraps solving errors

- `RegistrationError`: Builder operations
  - `DuplicateSolver(year, day)`: Attempted to register duplicate solver

All errors implement `std::error::Error` for easy integration.

### Custom Error Example

```rust
use aoc_solver::{Solver, SolveError, PartResult};

fn solve_part(
    parsed: &Self::Parsed,
    part: usize,
    _previous_partial: Option<&Self::PartialResult>,
) -> Result<PartResult<Self::PartialResult>, SolveError> {
    match part {
        1 => {
            // Can return custom errors
            let result = compute_complex_answer(parsed)
                .map_err(|e| SolveError::SolveFailed(Box::new(e)))?;
            Ok(PartResult {
                answer: result.to_string(),
                partial: None,
            })
        }
        _ => Err(SolveError::PartNotImplemented(part)),
    }
}
```

## License

MIT
