# Advent of Code Workspace

A Rust workspace containing libraries and tools for solving Advent of Code problems.

## Crates

### [aoc-solver](./aoc-solver/)
A flexible and type-safe framework for solving Advent of Code problems across multiple years and days.

**Features:**
- Type-safe solver interface with custom parsed data types
- Support for both independent and dependent parts
- Plugin system with automatic solver discovery
- Derive macro for zero-boilerplate registration
- Builder pattern for registry construction
- Result caching to avoid redundant computation

### [aoc-http-client](./aoc-http-client/)
HTTP client for interacting with the Advent of Code website.

**Features:**
- Session validation
- Puzzle input fetching
- Answer submission with detailed feedback
- Secure TLS using rustls
- Well-typed error handling

### [aoc-solver-macros](./aoc-solver-macros/)
Procedural macros for the aoc-solver library, providing the `#[derive(AutoRegisterSolver)]` macro for automatic solver registration.

## Quick Start

```rust
use aoc_solver::{AutoRegisterSolver, Solver, ParseError, PartResult, SolveError};
use aoc_http_client::AocClient;

// Define a solver with automatic registration
#[derive(AutoRegisterSolver)]
#[aoc(year = 2024, day = 1)]
pub struct Day1Solver;

impl Solver for Day1Solver {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch input from AOC website
    let client = AocClient::new()?;
    let session = std::env::var("AOC_SESSION")?;
    let input = client.get_input(2024, 1, &session)?;
    
    // Create and run solver
    let registry = aoc_solver::RegistryBuilder::new()
        .register_all_plugins()?
        .build();
    
    let mut solver = registry.create_solver(2024, 1, &input)?;
    
    let answer = solver.solve(1)?;
    println!("Part 1: {}", answer);
    
    // Submit answer
    let result = client.submit_answer(2024, 1, 1, &answer, &session)?;
    println!("Result: {:?}", result);
    
    Ok(())
}
```

## Examples

Each crate contains examples demonstrating its usage:

```bash
# Solver examples
cargo run -p aoc-solver --example independent_parts
cargo run -p aoc-solver --example dependent_parts
cargo run -p aoc-solver --example plugin_system

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
