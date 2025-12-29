use aoc_solver::{
    AocParser, AocSolver, AutoRegisterSolver, ParseError, PartSolver, SolveError, Solver,
    SolverRegistryBuilder,
};

#[derive(AocSolver)]
#[aoc_solver(max_parts = 2)]
struct TestSolver1;

impl AocParser for TestSolver1 {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect()
    }
}

impl PartSolver<1> for TestSolver1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for TestSolver1 {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

// Test that we can manually register a solver created with AocSolver derive
#[test]
fn test_aoc_solver_with_manual_registration() {
    let input = "2\n3\n4";
    let mut shared = <TestSolver1 as AocParser>::parse(input).unwrap();
    assert_eq!(shared, vec![2, 3, 4]);

    // Test Solver trait is implemented
    let result = <TestSolver1 as Solver>::solve_part(&mut shared, 1).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_solver_can_be_registered_manually() {
    // Test that the solver can be manually registered and used via the registry
    let mut builder = SolverRegistryBuilder::new();
    builder
        .register(2023, 15, 2, |input: &str| {
            Ok(Box::new(aoc_solver::SolverInstance::<TestSolver1>::new(
                2023, 15, input,
            )?))
        })
        .expect("Failed to register solver");
    let registry = builder.build();

    let input = "2\n3\n4";
    let mut solver = registry
        .create_solver(2023, 15, input)
        .expect("Failed to create solver");

    let result1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(result1.answer, "9");

    let result2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(result2.answer, "24");
}

// Test combining both macros: AutoRegisterSolver + AocSolver
#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2023, day = 20, tags = ["test", "combined"])]
struct CombinedMacroSolver;

impl AocParser for CombinedMacroSolver {
    type SharedData<'a> = Vec<i32>;

    fn parse(input: &str) -> Result<Self::SharedData<'_>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect()
    }
}

impl PartSolver<1> for CombinedMacroSolver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().sum::<i32>().to_string())
    }
}

impl PartSolver<2> for CombinedMacroSolver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(shared.iter().product::<i32>().to_string())
    }
}

#[test]
fn test_both_macros_work_together() {
    // Test that the Solver trait is implemented
    let input = "5\n6\n7";
    let mut shared = <CombinedMacroSolver as AocParser>::parse(input).unwrap();
    assert_eq!(shared, vec![5, 6, 7]);

    let result = <CombinedMacroSolver as Solver>::solve_part(&mut shared, 1).unwrap();
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

    let result1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(result1.answer, "18");

    let result2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(result2.answer, "210");
}
