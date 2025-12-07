use aoc_solver::{
    AocParser, AocSolver, AutoRegisterSolver, ParseError, PartSolver, SolveError, Solver,
    SolverRegistryBuilder,
};
use std::borrow::Cow;

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct TestSolver1;

impl AocParser for TestSolver1 {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
}

impl PartSolver<1> for TestSolver1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for TestSolver1 {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

// Test that we can manually register a solver created with AocSolver derive
#[test]
fn test_aoc_solver_with_manual_registration() {
    let input = "2\n3\n4";
    let mut cow = <TestSolver1 as AocParser>::parse(input).unwrap();
    assert_eq!(*cow, vec![2, 3, 4]);

    // Test Solver trait is implemented
    let result = <TestSolver1 as Solver>::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_solver_can_be_registered_manually() {
    // Test that the solver can be manually registered and used via the registry
    let builder = SolverRegistryBuilder::new();
    let builder = builder
        .register(2023, 15, |input: &str| {
            let shared = <TestSolver1 as AocParser>::parse(input)?;
            Ok(Box::new(aoc_solver::SolverInstanceCow::<TestSolver1>::new(
                2023, 15, shared,
            )))
        })
        .expect("Failed to register solver");
    let registry = builder.build();

    let input = "2\n3\n4";
    let mut solver = registry
        .create_solver(2023, 15, input)
        .expect("Failed to create solver");

    let answer1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(answer1, "9");

    let answer2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(answer2, "24");
}

// Test combining both macros: AutoRegisterSolver + AocSolver
#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 20, tags = ["test", "combined"])]
struct CombinedMacroSolver;

impl AocParser for CombinedMacroSolver {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Cow<'_, Self::SharedData>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Cow::Owned)
    }
}

impl PartSolver<1> for CombinedMacroSolver {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for CombinedMacroSolver {
    fn solve(shared: &mut Cow<'_, Vec<i32>>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

#[test]
fn test_both_macros_work_together() {
    // Test that the Solver trait is implemented
    let input = "5\n6\n7";
    let mut cow = <CombinedMacroSolver as AocParser>::parse(input).unwrap();
    assert_eq!(*cow, vec![5, 6, 7]);

    let result = <CombinedMacroSolver as Solver>::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "18");
}

#[test]
fn test_combined_solver_auto_registers() {
    // Test that the solver is automatically registered via the plugin system
    let registry = SolverRegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    let input = "5\n6\n7";
    let mut solver = registry
        .create_solver(2023, 20, input)
        .expect("Failed to create solver - was it registered?");

    let answer1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(answer1, "18");

    let answer2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(answer2, "210");
}
