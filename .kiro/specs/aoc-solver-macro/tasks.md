# Implementation Plan

- [x] 1. Add PartOutOfRange error variant to SolveError enum
  - Add new variant to SolveError enum in aoc-solver/src/error.rs
  - Implement Display trait for the new variant
  - Update error documentation
  - _Requirements: 12.1, 12.2, 12.3_

- [ ]* 1.1 Write unit tests for PartOutOfRange error
  - Test Display implementation shows correct message
  - Test error can be constructed and matched
  - _Requirements: 12.2_

- [x] 2. Implement attribute parsing for max_parts
  - Parse #[aoc_solver(max_parts = N)] attribute
  - Extract max_parts value as usize
  - Validate max_parts >= 1
  - Generate compile_error! for missing or invalid max_parts
  - _Requirements: 1.1, 5.2, 5.5_

- [ ]* 2.1 Write unit tests for attribute parsing
  - Test valid max_parts values are extracted correctly
  - Test missing max_parts produces error
  - Test invalid max_parts values produce errors
  - _Requirements: 5.2, 5.5_

- [x] 3. Implement impl block parsing
  - Parse ItemImpl from input TokenStream
  - Extract struct name from impl block
  - Extract type definitions (Parsed, PartialResult)
  - Extract parse function
  - Extract part functions (part1, part2, etc.)
  - _Requirements: 1.2, 2.1, 2.2, 3.1_

- [ ]* 3.1 Write unit tests for impl block parsing
  - Test type extraction works correctly
  - Test function extraction works correctly
  - Test handles missing components appropriately
  - _Requirements: 2.1, 2.2, 3.1_

- [x] 4. Implement validation logic
  - Validate Parsed type exists
  - Validate PartialResult type exists
  - Validate parse function exists
  - Validate part1 exists
  - Validate all parts from 1 to max_parts exist
  - Validate no parts exceed max_parts
  - Generate appropriate compile_error! messages for each failure
  - _Requirements: 2.3, 3.2, 5.1, 5.3, 5.4, 8.1_

- [ ]* 4.1 Write property test for validation completeness
  - **Property 6: Part validation enforces completeness**
  - **Validates: Requirements 5.1**
  - Generate random max_parts values
  - Generate part sets with gaps or excess parts
  - Verify validation catches all issues
  - Run 10 iterations

- [x] 5. Implement part function signature analysis
  - Detect parameter count (1 for independent, 2 for dependent)
  - Extract parameter types
  - Detect return type (String, Result<String>, PartResult, Result<PartResult>)
  - Validate return types are supported
  - Generate compile_error! for unsupported return types
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 6.1, 7.1, 8.3_

- [ ]* 5.1 Write property test for return type detection
  - **Property 2: String returns are wrapped correctly**
  - **Property 3: Result returns are unwrapped and wrapped**
  - **Property 4: PartResult returns are passed through**
  - **Property 5: Result<PartResult> returns are passed through**
  - **Validates: Requirements 4.1, 4.2, 4.3, 4.4, 6.2**
  - Generate different return type patterns
  - Verify correct detection and handling
  - Run 10 iterations

- [x] 6. Implement struct generation
  - Check if struct already exists in scope
  - Generate unit struct declaration if needed
  - Avoid duplicate struct declarations
  - _Requirements: 9.1, 9.2_

- [ ]* 6.1 Write property test for struct generation
  - **Property 10: Struct generation avoids duplicates**
  - **Validates: Requirements 9.1, 9.2**
  - Generate random struct names
  - Test with and without existing structs
  - Verify no duplicates are created
  - Run 10 iterations

- [x] 7. Implement Solver trait code generation
  - Generate impl Solver for StructName block
  - Forward Parsed and PartialResult types
  - Generate parse method that calls user's parse function
  - Generate solve_part method with match statement
  - Use fully qualified paths (::aoc_solver::)
  - _Requirements: 1.1, 1.2, 1.3, 10.1, 10.2_

- [ ]* 7.1 Write property test for type forwarding
  - **Property 1: Type forwarding preserves user types**
  - **Validates: Requirements 2.1, 2.2**
  - Generate random type names
  - Verify they appear correctly in generated trait impl
  - Run 10 iterations

- [ ]* 7.2 Write property test for fully qualified paths
  - **Property 11: Fully qualified paths are used**
  - **Validates: Requirements 10.1**
  - Scan all generated code
  - Verify all library types use ::aoc_solver:: prefix
  - Run 10 iterations

- [x] 8. Implement match arm generation for solve_part
  - Generate match arm for each part number
  - Call corresponding part function
  - Handle independent parts (1 parameter)
  - Handle dependent parts (2 parameters)
  - Wrap return values based on type
  - Generate default arm returning PartOutOfRange
  - _Requirements: 1.3, 6.1, 7.1, 11.1, 11.2_

- [ ]* 8.1 Write property test for part dispatch
  - **Property 7: Independent parts receive only parsed data**
  - **Property 8: Dependent parts receive previous partial**
  - **Property 12: Valid parts dispatch correctly**
  - **Property 13: Out-of-range parts return error**
  - **Validates: Requirements 6.1, 7.1, 11.1, 11.2**
  - Generate different part configurations
  - Verify correct dispatch and parameter passing
  - Run 10 iterations

- [ ]* 8.2 Write property test for partial data flow
  - **Property 9: Partial data flows between parts**
  - **Validates: Requirements 7.2**
  - Generate solvers with dependent parts
  - Verify data from part1 reaches part2
  - Run 10 iterations

- [x] 9. Implement return value wrapping logic
  - For String: wrap in PartResult { answer, partial: None }
  - For Result<String, E>: unwrap with ? then wrap
  - For PartResult<T>: use directly
  - For Result<PartResult<T>, E>: use directly
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 6.2_

- [x] 10. Create integration test for independent parts
  - Create example solver with independent parts
  - Verify it compiles
  - Verify it implements Solver trait
  - Verify solve_part dispatches correctly
  - _Requirements: 1.1, 6.1, 6.2_

- [x] 11. Create integration test for dependent parts
  - Create example solver with dependent parts
  - Verify it compiles
  - Verify partial data flows between parts
  - _Requirements: 1.1, 7.1, 7.2_

- [x] 12. Create integration test for AutoRegisterSolver compatibility
  - Create solver using both macros
  - Verify both macros generate code without conflicts
  - Verify solver can be registered and used
  - _Requirements: 13.1, 13.2_

- [ ] 13. Create compile-fail tests using trybuild
  - Test missing max_parts attribute
  - Test missing Parsed type
  - Test missing PartialResult type
  - Test missing parse function
  - Test missing part1
  - Test gaps in part numbers
  - Test parts exceeding max_parts
  - Test invalid max_parts values
  - Test unsupported return types
  - Verify error messages are helpful
  - _Requirements: 2.3, 3.2, 5.2, 5.3, 5.4, 5.5, 8.1, 8.3_

- [x] 14. Update documentation and examples
  - Add macro documentation to aoc-solver-macros/src/lib.rs
  - Create example showing basic usage
  - Create example showing dependent parts
  - Update aoc-solver README with macro usage
  - Add migration guide from manual implementation
  - _Requirements: 1.1_

- [x] 15. Final checkpoint - Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.
