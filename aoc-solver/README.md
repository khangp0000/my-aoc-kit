# aoc-solver

A flexible and type-safe Rust framework for solving Advent of Code problems across multiple years and days.

## Features

- **Type-safe solver interface**: Each solver defines its own shared data type
- **Trait-based design**: Separate `AocParser` and `PartSolver<N>` traits for clean separation of concerns
- **Compile-time part validation**: Const generics ensure all required parts are implemented
- **Flexible part dependencies**: Parts can share data through mutations to shared state
- **Builder pattern**: Fluent API for registry construction with compile-time immutability guarantees
- **Plugin system**: Automatic solver discovery and registration using the `inventory` crate
- **Derive macros**: Zero-boilerplate with `#[derive(AocSolver)]` and `#[derive(AutoRegisterSolver)]`
- **Flexible data ownership**: Generic associated type `SharedData<'a>` allows any ownership strategy (owned, borrowed)
- **Built-in timing**: Automatic parse and solve timing capture with `chrono::DateTime<Utc>` timestamps

## Quick Start

### Using the Derive Macro (Recommended)

The easiest way to create a solver is using `AocParser`, `PartSolver<N>`, and `#[derive(AocSolver)]`:

```rust
use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError};

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day1;

impl AocParser for Day1 {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.parse()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect()
    }
}

impl PartSolver<1> for Day1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for Day1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}
```

### Register and Use

```rust
use aoc_solver::{SolverRegistryBuilder, Solver, SolverInstance};

fn main() {
    let mut builder = SolverRegistryBuilder::new();
    builder.register(2023, 1, 2, |input: &str| {
        Ok(Box::new(SolverInstance::<Day1>::new(2023, 1, input)?))
    })
    .unwrap();
    let registry = builder.build();

    let input = "1\n2\n3\n4\n5";
    let mut solver = registry.create_solver(2023, 1, input).unwrap();

    println!("Part 1: {}", solver.solve(1).unwrap().answer); // Part 1: 15
    println!("Part 2: {}", solver.solve(2).unwrap().answer); // Part 2: 120
}
```

### Accessing Timing Information

The framework automatically captures parse and solve timing:

```rust
use aoc_solver::{SolverRegistryBuilder, DynSolver};

fn main() {
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .unwrap()
        .build();

    let input = "1\n2\n3\n4\n5";
    let mut solver = registry.create_solver(2023, 1, input).unwrap();

    // Parse timing is captured during solver creation
    println!("Parse duration: {:?}", solver.parse_duration());

    // Solve timing is captured per-part
    let result = solver.solve(1).unwrap();
    println!("Part 1: {} (solved in {:?})", result.answer, result.duration());
}
```

The `SolveResult` struct provides:
- `answer`: The computed answer as a `String`
- `solve_start` / `solve_end`: UTC timestamps for solve timing
- `duration()`: Convenience method returning `TimeDelta`

The `DynSolver` trait provides:
- `parse_start()` / `parse_end()`: UTC timestamps for parse timing
- `parse_duration()`: Convenience method returning `TimeDelta`

## Trait-Based Design

### AocParser Trait

Defines the shared data type and parsing logic:

```rust
pub trait AocParser {
    type SharedData<'a>;
    fn parse<'a>(input: &'a str) -> Result<Self::SharedData<'a>, ParseError>;
}
```

### PartSolver<N> Trait

Defines the solving logic for each part using const generics:

```rust
pub trait PartSolver<const N: u8>: AocParser {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError>;
}
```

### AocSolver Derive Macro

Generates the `Solver` trait implementation from `AocParser` + `PartSolver<N>`:

```rust
#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]  // Requires PartSolver<1> and PartSolver<2>
struct Day1;
```

**Compile-time checks:**
- If `PartSolver<1>` is not implemented, compilation fails with a clear error
- If `PartSolver<2>` is not implemented but `max_parts = 2`, compilation fails

## Dependent Parts

For problems where Part 2 depends on Part 1's computation:

```rust
#[derive(Debug)]
struct SharedData {
    numbers: Vec<i32>,
    sum: Option<i32>,
    count: Option<usize>,
}

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct Day5;

impl AocParser for Day5 {
    type SharedData<'a> = SharedData;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        let numbers: Vec<i32> = input
            .lines()
            .map(|line| line.parse().map_err(|_| ParseError::InvalidFormat("bad".into())))
            .collect::<Result<_, _>>()?;
        Ok(SharedData { numbers, sum: None, count: None })
    }
}

impl PartSolver<1> for Day5 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        let sum: i32 = shared.numbers.iter().sum();
        shared.sum = Some(sum);
        shared.count = Some(shared.numbers.len());
        Ok(sum.to_string())
    }
}

impl PartSolver<2> for Day5 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        // Use cached value if available (from Part 1)
        let sum = shared.sum.unwrap_or_else(|| shared.numbers.iter().sum());
        let count = shared.count.unwrap_or_else(|| shared.numbers.len());
        let avg = if count > 0 { sum as f64 / count as f64 } else { 0.0 };
        Ok(format!("{:.2}", avg))
    }
}
```

