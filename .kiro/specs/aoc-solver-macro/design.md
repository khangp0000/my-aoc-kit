# Design Document

## Overview

The `#[aoc_solver]` attribute macro simplifies Advent of Code solver implementation by automatically generating the `Solver` trait implementation. Users write an impl block with type definitions, a parse function, and part functions, and the macro generates all the boilerplate code needed to satisfy the `Solver` trait.

The macro provides:
- Automatic trait implementation generation
- Flexible return type handling (String, Result, PartResult)
- Support for both independent and dependent parts
- Compile-time validation with helpful error messages
- Compatibility with existing `AutoRegisterSolver` derive macro

## Architecture

### Macro Processing Pipeline

```
User Code (impl block)
    ↓
Parse with syn
    ↓
Validate structure
    ↓
Extract components (types, functions)
    ↓
Analyze part functions
    ↓
Generate Solver trait impl
    ↓
Generate struct (if needed)
    ↓
Output TokenStream
```

### Key Components

1. **Attribute Parser**: Extracts `max_parts` from `#[aoc_solver(max_parts = N)]`
2. **Impl Block Parser**: Extracts types, parse function, and part functions
3. **Validator**: Checks for required components and validates signatures
4. **Code Generator**: Produces the Solver trait implementation
5. **Error Generator**: Creates helpful compile_error! messages

## Components and Interfaces

### Input Structure

```rust
#[aoc_solver(max_parts = 2)]
impl Day1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Parsed, ParseError> { ... }
    fn part1(parsed: &Parsed) -> String { ... }
    fn part2(parsed: &Parsed) -> Result<String, SolveError> { ... }
}
```

### Output Structure

```rust
// Generated struct (if not exists)
struct Day1;

// Original impl block (preserved)
impl Day1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { ... }
    fn part1(parsed: &Vec<i32>) -> String { ... }
    fn part2(parsed: &Vec<i32>) -> Result<String, SolveError> { ... }
}

// Generated Solver trait impl
impl ::aoc_solver::Solver for Day1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
    fn parse(input: &str) -> Result<Self::Parsed, ::aoc_solver::ParseError> {
        <Day1>::parse(input)
    }
    
    fn solve_part(
        parsed: &Self::Parsed,
        part: usize,
        previous_partial: Option<&Self::PartialResult>,
    ) -> Result<::aoc_solver::PartResult<Self::PartialResult>, ::aoc_solver::SolveError> {
        match part {
            1 => {
                let answer = <Day1>::part1(parsed);
                Ok(::aoc_solver::PartResult { answer, partial: None })
            }
            2 => {
                let answer = <Day1>::part2(parsed)?;
                Ok(::aoc_solver::PartResult { answer, partial: None })
            }
            _ => Err(::aoc_solver::SolveError::PartOutOfRange(part)),
        }
    }
}
```

## Data Models

### Parsed Macro Input

