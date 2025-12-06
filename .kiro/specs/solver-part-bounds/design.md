# Solver Part Bounds Design

## Overview

Add `const PARTS: u8` to `Solver` trait and a sealed `SolverExt` trait providing range-validated solving. `DynSolver` always uses the checked version.

## Components

### Solver Trait (updated)

```rust
pub trait Solver {
    const PARTS: u8;
    type SharedData: ToOwned;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError>;
    fn solve_part(shared: &mut Cow<'_, Self::SharedData>, part: u8) -> Result<String, SolveError>;
}
```

### SolverExt Trait (new)

Sealed extension providing range validation. Cannot be overridden.

```rust
pub trait SolverExt: Solver {
    fn solve_part_checked_range(
        shared: &mut Cow<'_, Self::SharedData>,
        part: u8,
    ) -> Result<String, SolveError> {
        if (1..=Self::PARTS).contains(&part) {
            Self::solve_part(shared, part)
        } else {
            Err(SolveError::PartOutOfRange(part))
        }
    }
}

impl<T: Solver + ?Sized> SolverExt for T {}
```

### DynSolver Trait (updated)

```rust
pub trait DynSolver {
    fn solve(&mut self, part: u8) -> Result<String, SolveError>;
    fn parts(&self) -> u8;
    fn year(&self) -> u16;
    fn day(&self) -> u8;
}

impl<'a, S: SolverExt> DynSolver for SolverInstanceCow<'a, S> {
    fn solve(&mut self, part: u8) -> Result<String, SolveError> {
        S::solve_part_checked_range(&mut self.shared, part)
    }

    fn parts(&self) -> u8 {
        S::PARTS
    }

    fn year(&self) -> u16 { self.year }
    fn day(&self) -> u8 { self.day }
}
```

## Error Handling

| Condition | Error |
|-----------|-------|
| part == 0 or part > PARTS | `SolveError::PartOutOfRange(part)` |
| Part in range but not implemented | `SolveError::PartNotImplemented(part)` |

## Correctness Properties

*Properties that should hold across all valid executions.*

### Property 1: Out-of-range rejection
*For any* solver with PARTS = N, calling `solve(part)` where part = 0 OR part > N returns `PartOutOfRange(part)`.
**Validates: Requirements 2.1, 2.2**

### Property 2: Valid range delegation
*For any* solver with PARTS = N and part where 1 <= part <= N, `solve(part)` delegates to `solve_part(part)`.
**Validates: Requirements 2.3**

## Testing Strategy

Use `proptest` with 10 iterations per property test. Format: `**Feature: solver-part-bounds, Property N: description**`
