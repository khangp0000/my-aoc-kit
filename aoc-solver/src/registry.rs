//! Unified solver registry for managing and creating solver instances.
//!
//! This module provides a single registry system for Advent of Code solvers:
//!
//! - [`SolverRegistry`] - The immutable registry for looking up and creating solvers
//! - [`SolverRegistryBuilder`] - Builder pattern for constructing registries
//! - [`SolverRegistryStorage`] - Internal storage with efficient year/day indexing
//! - [`SolverFactory`] - Thread-safe factory function type (always Send + Sync)
//! - [`SolverInfo`] - Metadata about registered solvers
//! - [`RegisterableSolver`] - Trait for self-registering solvers
//! - [`SolverPlugin`] - Plugin system for automatic solver discovery
//!
//! # Storage Layout
//!
//! The registry uses a flat `Vec<Option<SolverFactoryEntry>>` with capacity for
//! 20 years × 25 days = 500 entries. Index calculation: `(year - 2015) * 25 + (day - 1)`.
//! This provides O(1) lookup and maintains ordering for iteration.
//!
//! # Example
//!
//! ```no_run
//! use aoc_solver::SolverRegistryBuilder;
//!
//! let registry = SolverRegistryBuilder::new()
//!     .register_all_plugins()
//!     .expect("Failed to register plugins")
//!     .build();
//!
//! // Create and use a solver
//! if let Ok(mut solver) = registry.create_solver(2023, 1, "input data") {
//!     let answer = solver.solve(1).expect("Failed to solve");
//!     println!("Answer: {}", answer);
//! }
//! ```

use crate::error::{ParseError, RegistrationError, SolverError};
use crate::instance::{DynSolver, SolverInstanceCow};

// ============================================================================
// Storage Constants and Index Calculation
// ============================================================================

/// Base year for AoC (first year of Advent of Code)
pub const BASE_YEAR: u16 = 2015;
/// Maximum number of years supported (2015-2034)
pub const MAX_YEARS: usize = 20;
/// Days per year in AoC (1-25)
pub const DAYS_PER_YEAR: usize = 25;
/// Total capacity of the flat storage
pub const CAPACITY: usize = MAX_YEARS * DAYS_PER_YEAR;

/// Calculate flat index from year/day, returning None if out of bounds
#[inline]
fn calc_index(year: u16, day: u8) -> Option<usize> {
    if year < BASE_YEAR || year >= BASE_YEAR + MAX_YEARS as u16 {
        return None;
    }
    if day == 0 || day > DAYS_PER_YEAR as u8 {
        return None;
    }
    let y = (year - BASE_YEAR) as usize;
    let d = (day - 1) as usize;
    Some(y * DAYS_PER_YEAR + d)
}

/// Reconstruct year/day from flat index
#[inline]
fn from_index(index: usize) -> (u16, u8) {
    let year = BASE_YEAR + (index / DAYS_PER_YEAR) as u16;
    let day = (index % DAYS_PER_YEAR) as u8 + 1;
    (year, day)
}

// ============================================================================
// Factory Types
// ============================================================================

/// Thread-safe factory function type for creating solver instances
pub type SolverFactory =
    Box<dyn for<'a> Fn(&'a str) -> Result<Box<dyn DynSolver + 'a>, ParseError> + Send + Sync>;

/// Metadata about a registered solver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SolverInfo {
    /// The Advent of Code year
    pub year: u16,
    /// The day number (1-25)
    pub day: u8,
    /// Number of parts this solver supports
    pub parts: u8,
}

/// Factory entry with metadata
struct SolverFactoryEntry {
    factory: SolverFactory,
    parts: u8,
}

/// Builder for constructing a SolverRegistry with fluent API
///
/// The builder pattern allows for method chaining and ensures the registry
/// is immutable after construction. It also provides duplicate detection
/// during registration.
///
/// # Example
///
/// ```ignore
/// # use aoc_solver::SolverRegistryBuilder;
/// let registry = SolverRegistryBuilder::new()
///     .register(2023, 1, |input| { /* ... */ Ok(Box::new(/* solver */)) })
///     .unwrap()
///     .register(2023, 2, |input| { /* ... */ Ok(Box::new(/* solver */)) })
///     .unwrap()
///     .build();
/// ```
pub struct SolverRegistryBuilder {
    entries: Vec<Option<SolverFactoryEntry>>,
}

impl SolverRegistryBuilder {
    /// Create a new empty registry builder with pre-allocated storage
    pub fn new() -> Self {
        Self {
            entries: (0..CAPACITY).map(|_| None).collect(),
        }
    }

