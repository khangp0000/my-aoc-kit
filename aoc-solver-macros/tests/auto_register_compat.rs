use aoc_solver::{ParseError, RegistryBuilder, Solver};
use aoc_solver_macros::aoc_solver;

struct TestSolver1;

// Test that aoc_solver macro works
#[aoc_solver(max_parts = 2)]
impl TestSolver1 {
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
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
    
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    fn part2(parsed: &Vec<i32>) -> String {
        parsed.iter().product::<i32>().to_string()
    }
}

// Test that we can manually register a solver created with aoc_solver
#[test]
fn test_aoc_solver_with_manual_registration() {
    let input = "2\n3\n4";
    let parsed = TestSolver1::parse(input).unwrap();
    assert_eq!(parsed, vec![2, 3, 4]);
    
    // Test Solver trait is implemented
    let result = TestSolver1::solve_part(&parsed, 1, None).unwrap();
    assert_eq!(result.answer, "9");
}

#[test]
fn test_solver_can_be_registered_manually() {
    // Test that the solver can be manually registered and used via the registry
    let mut builder = RegistryBuilder::new();
    aoc_solver::register_solver!(builder, TestSolver1, 2023, 99);
    let registry = builder.build();
    
    let input = "2\n3\n4";
    let mut solver = registry.create_solver(2023, 99, input)
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
    type Parsed = Vec<i32>;
    type PartialResult = ();
    
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
    
    fn part1(parsed: &Vec<i32>) -> String {
        parsed.iter().sum::<i32>().to_string()
    }
    
    fn part2(parsed: &Vec<i32>) -> String {
        parsed.iter().product::<i32>().to_string()
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
    let parsed = CombinedMacroSolver::parse(input).unwrap();
    assert_eq!(parsed, vec![5, 6, 7]);
    
    let result = CombinedMacroSolver::solve_part(&parsed, 1, None).unwrap();
    assert_eq!(result.answer, "18");
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