```rust
struct MacroInput {
    max_parts: usize,
    impl_block: ItemImpl,
    struct_name: Ident,
}

struct ExtractedComponents {
    parsed_type: Option<Type>,
    partial_result_type: Option<Type>,
    parse_fn: Option<ImplItemFn>,
    part_fns: Vec<PartFunction>,
}

struct PartFunction {
    number: usize,
    fn_item: ImplItemFn,
    signature: PartSignature,
}

enum PartSignature {
    Independent {
        parsed_param: Type,
        return_type: ReturnType,
    },
    Dependent {
        parsed_param: Type,
        prev_param: Type,
        return_type: ReturnType,
    },
}

enum ReturnType {
    String,
    ResultString,
    PartResult,
    ResultPartResult,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Type forwarding preserves user types
*For any* valid type definitions in the impl block, the generated Solver trait SHALL use those exact types for Parsed and PartialResult
**Validates: Requirements 2.1, 2.2**

### Property 2: String returns are wrapped correctly
*For any* part function returning String, the generated code SHALL wrap it in PartResult with partial: None
**Validates: Requirements 4.1, 6.2**

### Property 3: Result returns are unwrapped and wrapped
*For any* part function returning Result<String, SolveError>, the generated code SHALL unwrap with ? and wrap in PartResult
**Validates: Requirements 4.2, 6.2**

### Property 4: PartResult returns are passed through
*For any* part function returning PartResult<T>, the generated code SHALL use it directly without modification
**Validates: Requirements 4.3**

### Property 5: Result<PartResult> returns are passed through
*For any* part function returning Result<PartResult<T>, SolveError>, the generated code SHALL use it directly without modification
**Validates: Requirements 4.4**

### Property 6: Part validation enforces completeness
*For any* max_parts value N, the macro SHALL verify that all parts from 1 to N exist
**Validates: Requirements 5.1**

### Property 7: Independent parts receive only parsed data
*For any* part function with one parameter, the generated code SHALL call it with only the parsed data
**Validates: Requirements 6.1**

### Property 8: Dependent parts receive previous partial
*For any* part function with two parameters, the generated code SHALL pass the previous_partial parameter
**Validates: Requirements 7.1**

### Property 9: Partial data flows between parts
*For any* part returning PartResult with partial: Some(data), that data SHALL be available to the next part's prev parameter
**Validates: Requirements 7.2**

### Property 10: Struct generation avoids duplicates
*For any* struct name, if the struct exists, the macro SHALL not generate a duplicate declaration
**Validates: Requirements 9.1, 9.2**

### Property 11: Fully qualified paths are used
*For any* generated code, all library types SHALL use fully qualified paths starting with ::aoc_solver::
**Validates: Requirements 10.1**

### Property 12: Valid parts dispatch correctly
*For any* part number from 1 to max_parts, solve_part SHALL call the corresponding part function
**Validates: Requirements 11.1**

### Property 13: Out-of-range parts return error
*For any* part number greater than max_parts, solve_part SHALL return Err(SolveError::PartOutOfRange(part))
**Validates: Requirements 11.2**

## Error Handling

### Compile-Time Errors

The macro generates helpful compile-time errors for common mistakes:

#### Missing max_parts Attribute
```rust
compile_error!("aoc_solver: missing required attribute 'max_parts'. Use: #[aoc_solver(max_parts = N)]");
```

#### Missing Type Definitions
```rust
compile_error!("aoc_solver: missing required type 'Parsed'. Add: type Parsed = YourType;");
compile_error!("aoc_solver: missing required type 'PartialResult'. Add: type PartialResult = YourType; (or () for independent parts)");
```

#### Missing parse Function
```rust
compile_error!("aoc_solver: missing required 'parse' function. Add: fn parse(input: &str) -> Result<Parsed, ParseError> { ... }");
```

#### Missing part1
```rust
compile_error!("aoc_solver: at least 'part1' function is required. Add: fn part1(parsed: &Parsed) -> String { ... }");
```

#### Missing Parts in Range
```rust
compile_error!("aoc_solver: part2 is missing but max_parts = 3. All parts from 1 to max_parts must be implemented.");
```

#### Part Number Exceeds max_parts
```rust
compile_error!("aoc_solver: part4 exceeds max_parts = 2. Remove this part or increase max_parts.");
```

#### Invalid max_parts Value
```rust
compile_error!("aoc_solver: max_parts must be at least 1");
```

#### Unsupported Return Type
```rust
compile_error!("aoc_solver: part1 return type is not supported. Must be one of: String, Result<String, SolveError>, PartResult<PartialResult>, or Result<PartResult<PartialResult>, SolveError>");
```

### Runtime Errors

The generated code returns appropriate errors:

- `SolveError::PartOutOfRange(part)`: When part number > max_parts
- `SolveError::SolveFailed(msg)`: When part function returns Err
- `SolveError::PartNotImplemented(part)`: Reserved for manual implementations (not used by macro)

### New Error Variant

Add to `aoc-solver/src/error.rs`:

```rust
pub enum SolveError {
    // ... existing variants ...
    
    /// Part number is out of range (exceeds max_parts)
    PartOutOfRange(usize),
}

