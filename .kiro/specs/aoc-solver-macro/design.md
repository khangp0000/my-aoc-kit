# Design Document

## Overview

The `#[aoc_solver]` attribute macro simplifies Advent of Code solver implementation by automatically generating the `Solver` trait implementation. Users write an impl block with type definitions, a parse function, and part functions, and the macro generates all the boilerplate code needed to satisfy the `Solver` trait.

The macro provides:
- Automatic trait implementation generation
- Flexible return type handling (String, Result<String>)
- Support for both independent and dependent parts through SharedData mutation
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
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<SharedData, ParseError> { ... }
    fn part1(shared: &mut Vec<i32>) -> String { ... }
    fn part2(shared: &mut Vec<i32>) -> Result<String, SolveError> { ... }
}
```

### Output Structure

```rust
// Modified impl block (without type definitions)
impl Day1 {
    fn parse(input: &str) -> Result<Vec<i32>, ParseError> { ... }
    fn part1(shared: &mut Vec<i32>) -> String { ... }
    fn part2(shared: &mut Vec<i32>) -> Result<String, SolveError> { ... }
}

// Generated Solver trait impl
impl ::aoc_solver::Solver for Day1 {
    type SharedData = Vec<i32>;
    
    fn parse(input: &str) -> Result<std::borrow::Cow<'_, Self::SharedData>, ::aoc_solver::ParseError> {
        <Day1>::parse(input).map(std::borrow::Cow::Owned)
    }
    
    fn solve_part(
        shared: &mut std::borrow::Cow<'_, Self::SharedData>,
        part: usize,
    ) -> Result<String, ::aoc_solver::SolveError> {
        match part {
            1 => Ok(<Day1>::part1(shared.to_mut())),
            2 => <Day1>::part2(shared.to_mut()),
            _ => Err(::aoc_solver::SolveError::PartNotImplemented(part)),
        }
    }
}
```

## Data Models

### Macro Input Structure

```rust
struct MacroInput {
    max_parts: usize,
    impl_block: ItemImpl,
    struct_name: Ident,
}

struct ExtractedComponents {
    shared_data_type: Option<Type>,
    parse_fn: Option<ImplItemFn>,
    part_fns: Vec<PartFunction>,
}

struct PartFunction {
    number: usize,
    fn_item: ImplItemFn,
    signature: PartSignature,
}

struct PartSignature {
    return_type: ReturnType,
}

enum ReturnType {
    String,
    ResultString,
}
```

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system-essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Type forwarding preserves user types
*For any* valid SharedData type definition in the impl block, the generated Solver trait SHALL use that exact type
**Validates: Requirements 2.1**

### Property 2: String returns are passed through correctly
*For any* part function returning String, the generated code SHALL return it directly
**Validates: Requirements 4.1**

### Property 3: Result returns are passed through correctly
*For any* part function returning Result<String, SolveError>, the generated code SHALL return it directly
**Validates: Requirements 4.2**

### Property 6: Part validation enforces completeness
*For any* max_parts value N, the macro SHALL verify that all parts from 1 to N exist
**Validates: Requirements 5.1**

### Property 7: All parts receive mutable shared data
*For any* part function, the generated code SHALL call it with `&mut SharedData`
**Validates: Requirements 6.1**

### Property 8: Parts can modify shared data
*For any* part function that modifies SharedData, those modifications SHALL persist for subsequent parts
**Validates: Requirements 7.1**

### Property 9: Fully qualified paths are used
*For any* generated code, all library types SHALL use fully qualified paths starting with ::aoc_solver::
**Validates: Requirements 10.1**

### Property 10: Valid parts dispatch correctly
*For any* part number from 1 to max_parts, solve_part SHALL call the corresponding part function
**Validates: Requirements 11.1**

### Property 11: Out-of-range parts return error
*For any* part number greater than max_parts, solve_part SHALL return Err(SolveError::PartNotImplemented(part))
**Validates: Requirements 11.2**

## Error Handling

### Compile-Time Errors

The macro generates helpful compile-time errors for common mistakes:

#### Missing max_parts Attribute
```rust
compile_error!("aoc_solver: missing required attribute 'max_parts'. Use: #[aoc_solver(max_parts = N)]");
```

#### Missing Type Definition
```rust
compile_error!("aoc_solver: missing required type 'SharedData'. Add: type SharedData = YourType;");
```

#### Missing parse Function
```rust
compile_error!("aoc_solver: missing required 'parse' function. Add: fn parse(input: &str) -> Result<SharedData, ParseError> { ... }");
```

#### Missing part1
```rust
compile_error!("aoc_solver: at least 'part1' function is required. Add: fn part1(shared: &mut SharedData) -> String { ... }");
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
compile_error!("aoc_solver: part1 return type is not supported. Must be one of: String or Result<String, SolveError>");
```

### Runtime Errors

The generated code returns appropriate errors:

- `SolveError::PartNotImplemented(part)`: When part number > max_parts
- `SolveError::SolveFailed(msg)`: When part function returns Err
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
2. **Type Extraction**: Test extraction of SharedData type
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

### Cow Handling in Generated Code

The macro bridges the gap between user-friendly part function signatures and the actual Solver trait requirements:

**User writes:**
```rust
fn part1(shared: &mut Vec<i32>) -> String {
    shared.iter().sum::<i32>().to_string()
}
```

**Solver trait requires:**
```rust
fn solve_part(
    shared: &mut Cow<'_, Self::SharedData>,
    part: usize,
) -> Result<String, SolveError>
```

**Macro generates:**
```rust
fn solve_part(
    shared: &mut ::std::borrow::Cow<'_, Self::SharedData>,
    part: usize,
) -> Result<String, ::aoc_solver::SolveError> {
    match part {
        1 => {
            // Call .to_mut() to get &mut SharedData from Cow
            Ok(<Day1>::part1(shared.to_mut()))
        }
        _ => Err(::aoc_solver::SolveError::PartNotImplemented(part)),
    }
}
```

**Key transformations:**

1. **Parse function wrapping**: User's `Result<SharedData, ParseError>` is wrapped with `Cow::Owned`:
   ```rust
   fn parse(input: &str) -> Result<::std::borrow::Cow<'_, Self::SharedData>, ::aoc_solver::ParseError> {
       <Day1>::parse(input).map(::std::borrow::Cow::Owned)
   }
   ```

2. **Part function calls**: User's `&mut SharedData` parameter is obtained via `.to_mut()`:
   ```rust
   <Day1>::part1(shared.to_mut())
   ```
   This triggers cloning only when the Cow contains borrowed data, enabling zero-copy optimization.

3. **Return type wrapping**: User's return types are wrapped appropriately:
   - `String` → `Ok(result)`
   - `Result<String, SolveError>` → passed through directly

**Benefits:**
- Users write simple, intuitive signatures
- Macro handles all Cow complexity automatically
- Zero-copy optimization works transparently
- Type safety maintained throughout

### Return Type Detection

The macro analyzes return types using pattern matching:

```rust
fn detect_return_type(ty: &Type) -> Result<ReturnType, Error> {
    match ty {
        Type::Path(path) if is_string(path) => Ok(ReturnType::String),
        Type::Path(path) if is_result_string(path) => Ok(ReturnType::ResultString),
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
