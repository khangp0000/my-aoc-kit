//! Solver registry for managing and creating solver instances

use crate::error::{ParseError, RegistrationError, SolverError};
use crate::instance::{DynSolver, SolverInstanceCow};
use std::collections::HashMap;

/// Factory function type for creating solver instances
pub type SolverFactory =
    Box<dyn for<'a> Fn(&'a str) -> Result<Box<dyn DynSolver + 'a>, ParseError>>;

/// Builder for constructing a SolverRegistry with fluent API
///
/// The builder pattern allows for method chaining and ensures the registry
/// is immutable after construction. It also provides duplicate detection
/// during registration.
///
/// # Example
///
/// ```ignore
/// # use aoc_solver::RegistryBuilder;
/// let registry = RegistryBuilder::new()
///     .register(2023, 1, |input| { /* ... */ Ok(Box::new(/* solver */)) })
///     .unwrap()
///     .register(2023, 2, |input| { /* ... */ Ok(Box::new(/* solver */)) })
///     .unwrap()
///     .build();
/// ```
pub struct RegistryBuilder {
    solvers: HashMap<(u16, u8), SolverFactory>,
}

impl RegistryBuilder {
    /// Create a new empty registry builder
    pub fn new() -> Self {
        Self {
            solvers: HashMap::new(),
        }
    }

    /// Register a solver factory function for a specific year and day
    ///
    /// Returns an error if a solver is already registered for the given year-day combination.
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `factory` - A function that takes input and returns a boxed DynSolver
    ///
    /// # Returns
    /// * `Ok(Self)` - Builder with the solver registered, ready for chaining
    /// * `Err(RegistrationError)` - Duplicate solver for this year-day combination
    pub fn register<F>(mut self, year: u16, day: u8, factory: F) -> Result<Self, RegistrationError>
    where
        F: for<'a> Fn(&'a str) -> Result<Box<dyn DynSolver + 'a>, ParseError> + 'static,
    {
        if self.solvers.contains_key(&(year, day)) {
            return Err(RegistrationError::DuplicateSolver(year, day));
        }
        self.solvers.insert((year, day), Box::new(factory));
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
    /// # use aoc_solver::RegistryBuilder;
    /// let registry = RegistryBuilder::new()
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
            self = plugin.solver.register_with(self, plugin.year, plugin.day)?;
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
    /// # use aoc_solver::RegistryBuilder;
    /// // Register only solvers tagged as "easy"
    /// let registry = RegistryBuilder::new()
    ///     .register_solver_plugins(|plugin| {
    ///         plugin.tags.contains(&"easy")
    ///     })
    ///     .unwrap()
    ///     .build();
    ///
    /// // Register only 2023 solvers
    /// let registry = RegistryBuilder::new()
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
                self = plugin.solver.register_with(self, plugin.year, plugin.day)?;
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
            solvers: self.solvers,
        }
    }
}

impl Default for RegistryBuilder {
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
/// # use aoc_solver::{RegistryBuilder, SolverRegistry};
/// let registry = RegistryBuilder::new().build();
/// // Can only create solvers, not register new ones
/// // let solver = registry.create_solver(2023, 1, "input data").unwrap();
/// ```
pub struct SolverRegistry {
    solvers: HashMap<(u16, u8), SolverFactory>,
}

impl SolverRegistry {
    /// Create a solver instance for a specific year and day
    ///
    /// # Arguments
    /// * `year` - The Advent of Code year
    /// * `day` - The day number (1-25)
    /// * `input` - The input string for the problem
    ///
    /// # Returns
    /// * `Ok(Box<dyn DynSolver>)` - Successfully created solver
    /// * `Err(SolverError)` - Solver not found or parsing failed
    pub fn create_solver<'a>(
        &self,
        year: u16,
        day: u8,
        input: &'a str,
    ) -> Result<Box<dyn DynSolver + 'a>, SolverError> {
        let factory = self
            .solvers
            .get(&(year, day))
            .ok_or(SolverError::NotFound(year, day))?;

        factory(input).map_err(SolverError::ParseError)
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
/// use aoc_solver::{ParseError, RegisterableSolver, RegistryBuilder, SolveError, Solver};
/// use std::borrow::Cow;
///
/// struct MyDay1;
///
/// impl Solver for MyDay1 {
///     type SharedData = ();
///     const PARTS: u8 = 2;
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
///     
///     fn solve_part(_: &mut Cow<'_, Self::SharedData>, _: u8) -> Result<String, SolveError> {
///         Err(SolveError::PartNotImplemented(0))
///     }
/// }
///
/// let solver = MyDay1;
/// let builder = RegistryBuilder::new();
/// let builder = solver.register_with(builder, 2023, 1).unwrap();
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
    /// * `Ok(RegistryBuilder)` - Builder with the solver registered
    /// * `Err(RegistrationError)` - Duplicate solver for this year-day combination
    fn register_with(
        &self,
        builder: RegistryBuilder,
        year: u16,
        day: u8,
    ) -> Result<RegistryBuilder, RegistrationError>;
}

/// Blanket implementation of RegisterableSolver for all Solver types
///
/// This allows any type implementing `Solver` to automatically work with
/// the plugin system and fluent builder API.
impl<S> RegisterableSolver for S
where
    S: crate::solver::Solver + Sync + 'static,
{
    fn register_with(
        &self,
        builder: RegistryBuilder,
        year: u16,
        day: u8,
    ) -> Result<RegistryBuilder, RegistrationError> {
        builder.register(year, day, move |input: &str| {
            let shared = S::parse(input)?;
            Ok(Box::new(SolverInstanceCow::<S>::new(year, day, shared)))
        })
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
/// use aoc_solver::{ParseError, SolveError, Solver, SolverPlugin};
/// use std::borrow::Cow;
///
/// struct Day1Solver;
///
/// impl Solver for Day1Solver {
///     type SharedData = ();
///     const PARTS: u8 = 1;
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
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
/// Note: This macro is kept for backward compatibility but now works with the
/// mutable registry pattern. For new code, consider using the builder pattern directly.
///
/// # Example
///
/// ```
/// use aoc_solver::{register_solver, ParseError, RegistryBuilder, SolveError, Solver, SolverRegistry};
/// use std::borrow::Cow;
///
/// struct MyDay1Solver;
///
/// impl Solver for MyDay1Solver {
///     type SharedData = ();
///     const PARTS: u8 = 1;
///     
///     fn parse(_: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
///         Ok(Cow::Owned(()))
///     }
///     
///     fn solve_part(_: &mut Cow<'_, Self::SharedData>, _: u8) -> Result<String, SolveError> {
///         Err(SolveError::PartNotImplemented(0))
///     }
/// }
///
/// // Old style (still works for backward compatibility)
/// let mut builder = RegistryBuilder::new();
/// register_solver!(builder, MyDay1Solver, 2023, 1);
/// let registry = builder.build();
/// ```
#[macro_export]
macro_rules! register_solver {
    ($builder:expr, $solver:ty, $year:expr, $day:expr) => {
        $builder = $builder
            .register($year, $day, |input: &str| {
                let shared = <$solver>::parse(input)?;
                Ok(Box::new($crate::SolverInstanceCow::<$solver>::new(
                    $year, $day, shared,
                )))
            })
            .expect("Failed to register solver");
    };
}
