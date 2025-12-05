//! Procedural macros for the aoc-solver library

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Lit, PathSegment, ReturnType as SynReturnType, Type, TypePath};

/// Derive macro for automatically registering solvers with the plugin system
///
/// This macro generates the necessary code to register a solver with the inventory
/// system, allowing it to be discovered and registered automatically.
///
/// # Attributes
///
/// - `year`: Required. The Advent of Code year (e.g., 2023)
/// - `day`: Required. The day number (1-25)
/// - `tags`: Optional. Array of string literals for filtering (e.g., ["easy", "parsing"])
///
/// # Requirements
///
/// The type must implement the `Solver` trait. If the trait is not implemented,
/// you will get a clear compile-time error:
///
/// ```text
/// error[E0277]: the trait bound `YourSolver: Solver` is not satisfied
///   |
///   | struct YourSolver;
///   |        ^^^^^^^^^^ unsatisfied trait bound
///   |
/// help: the trait `Solver` is not implemented for `YourSolver`
/// ```
///
/// # Example
///
/// ```ignore
/// use aoc_solver::{Solver, ParseError, PartResult, SolveError};
/// use aoc_solver_macros::AutoRegisterSolver;
///
/// #[derive(AutoRegisterSolver)]
/// #[aoc(year = 2023, day = 1, tags = ["easy", "parsing"])]
/// struct Day1Solver;
///
/// impl Solver for Day1Solver {
///     // ... implementation
/// }
/// ```
#[proc_macro_derive(AutoRegisterSolver, attributes(aoc))]
pub fn derive_auto_register_solver(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // Extract the struct name
    let name = &input.ident;
    
    // Find the #[aoc(...)] attribute
    let aoc_attr = input.attrs.iter()
        .find(|attr| attr.path().is_ident("aoc"))
        .expect("AutoRegisterSolver derive macro requires #[aoc(...)] attribute");
    
    // Parse the attribute arguments
    let mut year: Option<u32> = None;
    let mut day: Option<u32> = None;
    let mut tags: Vec<String> = Vec::new();
    
    // Parse nested meta items
    aoc_attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("year") {
            let value: Lit = meta.value()?.parse()?;
            if let Lit::Int(lit_int) = value {
                year = Some(lit_int.base10_parse()?);
            }
        } else if meta.path.is_ident("day") {
            let value: Lit = meta.value()?.parse()?;
            if let Lit::Int(lit_int) = value {
                day = Some(lit_int.base10_parse()?);
            }
        } else if meta.path.is_ident("tags") {
            // Parse array of string literals: tags = ["a", "b"]
            let _ = meta.value()?;  // Consume the '='
            let content;
            syn::bracketed!(content in meta.input);
            while !content.is_empty() {
                let lit: Lit = content.parse()?;
                if let Lit::Str(lit_str) = lit {
                    tags.push(lit_str.value());
                }
                // Skip comma if present
                if content.peek(syn::Token![,]) {
                    let _: syn::Token![,] = content.parse()?;
                }
            }
        }
        Ok(())
    }).expect("Failed to parse #[aoc(...)] attribute");
    
    let year = year.expect("Missing required 'year' attribute");
    let day = day.expect("Missing required 'day' attribute");
    
    // Generate the tags array
    let tags_array = if tags.is_empty() {
        quote! { &[] }
    } else {
        let tag_strs = tags.iter().map(|s| s.as_str());
        quote! { &[#(#tag_strs),*] }
    };
    
    // Generate the code with a compile-time trait bound check
    let expanded = quote! {
        // Compile-time check that the type implements Solver trait
        // This generates a helpful error message if the trait is not implemented
        const _: () = {
            // Custom trait to provide a better error message
            trait MustImplementSolver: ::aoc_solver::Solver {}
            impl MustImplementSolver for #name {}
        };
        
        ::aoc_solver::inventory::submit! {
            ::aoc_solver::SolverPlugin {
                year: #year,
                day: #day,
                solver: &#name,
                tags: #tags_array,
            }
        }
    };
    
    TokenStream::from(expanded)
}

/// Attribute macro for generating Solver trait implementation
///
/// This macro simplifies solver implementation by automatically generating the
/// `Solver` trait implementation from an impl block containing type definitions,
/// a parse function, and part functions.
///
/// # Attributes
///
/// - `max_parts`: Required. The maximum number of parts (e.g., max_parts = 2)
///
/// # Requirements
///
/// The impl block must contain:
/// - `type Parsed = T`: The parsed input type
/// - `type PartialResult = T`: The type for data shared between parts (use `()` for independent parts)
/// - `fn parse(input: &str) -> Result<Parsed, ParseError>`: The parsing function
/// - `fn part1(parsed: &Parsed) -> ReturnType`: At minimum, part1 must be implemented
/// - Additional part functions up to max_parts
///
/// # Part Function Signatures
///
/// Part functions can have different signatures:
/// - Independent: `fn partN(parsed: &Parsed) -> ReturnType`
/// - Dependent: `fn partN(parsed: &Parsed, prev: Option<&PartialResult>) -> ReturnType`
///
/// Return types can be:
/// - `String`: Automatically wrapped in PartResult
/// - `Result<String, SolveError>`: Unwrapped and wrapped in PartResult
/// - `PartResult<PartialResult>`: Used directly
/// - `Result<PartResult<PartialResult>, SolveError>`: Used directly
///
/// # Example
///
/// ```ignore
/// use aoc_solver::{ParseError, PartResult, SolveError};
/// use aoc_solver_macros::aoc_solver;
///
/// struct Day1;  // Define the struct first
///
/// #[aoc_solver(max_parts = 2)]
/// impl Day1 {
///     type Parsed = Vec<i32>;
///     type PartialResult = ();
///     
///     fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
///         input.lines()
///             .map(|line| line.parse().map_err(|_| 
///                 ParseError::InvalidFormat("Expected integer".into())))
///             .collect()
///     }
///     
///     fn part1(parsed: &Vec<i32>) -> String {
///         parsed.iter().sum::<i32>().to_string()
///     }
///     
///     fn part2(parsed: &Vec<i32>) -> String {
///         parsed.iter().product::<i32>().to_string()
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn aoc_solver(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the max_parts attribute
    let max_parts = match parse_max_parts(attr) {
        Ok(n) => n,
        Err(e) => return e,
    };
    
    // Parse the impl block
    let impl_block = parse_macro_input!(item as ItemImpl);
    
    // Extract components from the impl block
    let components = extract_components(&impl_block);
    
    // Validate components and analyze signatures
    let signatures = match validate_components(&components, max_parts) {
        Ok(sigs) => sigs,
        Err(e) => return e,
    };
    
    // Generate a modified impl block without the type definitions
    let modified_impl = generate_modified_impl(&impl_block);
    
    // Generate Solver trait implementation
    let solver_impl = generate_solver_impl(&components, &signatures, max_parts);
    
    // Return modified impl block + Solver trait impl
    let expanded = quote! {
        #modified_impl
        #solver_impl
    };
    
    TokenStream::from(expanded)
}

/// Parse the max_parts attribute value
fn parse_max_parts(attr: TokenStream) -> Result<usize, TokenStream> {
    if attr.is_empty() {
        return Err(TokenStream::from(quote! {
            compile_error!("aoc_solver: missing required attribute 'max_parts'. Use: #[aoc_solver(max_parts = N)]");
        }));
    }
    
    // Use a cell to capture the value
    let mut max_parts: Option<usize> = None;
    
    // Parse as a meta list parser
    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("max_parts") {
            let value: Lit = meta.value()?.parse()?;
            if let Lit::Int(lit_int) = value {
                let n: usize = lit_int.base10_parse()?;
                if n < 1 {
                    return Err(meta.error("max_parts must be at least 1"));
                }
                max_parts = Some(n);
                return Ok(());
            }
        }
        Err(meta.error("expected max_parts = N"))
    });
    
    syn::parse::Parser::parse(parser, attr).map_err(|e| {
        let msg = format!("aoc_solver: {}", e);
        TokenStream::from(quote! {
            compile_error!(#msg);
        })
    })?;
    
    max_parts.ok_or_else(|| {
        TokenStream::from(quote! {
            compile_error!("aoc_solver: missing required attribute 'max_parts'. Use: #[aoc_solver(max_parts = N)]");
        })
    })
}