    /// Register a solver factory with explicit parts count
    ///
    /// Returns error if year/day is out of bounds or already registered.
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year (2015-2034)
    /// * `day` - The day number (1-25)
    /// * `parts` - Number of parts this solver supports
    /// * `factory` - A function that takes input and returns a boxed DynSolver
    ///
    /// # Returns
    /// * `Ok(&mut Self)` - Builder with the solver registered, ready for chaining
    /// * `Err(RegistrationError)` - Invalid year/day or duplicate registration
    pub fn register_factory<F>(
        &mut self,
        year: u16,
        day: u8,
        parts: u8,
        factory: F,
    ) -> Result<&mut Self, RegistrationError>
    where
        F: for<'a> Fn(&'a str) -> Result<Box<dyn DynSolver + 'a>, ParseError>
            + Send
            + Sync
            + 'static,
    {
        let index = calc_index(year, day).ok_or(RegistrationError::InvalidYearDay(year, day))?;

        if self.entries[index].is_some() {
            return Err(RegistrationError::DuplicateSolverFactory(year, day));
        }

        self.entries[index] = Some(SolverFactoryEntry {
            factory: Box::new(factory),
            parts,
        });
        Ok(self)
    }

    /// Register a solver factory function for a specific year and day (legacy wrapper)
    ///
    /// This is a convenience method that defaults to 2 parts (standard AoC).
    /// For explicit parts count, use `register_factory()`.
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `factory` - A function that takes input and returns a boxed DynSolver
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with the solver registered, ready for chaining
    /// * `Err(RegistrationError)` - Invalid year/day or duplicate registration
    pub fn register<F>(mut self, year: u16, day: u8, factory: F) -> Result<Self, RegistrationError>
    where
        F: for<'a> Fn(&'a str) -> Result<Box<dyn DynSolver + 'a>, ParseError>
            + Send
            + Sync
            + 'static,
    {
        self.register_factory(year, day, 2, factory)?;
        Ok(self)
    }

    /// Register all collected solver plugins
    ///
    /// Iterates through all plugins submitted via `inventory::submit!` and
    /// registers each one with the builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use aoc_solver::SolverRegistryBuilder;
    /// let registry = SolverRegistryBuilder::new()
    ///     .register_all_plugins()
    ///     .unwrap()
    ///     .build();
    /// ```
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with all plugins registered
    /// * `Err(RegistrationError)` - Duplicate solver found
    pub fn register_all_plugins(mut self) -> Result<Self, RegistrationError> {
        for plugin in inventory::iter::<SolverPlugin>() {
            plugin.solver.register_with(&mut self, plugin.year, plugin.day)?;
        }
        Ok(self)
    }

    /// Register solver plugins that match the given filter predicate
    ///
    /// Only registers plugins for which the filter function returns `true`.
    /// This allows selective registration based on tags, year, day, or any
    /// other criteria.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use aoc_solver::SolverRegistryBuilder;
    /// // Register only solvers tagged as "easy"
    /// let registry = SolverRegistryBuilder::new()
    ///     .register_solver_plugins(|plugin| {
    ///         plugin.tags.contains(&"easy")
    ///     })
    ///     .unwrap()
    ///     .build();
    ///
    /// // Register only 2023 solvers
    /// let registry = SolverRegistryBuilder::new()
    ///     .register_solver_plugins(|plugin| plugin.year == 2023)
    ///     .unwrap()
    ///     .build();
    /// ```
    ///
    /// # Arguments
    /// * `filter` - A predicate function that determines which plugins to register
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with matching plugins registered
    /// * `Err(RegistrationError)` - Duplicate solver found
    pub fn register_solver_plugins<F>(mut self, filter: F) -> Result<Self, RegistrationError>
    where
        F: Fn(&SolverPlugin) -> bool,
    {
        for plugin in inventory::iter::<SolverPlugin>() {
            if filter(plugin) {
                plugin.solver.register_with(&mut self, plugin.year, plugin.day)?;
            }
        }
        Ok(self)
    }

    /// Finalize the builder and create an immutable registry
    ///
    /// Consumes the builder and returns a `SolverRegistry` that can only
    /// be used for solver lookup and creation.
    pub fn build(self) -> SolverRegistry {
        SolverRegistry {
            storage: SolverRegistryStorage {
                entries: self.entries,
            },
        }
    }
}

impl Default for SolverRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Immutable registry for looking up and creating solvers
///
/// The registry maps (year, day) pairs to factory functions that can create
/// solver instances. Once built, it cannot be modified.
///
/// # Example
///
/// ```no_run
/// # use aoc_solver::{SolverRegistryBuilder, SolverRegistry};
/// let registry = SolverRegistryBuilder::new().build();
/// // Can only create solvers, not register new ones
/// // let solver = registry.create_solver(2023, 1, "input data").unwrap();
/// ```
pub struct SolverRegistry {
    storage: SolverRegistryStorage,
}

