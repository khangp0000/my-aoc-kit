# my-aoc-kit

A personal Rust workspace containing libraries and tools for solving Advent of Code problems.

> *"Why write a simple script when you can build an enterprise-grade framework with 5 crates, 2 derive macros, a plugin system, and zero-copy semantics? Because spending 3 weeks on tooling to save 5 minutes per puzzle is peak software engineering."*

## Crates

### [aoc-solver](./aoc-solver/)
A flexible and type-safe framework for solving Advent of Code problems across multiple years and days.

**Features:**
- Trait-based design with `AocParser` and `PartSolver<N>` for clean separation of concerns
- Derive macros: `#[derive(AocSolver)]` and `#[derive(AutoRegisterSolver)]`
- Plugin system with automatic solver discovery via `inventory`
- Zero-copy support with `Cow<SharedData>`
- Builder pattern for registry construction

### [aoc-solver-macros](./aoc-solver-macros/)
Procedural macros for the aoc-solver library:
- `#[derive(AocSolver)]` - Generates `Solver` trait from `AocParser` + `PartSolver<N>`
- `#[derive(AutoRegisterSolver)]` - Automatic plugin registration with year/day/tags

### [aoc-http-client](./aoc-http-client/)
HTTP client for interacting with the Advent of Code website.

**Features:**
- Session validation
- Puzzle input fetching
- Answer submission with detailed feedback
- Secure TLS using rustls

### [aoc-cli](./aoc-cli/)
Command-line interface for running solvers.

> *"Because copy-pasting from a browser is for people who value their time."*

**Features:**
- Run solvers by year/day/part with filtering
- Automatic input fetching and caching
- Tag-based solver filtering
- Ordered result output

### [aoc-solutions](./aoc-solutions/)
Actual puzzle solutions with automatic registration via the plugin system.

## Quick Start

```rust
use std::borrow::Cow;
use aoc_solver::{
    AocParser, AocSolver, AutoRegisterSolver,
    ParseError, PartSolver, SolveError,
};

// Define a solver with automatic registration
#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2024, day = 1, tags = ["easy"])]
pub struct Day1;

impl AocParser for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input.lines()
            .map(|line| line.parse().map_err(|_| 
                ParseError::InvalidFormat("Expected integer".to_string())))
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
}

impl PartSolver<1> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for Day1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}
```

### Using the Registry

```rust
use aoc_solver::{SolverRegistryBuilder, SolverInstanceCow};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Auto-discover all registered solvers
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()?
        .build();
    
    let input = "1\n2\n3\n4\n5";
    let mut solver = registry.create_solver(2024, 1, input)?;
    
    println!("Part 1: {}", solver.solve(1)?); // 15
    println!("Part 2: {}", solver.solve(2)?); // 120
    
    Ok(())
}
```

### Fetching Input from AOC

```rust
use aoc_http_client::AocClient;

let client = AocClient::new()?;
let session = std::env::var("AOC_SESSION")?;
let input = client.get_input(2024, 1, &session)?;

// Submit answer
let result = client.submit_answer(2024, 1, 1, "42", &session)?;
```

## Examples

```bash
# Solver examples
cargo run -p aoc-solver --example independent_parts
cargo run -p aoc-solver --example dependent_parts
cargo run -p aoc-solver --example plugin_system
cargo run -p aoc-solver --example macro_usage

# HTTP client example
export AOC_SESSION="your_session_cookie"
cargo run -p aoc-http-client --example basic_usage
```

## Documentation

For detailed documentation, see each crate's README:
- [aoc-solver README](./aoc-solver/README.md)
- [aoc-http-client README](./aoc-http-client/README.md)

## License

MIT