## Plugin System (Automatic Registration)

Combine `#[derive(AocSolver)]` with `#[derive(AutoRegisterSolver)]` for automatic discovery:

```rust
use aoc_solver::{AocParser, AocSolver, AutoRegisterSolver, ParseError, PartSolver, SolveError};

#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 1, tags = ["easy"])]
struct Day1;

impl AocParser for Day1 { /* ... */ }
impl PartSolver<1> for Day1 { /* ... */ }
impl PartSolver<2> for Day1 { /* ... */ }

// Discover and use all registered solvers
fn main() {
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .unwrap()
        .build();

    let mut solver = registry.create_solver(2023, 1, "1\n2\n3").unwrap();
    println!("Part 1: {}", solver.solve(1).unwrap());
}
```

### Filtering Plugins

```rust
// Register only solvers with specific tags
let registry = SolverRegistryBuilder::new()
    .register_solver_plugins(|plugin| plugin.tags.contains(&"easy"))
    .unwrap()
    .build();

// Register only 2023 solvers
let registry = SolverRegistryBuilder::new()
    .register_solver_plugins(|plugin| plugin.year == 2023)
    .unwrap()
    .build();
```

## Manual Solver Implementation

You can also implement the `Solver` trait directly without macros:

```rust
use aoc_solver::{AocParser, Solver, ParseError, SolveError};

struct Day1;

impl AocParser for Day1 {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| ParseError::InvalidFormat("bad".into())))
            .collect()
    }
}

impl Solver for Day1 {
    const PARTS: u8 = 2;

    fn solve_part(shared: &mut Self::SharedData<'_>, part: u8) -> Result<String, SolveError> {
        match part {
            1 => Ok(shared.iter().sum::<i32>().to_string()),
            2 => Ok(shared.iter().product::<i32>().to_string()),
            _ => Err(SolveError::PartNotImplemented(part)),
        }
    }
}
```

## Zero-Copy Parsing

For inputs that don't need transformation, use `&'a str` for true zero-copy:

```rust
use aoc_solver::{AocParser, AocSolver, ParseError, PartSolver, SolveError};

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct ZeroCopyExample;

impl AocParser for ZeroCopyExample {
    // No allocation - just borrow the input directly!
    type SharedData<'a> = &'a str;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        Ok(input)
    }
}

impl PartSolver<1> for ZeroCopyExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.lines().count().to_string())
    }
}

impl PartSolver<2> for ZeroCopyExample {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.len().to_string())
    }
}
```

## Key Concepts

### Flexible Data Ownership

The library uses a generic associated type `SharedData<'a>` for maximum flexibility:
- Use owned types like `Vec<T>` or custom structs when you need to store and mutate data (simplest approach)
- Use `&'a str` or `&'a [T]` for true zero-copy borrowed data when no transformation is needed

### SolverRegistryBuilder and SolverRegistry

**SolverRegistryBuilder** (mutable, for construction):
- `register()`: Add a solver factory with year, day, parts count, and factory function
- `register_all_plugins()`: Register all discovered plugins
- `register_solver_plugins(filter)`: Register plugins matching a predicate
- `build()`: Finalize into an immutable `SolverRegistry`

**SolverRegistry** (immutable, for usage):
- `storage()`: Access the internal `SolverRegistryStorage` for iteration/lookup
- `create_solver()`: Create a solver instance with input

**SolverRegistryStorage** (internal storage):
- `iter_info()`: Iterate over registered solver metadata in (year, day) order
- `get_info()`: Get metadata for a specific year/day
- `contains()`: Check if a solver is registered
- `len()`, `is_empty()`: Count registered solvers

## Examples

```bash
cargo run --example independent_parts
cargo run --example dependent_parts
cargo run --example macro_usage
cargo run --example plugin_system
```

## Error Handling

- `ParseError`: Input parsing failures
- `SolveError`: Part solving failures (`PartNotImplemented`, `PartOutOfRange`, `SolveFailed`)
- `SolverError`: Registry operations (`NotFound`, wraps parse/solve errors)
- `RegistrationError`: Duplicate solver registration

## License

MIT