impl SolverRegistry {
    /// Get readonly access to the storage for iteration/lookup
    pub fn storage(&self) -> &SolverRegistryStorage {
        &self.storage
    }

    /// Create a solver instance for a specific year and day
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `input` - The input string for the problem
    ///
    /// # Returns
    /// * `Ok(Box<dyn DynSolver>)` - Successfully created solver
    /// * `Err(SolverError)` - Solver not found, invalid year/day, or parsing failed
    pub fn create_solver<'a>(
        &self,
        year: u16,
        day: u8,
        input: &'a str,
    ) -> Result<Box<dyn DynSolver + 'a>, SolverError> {
        let index = calc_index(year, day).ok_or(SolverError::InvalidYearDay(year, day))?;

        let entry = self
            .storage
            .entries
            .get(index)
            .and_then(|e| e.as_ref())
            .ok_or(SolverError::NotFound(year, day))?;

        (entry.factory)(input).map_err(SolverError::ParseError)
    }
}

/// Trait for solvers that can register themselves with a registry builder
///
/// This trait provides a type-erased interface for solvers to self-register.
/// Unlike the `Solver` trait which has associated types, this trait has no
/// associated types, allowing for collection of different solver types in
/// a single container.
///
/// # Automatic Implementation
///
/// Any type implementing `Solver` automatically gets a `RegisterableSolver`
/// implementation through a blanket impl, enabling it to be used in the
/// plugin system with the fluent builder API.
///
/// # Example
///
/// ```no_run
/// use aoc_solver::{AocParser, ParseError, RegisterableSolver, SolverRegistryBuilder, SolveError, Solver};
/// use std::borrow::Cow;
///
/// struct MyDay1;
///
/// impl AocParser for MyDay1 {
///     type SharedData = ();
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
/// }
///
/// impl Solver for MyDay1 {
///     const PARTS: u8 = 2;
///     
///     fn solve_part(_: &mut Cow<'_, Self::SharedData>, _: u8) -> Result<String, SolveError> {
///         Err(SolveError::PartNotImplemented(0))
///     }
/// }
///
/// let solver = MyDay1;
/// let mut builder = SolverRegistryBuilder::new();
/// solver.register_with(&mut builder, 2023, 1).unwrap();
/// let registry = builder.build();
/// ```
pub trait RegisterableSolver: Sync {
    /// Register this solver type with the builder for a specific year and day
    ///
    /// # Arguments
    /// * `builder` - The registry builder to register with
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    ///
    /// # Returns
    /// * `Ok(&mut SolverRegistryBuilder)` - Builder with the solver registered
    /// * `Err(RegistrationError)` - Duplicate solver or invalid year/day
    fn register_with<'a>(
        &self,
        builder: &'a mut SolverRegistryBuilder,
        year: u16,
        day: u8,
    ) -> Result<&'a mut SolverRegistryBuilder, RegistrationError>;

    /// Get the number of parts this solver supports
    fn parts(&self) -> u8;
}

/// Blanket implementation of RegisterableSolver for all Solver types
///
/// This allows any type implementing `Solver` to automatically work with
/// the plugin system and fluent builder API.
impl<S> RegisterableSolver for S
where
    S: crate::solver::Solver + Sync + 'static,
{
    fn register_with<'a>(
        &self,
        builder: &'a mut SolverRegistryBuilder,
        year: u16,
        day: u8,
    ) -> Result<&'a mut SolverRegistryBuilder, RegistrationError> {
        builder.register_factory(year, day, S::PARTS, move |input: &str| {
            let shared = S::parse(input)?;
            Ok(Box::new(SolverInstanceCow::<S>::new(year, day, shared)))
        })
    }

    fn parts(&self) -> u8 {
        S::PARTS
    }
}

/// Plugin information for automatic solver registration
///
/// This struct holds metadata about a solver plugin, including its year, day,
/// a type-erased solver instance, and optional tags for filtering.
///
/// # Example
///
/// ```no_run
/// use aoc_solver::{AocParser, ParseError, SolveError, Solver, SolverPlugin};
/// use std::borrow::Cow;
///
/// struct Day1Solver;
///
/// impl AocParser for Day1Solver {
///     type SharedData = ();
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
/// }
///
/// impl Solver for Day1Solver {
///     const PARTS: u8 = 1;
///     
///     fn solve_part(_: &mut Cow<'_, Self::SharedData>, _: u8) -> Result<String, SolveError> {
///         Err(SolveError::PartNotImplemented(0))
///     }
/// }
///
/// inventory::submit! {
///     SolverPlugin {
///         year: 2023,
///         day: 1,
///         solver: &Day1Solver,
///         tags: &["2023", "easy"],
///     }
/// }
/// ```
pub struct SolverPlugin {
    /// The Advent of Code year
    pub year: u16,
    /// The day number (1-25)
    pub day: u8,
    /// The solver instance (type-erased)
    pub solver: &'static dyn RegisterableSolver,
    /// Optional tags for filtering (e.g., "easy", "hard", "2023", "parsing")
    pub tags: &'static [&'static str],
}

