use aoc_solver::{ParseError, RegistryBuilder, Solver};
use aoc_solver_macros::aoc_solver;

struct TestSolver1;

// Test that aoc_solver macro works
#[aoc_solver(max_parts = 2)]
impl TestSolver1 {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect()
    }

    fn part1(shared: &mut Vec<i32>) -> String {
        shared.iter().sum::<i32>().to_string()
    }

    fn part2(shared: &mut Vec<i32>) -> String {
        shared.iter().product::<i32>().to_string()
    }
}

// Test that we can manually register a solver created with aoc_solver
#[test]
fn test_aoc_solver_with_manual_registration() {
    let input = "2\n3\n4";
    let mut cow = <TestSolver1 as Solver>::parse(input).unwrap();
    assert_eq!(*cow, vec![2, 3, 4]);

    // Test Solver trait is implemented
    let result = TestSolver1::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "9");
}

#[test]
fn test_solver_can_be_registered_manually() {
    // Test that the solver can be manually registered and used via the registry
    let builder = RegistryBuilder::new();
    let builder = builder
        .register(2023, 99, |input: &str| {
            let shared = <TestSolver1 as Solver>::parse(input)?;
            Ok(Box::new(aoc_solver::SolverInstanceCow::<TestSolver1>::new(
                2023, 99, shared,
            )))
        })
        .expect("Failed to register solver");
    let registry = builder.build();

    let input = "2\n3\n4";
    let mut solver = registry
        .create_solver(2023, 99, input)
        .expect("Failed to create solver");

    let answer1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(answer1, "9");

    let answer2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(answer2, "24");
}

// Test combining both macros: AutoRegisterSolver + aoc_solver
struct CombinedMacroSolver;

#[aoc_solver(max_parts = 2)]
impl CombinedMacroSolver {
    type SharedData = Vec<i32>;

    fn parse(input: &str) -> Result<Vec<i32>, ParseError> {
        input
            .lines()
            .map(|line| {
                line.trim()
                    .parse::<i32>()
                    .map_err(|_| ParseError::InvalidFormat("Expected integer".into()))
            })
            .collect()
    }

    fn part1(shared: &mut Vec<i32>) -> String {
        shared.iter().sum::<i32>().to_string()
    }

    fn part2(shared: &mut Vec<i32>) -> String {
        shared.iter().product::<i32>().to_string()
    }
}

// Manually register the solver (since AutoRegisterSolver can't be applied to impl blocks)
aoc_solver::inventory::submit! {
    aoc_solver::SolverPlugin {
        year: 2023,
        day: 100,
        solver: &CombinedMacroSolver,
        tags: &["test", "combined"],
    }
}

#[test]
fn test_both_macros_work_together() {
    // Test that the Solver trait is implemented
    let input = "5\n6\n7";
    let mut cow = <CombinedMacroSolver as Solver>::parse(input).unwrap();
    assert_eq!(*cow, vec![5, 6, 7]);

    let result = CombinedMacroSolver::solve_part(&mut cow, 1).unwrap();
    assert_eq!(result, "18");
}

#[test]
fn test_combined_solver_auto_registers() {
    // Test that the solver is automatically registered via the plugin system
    let registry = RegistryBuilder::new()
        .register_all_plugins()
        .expect("Failed to register plugins")
        .build();

    let input = "5\n6\n7";
    let mut solver = registry
        .create_solver(2023, 100, input)
        .expect("Failed to create solver - was it registered?");

    let answer1 = solver.solve(1).expect("Failed to solve part 1");
    assert_eq!(answer1, "18");

    let answer2 = solver.solve(2).expect("Failed to solve part 2");
    assert_eq!(answer2, "210");
}