/// Extracted components from the impl block
#[allow(dead_code)]
struct ExtractedComponents {
    struct_name: Ident,
    parsed_type: Option<Type>,
    partial_result_type: Option<Type>,
    parse_fn: Option<ImplItemFn>,
    part_fns: Vec<(usize, ImplItemFn)>,
}

/// Extract all required components from the impl block
fn extract_components(impl_block: &ItemImpl) -> ExtractedComponents {
    let struct_name = match &*impl_block.self_ty {
        Type::Path(type_path) => {
            type_path.path.segments.last()
                .expect("Expected at least one path segment")
                .ident.clone()
        }
        _ => panic!("Expected a simple type path for impl block"),
    };
    
    let mut parsed_type: Option<Type> = None;
    let mut partial_result_type: Option<Type> = None;
    let mut parse_fn: Option<ImplItemFn> = None;
    let mut part_fns: Vec<(usize, ImplItemFn)> = Vec::new();
    
    for item in &impl_block.items {
        match item {
            ImplItem::Type(ty) => {
                if ty.ident == "Parsed" {
                    parsed_type = Some(ty.ty.clone());
                } else if ty.ident == "PartialResult" {
                    partial_result_type = Some(ty.ty.clone());
                }
            }
            ImplItem::Fn(func) => {
                let fn_name = func.sig.ident.to_string();
                
                if fn_name == "parse" {
                    parse_fn = Some(func.clone());
                } else if fn_name.starts_with("part") {
                    // Extract part number from function name (e.g., "part1" -> 1)
                    if let Ok(part_num) = fn_name[4..].parse::<usize>() {
                        part_fns.push((part_num, func.clone()));
                    }
                }
            }
            _ => {}
        }
    }
    
    // Sort part functions by part number
    part_fns.sort_by_key(|(num, _)| *num);
    
    ExtractedComponents {
        struct_name,
        parsed_type,
        partial_result_type,
        parse_fn,
        part_fns,
    }
}