// Enable plugin collection via inventory
inventory::collect!(SolverPlugin);

/// Macro to register a solver with the registry builder
///
/// This macro simplifies the registration process by automatically creating
/// a factory function that parses input and wraps the result in a SolverInstance.
///
/// # Example
///
/// ```
/// use aoc_solver::{AocParser, register_solver, ParseError, SolverRegistryBuilder, SolveError, Solver, SolverRegistry};
/// use std::borrow::Cow;
///
/// struct MyDay1Solver;
///
/// impl AocParser for MyDay1Solver {
///     type SharedData = ();
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
/// }
///
/// impl Solver for MyDay1Solver {
///     const PARTS: u8 = 1;
///     
///     fn solve_part(_: &mut Cow<'_, Self::SharedData>, _: u8) -> Result<String, SolveError> {
///         Err(SolveError::PartNotImplemented(0))
///     }
/// }
///
/// let mut builder = SolverRegistryBuilder::new();
/// register_solver!(builder, MyDay1Solver, 2023, 1);
/// let registry = builder.build();
/// ```
#[macro_export]
macro_rules! register_solver {
    ($builder:expr, $solver:ty, $year:expr, $day:expr) => {
        $builder
            .register_factory($year, $day, <$solver>::PARTS, |input: &str| {
                let shared = <$solver>::parse(input)?;
                Ok(Box::new($crate::SolverInstanceCow::<$solver>::new(
                    $year, $day, shared,
                )))
            })
            .expect("Failed to register solver");
    };
}

// ============================================================================
// Factory Storage Implementation
// ============================================================================

/// Immutable storage for solver factories.
///
/// Provides efficient lookup and iteration over registered solver factories.
/// Supports years 2015-2034 and days 1-25. The internal implementation may
/// vary to optimize for different use cases (e.g., memory vs speed).
///
/// # Ordering Guarantee
///
/// All iteration methods (`iter_info`, `iter_factories`) MUST yield items
/// in ascending (year, day) order. This is a contract that consumers rely on
/// for grouping operations like `chunk_by`. Any alternative storage implementation
/// must maintain this ordering invariant.
pub struct SolverRegistryStorage {
    entries: Vec<Option<SolverFactoryEntry>>,
}

impl SolverRegistryStorage {
    /// Iterate over metadata for all registered factories.
    ///
    /// Items are yielded in ascending (year, day) order. This ordering is
    /// guaranteed and can be relied upon for grouping operations like `chunk_by`.
    pub fn iter_info(&self) -> impl Iterator<Item = SolverInfo> + '_ {
        self.entries.iter().enumerate().filter_map(|(i, entry)| {
            entry.as_ref().map(|e| {
                let (year, day) = from_index(i);
                SolverInfo {
                    year,
                    day,
                    parts: e.parts,
                }
            })
        })
    }

    /// Get metadata for a specific factory
    pub fn get_info(&self, year: u16, day: u8) -> Option<SolverInfo> {
        calc_index(year, day)
            .and_then(|i| self.entries.get(i)?.as_ref())
            .map(|e| SolverInfo {
                year,
                day,
                parts: e.parts,
            })
    }

    /// Check if a factory exists for year/day
    pub fn contains(&self, year: u16, day: u8) -> bool {
        self.get_info(year, day).is_some()
    }

    /// Iterate over all factories with their metadata.
    ///
    /// Items are yielded in ascending (year, day) order. This ordering is
    /// guaranteed and can be relied upon for grouping operations like `chunk_by`.
    pub fn iter_factories(&self) -> impl Iterator<Item = (SolverInfo, &SolverFactory)> + '_ {
        self.entries.iter().enumerate().filter_map(|(i, entry)| {
            entry.as_ref().map(|e| {
                let (year, day) = from_index(i);
                (
                    SolverInfo {
                        year,
                        day,
                        parts: e.parts,
                    },
                    &e.factory,
                )
            })
        })
    }

    /// Get the number of registered factories
    pub fn len(&self) -> usize {
        self.entries.iter().filter(|e| e.is_some()).count()
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.entries.iter().all(|e| e.is_none())
    }
}