impl Display for SolveError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing arms ...
            SolveError::PartOutOfRange(part) => {
                write!(f, "Part {} is out of range", part)
            }
        }
    }
}
```

## Testing Strategy

### Unit Tests

Unit tests verify specific macro behaviors:

1. **Attribute Parsing**: Test extraction of max_parts value
2. **Type Extraction**: Test extraction of Parsed and PartialResult types
3. **Function Detection**: Test detection of parse and part functions
4. **Error Generation**: Test that appropriate compile_error! messages are generated

### Property-Based Tests

Property-based tests verify universal behaviors across many inputs. Each test will run 10 iterations as specified.

1. **Type Forwarding Property**: Generate random type names, verify they appear in generated code
2. **Return Type Wrapping Property**: Generate different return types, verify correct wrapping
3. **Part Validation Property**: Generate different max_parts values, verify validation logic
4. **Path Qualification Property**: Verify all generated code uses fully qualified paths

### Integration Tests

Integration tests verify end-to-end functionality:

1. **Independent Parts Example**: Test solver with independent parts compiles and runs
2. **Dependent Parts Example**: Test solver with dependent parts compiles and runs
3. **AutoRegisterSolver Compatibility**: Test both macros work together
4. **Error Message Quality**: Test that error messages are helpful (using trybuild)

### Compile-Fail Tests

Use `trybuild` to test compile-time error messages:

1. Test missing max_parts attribute
2. Test missing type definitions
3. Test missing parse function
4. Test missing part1
5. Test gaps in part numbers
6. Test part numbers exceeding max_parts
7. Test invalid max_parts values
8. Test unsupported return types

## Implementation Notes

### Macro Structure

The macro will be implemented in `aoc-solver-macros/src/lib.rs` as an attribute macro:

```rust
#[proc_macro_attribute]
pub fn aoc_solver(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 1. Parse attribute to get max_parts
    // 2. Parse impl block
    // 3. Validate structure
    // 4. Extract components
    // 5. Generate code
    // 6. Return combined TokenStream
}
```

### Code Generation Strategy

1. **Preserve Original**: Keep the original impl block intact
2. **Generate Struct**: Only if it doesn't exist
3. **Generate Trait Impl**: Create Solver trait implementation
4. **Use Fully Qualified Paths**: Avoid import dependencies

### Return Type Detection

The macro analyzes return types using pattern matching:

```rust
fn detect_return_type(ty: &Type) -> Result<ReturnType, Error> {
    match ty {
        Type::Path(path) if is_string(path) => Ok(ReturnType::String),
        Type::Path(path) if is_result_string(path) => Ok(ReturnType::ResultString),
        Type::Path(path) if is_part_result(path) => Ok(ReturnType::PartResult),
        Type::Path(path) if is_result_part_result(path) => Ok(ReturnType::ResultPartResult),
        _ => Err(Error::UnsupportedReturnType),
    }
}
```

### Part Function Signature Detection

The macro detects independent vs dependent parts by parameter count:

```rust
fn analyze_part_signature(fn_item: &ImplItemFn) -> PartSignature {
    match fn_item.sig.inputs.len() {
        1 => PartSignature::Independent { ... },
        2 => PartSignature::Dependent { ... },
        _ => panic!("Invalid part function signature"),
    }
}
```

### Compatibility with AutoRegisterSolver

The macros work together because:
1. `#[aoc_solver]` generates the struct and Solver impl
2. `#[derive(AutoRegisterSolver)]` generates the registration code
3. Both operate on the same struct type
4. No naming conflicts occur

Usage:
```rust
#[derive(AutoRegisterSolver)]
#[aoc(year = 2023, day = 1)]
#[aoc_solver(max_parts = 2)]
impl Day1 {
    // ...
}
```

## Dependencies

- `syn`: For parsing Rust syntax
- `quote`: For generating Rust code
- `proc-macro2`: For token manipulation
- `trybuild`: For compile-fail tests (dev dependency)

## Future Enhancements

1. **Auto-detect max_parts**: Scan for part functions and infer max_parts
2. **Custom error messages**: Allow users to customize error messages
3. **Part aliases**: Support alternative names like `solve_part1`
4. **Async support**: Support async part functions
5. **Parallel execution**: Generate code for parallel part execution
