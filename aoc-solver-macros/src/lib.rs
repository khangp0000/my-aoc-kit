//! Procedural macros for the aoc-solver library

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Lit};

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