/// Validate that all required components are present
fn validate_components(components: &ExtractedComponents, max_parts: usize) -> Result<Vec<PartSignature>, TokenStream> {
    // Validate Parsed type exists
    if components.parsed_type.is_none() {
        return Err(TokenStream::from(quote! {
            compile_error!("aoc_solver: missing required type 'Parsed'. Add: type Parsed = YourType;");
        }));
    }
    
    // Validate PartialResult type exists
    if components.partial_result_type.is_none() {
        return Err(TokenStream::from(quote! {
            compile_error!("aoc_solver: missing required type 'PartialResult'. Add: type PartialResult = YourType; (or () for independent parts)");
        }));
    }
    
    // Validate parse function exists
    if components.parse_fn.is_none() {
        return Err(TokenStream::from(quote! {
            compile_error!("aoc_solver: missing required 'parse' function. Add: fn parse(input: &str) -> Result<Parsed, ParseError> { ... }");
        }));
    }
    
    // Validate part1 exists
    if components.part_fns.is_empty() || components.part_fns[0].0 != 1 {
        return Err(TokenStream::from(quote! {
            compile_error!("aoc_solver: at least 'part1' function is required. Add: fn part1(parsed: &Parsed) -> String { ... }");
        }));
    }
    
    // Validate all parts from 1 to max_parts exist
    for expected_part in 1..=max_parts {
        if !components.part_fns.iter().any(|(num, _)| *num == expected_part) {
            let msg = format!(
                "aoc_solver: part{} is missing but max_parts = {}. All parts from 1 to max_parts must be implemented.",
                expected_part, max_parts
            );
            return Err(TokenStream::from(quote! {
                compile_error!(#msg);
            }));
        }
    }
    
    // Validate no parts exceed max_parts
    for (part_num, _) in &components.part_fns {
        if *part_num > max_parts {
            let msg = format!(
                "aoc_solver: part{} exceeds max_parts = {}. Remove this part or increase max_parts.",
                part_num, max_parts
            );
            return Err(TokenStream::from(quote! {
                compile_error!(#msg);
            }));
        }
    }
    
    // Analyze signatures of all part functions
    let mut signatures = Vec::new();
    for (_, func) in &components.part_fns {
        let sig = analyze_part_signature(func)?;
        signatures.push(sig);
    }
    
    Ok(signatures)
}


/// Return type classification for part functions
#[derive(Debug, Clone, Copy, PartialEq)]
enum ReturnType {
    String,
    ResultString,
    PartResult,
    ResultPartResult,
}

/// Part function signature information
#[allow(dead_code)]
struct PartSignature {
    is_dependent: bool,  // true if has prev parameter
    return_type: ReturnType,
}

/// Analyze a part function's signature
fn analyze_part_signature(func: &ImplItemFn) -> Result<PartSignature, TokenStream> {
    // Count parameters (excluding self)
    let param_count = func.sig.inputs.iter()
        .filter(|arg| !matches!(arg, FnArg::Receiver(_)))
        .count();
    
    let is_dependent = match param_count {
        1 => false,  // Independent: fn partN(parsed: &Parsed)
        2 => true,   // Dependent: fn partN(parsed: &Parsed, prev: Option<&PartialResult>)
        _ => {
            let fn_name = &func.sig.ident;
            let msg = format!(
                "aoc_solver: {} has invalid signature. Expected 1 parameter (independent) or 2 parameters (dependent).",
                fn_name
            );
            return Err(TokenStream::from(quote! {
                compile_error!(#msg);
            }));
        }
    };
    
    // Analyze return type
    let return_type = match &func.sig.output {
        SynReturnType::Default => {
            let fn_name = &func.sig.ident;
            let msg = format!(
                "aoc_solver: {} must have a return type. Supported types: String, Result<String, SolveError>, PartResult<PartialResult>, or Result<PartResult<PartialResult>, SolveError>",
                fn_name
            );
            return Err(TokenStream::from(quote! {
                compile_error!(#msg);
            }));
        }
        SynReturnType::Type(_, ty) => classify_return_type(ty, &func.sig.ident)?,
    };
    
    Ok(PartSignature {
        is_dependent,
        return_type,
    })
}

/// Classify the return type of a part function
fn classify_return_type(ty: &Type, fn_name: &Ident) -> Result<ReturnType, TokenStream> {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let last_segment = path.segments.last().unwrap();
            
            match last_segment.ident.to_string().as_str() {
                "String" => Ok(ReturnType::String),
                "Result" => {
                    // Check if it's Result<String, _> or Result<PartResult<_>, _>
                    if is_result_string(last_segment) {
                        Ok(ReturnType::ResultString)
                    } else if is_result_part_result(last_segment) {
                        Ok(ReturnType::ResultPartResult)
                    } else {
                        unsupported_return_type_error(fn_name)
                    }
                }
                "PartResult" => Ok(ReturnType::PartResult),
                _ => unsupported_return_type_error(fn_name),
            }
        }
        _ => unsupported_return_type_error(fn_name),
    }
}

/// Check if a Result type is Result<String, _>
fn is_result_string(segment: &PathSegment) -> bool {
    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
        if let Some(syn::GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.args.first() {
            if let Some(seg) = path.segments.last() {
                return seg.ident == "String";
            }
        }
    }
    false
}

/// Check if a Result type is Result<PartResult<_>, _>
fn is_result_part_result(segment: &PathSegment) -> bool {
    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
        if let Some(syn::GenericArgument::Type(Type::Path(TypePath { path, .. }))) = args.args.first() {
            if let Some(seg) = path.segments.last() {
                return seg.ident == "PartResult";
            }
        }
    }
    false
}

/// Generate error for unsupported return type
fn unsupported_return_type_error(fn_name: &Ident) -> Result<ReturnType, TokenStream> {
    let msg = format!(
        "aoc_solver: {} return type is not supported. Must be one of: String, Result<String, SolveError>, PartResult<PartialResult>, or Result<PartResult<PartialResult>, SolveError>",
        fn_name
    );
    Err(TokenStream::from(quote! {
        compile_error!(#msg);
    }))
}



/// Generate the Solver trait implementation
fn generate_solver_impl(
    components: &ExtractedComponents,
    signatures: &[PartSignature],
    max_parts: usize,
) -> proc_macro2::TokenStream {
    let struct_name = &components.struct_name;
    let parsed_type = components.parsed_type.as_ref().unwrap();
    let partial_result_type = components.partial_result_type.as_ref().unwrap();
    
    // Generate the solve_part match arms
    let match_arms = generate_match_arms(components, signatures, max_parts);
    
    quote! {
        impl ::aoc_solver::Solver for #struct_name {
            type Parsed = #parsed_type;
            type PartialResult = #partial_result_type;
            
            fn parse(input: &str) -> Result<Self::Parsed, ::aoc_solver::ParseError> {
                <#struct_name>::parse(input)
            }
            
            fn solve_part(
                parsed: &Self::Parsed,
                part: usize,
                previous_partial: Option<&Self::PartialResult>,
            ) -> Result<::aoc_solver::PartResult<Self::PartialResult>, ::aoc_solver::SolveError> {
                match part {
                    #(#match_arms)*
                    _ => Err(::aoc_solver::SolveError::PartOutOfRange(part)),
                }
            }
        }
    }
}

/// Generate match arms for solve_part
fn generate_match_arms(
    components: &ExtractedComponents,
    signatures: &[PartSignature],
    _max_parts: usize,
) -> Vec<proc_macro2::TokenStream> {
    let struct_name = &components.struct_name;
    
    components.part_fns.iter()
        .zip(signatures.iter())
        .map(|((part_num, func), sig)| {
            let part_fn_name = &func.sig.ident;
            
            let part_call = if sig.is_dependent {
                quote! { <#struct_name>::#part_fn_name(parsed, previous_partial) }
            } else {
                quote! { <#struct_name>::#part_fn_name(parsed) }
            };
            
            let wrapped_call = match sig.return_type {
                ReturnType::String => {
                    quote! {
                        let answer = #part_call;
                        Ok(::aoc_solver::PartResult {
                            answer,
                            partial: None,
                        })
                    }
                }
                ReturnType::ResultString => {
                    quote! {
                        let answer = #part_call?;
                        Ok(::aoc_solver::PartResult {
                            answer,
                            partial: None,
                        })
                    }
                }
                ReturnType::PartResult => {
                    quote! {
                        Ok(#part_call)
                    }
                }
                ReturnType::ResultPartResult => {
                    quote! {
                        #part_call
                    }
                }
            };
            
            quote! {
                #part_num => {
                    #wrapped_call
                }
            }
        })
        .collect()
}


/// Generate a modified impl block without type definitions
/// (since inherent associated types are unstable)
fn generate_modified_impl(impl_block: &ItemImpl) -> proc_macro2::TokenStream {
    let self_ty = &impl_block.self_ty;
    let generics = &impl_block.generics;
    
    // Filter out type definitions, keep only functions
    let items: Vec<_> = impl_block.items.iter()
        .filter(|item| !matches!(item, ImplItem::Type(_)))
        .collect();
    
    quote! {
        impl #generics #self_ty {
            #(#items)*
        }
    }
}
