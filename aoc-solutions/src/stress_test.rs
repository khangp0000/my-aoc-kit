//! Stress test solvers - 100 solvers that sleep for random durations
//!
//! These solvers are used to test parallelism without actual computation.
//! Each solver sleeps for a deterministic duration based on year/day.

use aoc_solver::{AocParser, ParseError, SolveError, Solver, SolverPlugin};
use std::thread;
use std::time::Duration;

/// Shared data for stress test solvers - just stores the sleep duration
#[derive(Clone)]
pub struct StressTestData {
    sleep_ms: u64,
}

/// Macro to generate a stress test solver for a specific year/day
macro_rules! stress_solver {
    ($name:ident, $year:expr, $day:expr) => {
        stress_solver_many_parts!($name, $year, $day, 2);
    };
}

/// Macro to generate a stress test solver with configurable number of parts
macro_rules! stress_solver_many_parts {
    ($name:ident, $year:expr, $day:expr, $parts:expr) => {
        pub struct $name;

        impl AocParser for $name {
            type SharedData<'a> = StressTestData;

            fn parse(_input: &str) -> Result<Self::SharedData<'_>, ParseError> {
                // Deterministic sleep duration based on year/day: 10-100ms
                let sleep_ms = 10 + (($year as u64 * 25 + $day as u64) % 91);
                Ok(StressTestData { sleep_ms })
            }
        }

        impl Solver for $name {
            const PARTS: u8 = $parts;

            fn solve_part(
                shared: &mut Self::SharedData<'_>,
                part: u8,
            ) -> Result<String, SolveError> {
                // Sleep for the configured duration
                thread::sleep(Duration::from_millis(shared.sleep_ms));
                // Return a deterministic answer
                Ok(format!("{}_{}_part{}", $year, $day, part))
            }
        }

        aoc_solver::inventory::submit! {
            SolverPlugin {
                year: $year,
                day: $day,
                solver: &$name,
                tags: &["stress-test"],
            }
        }
    };
}

// Generate 100 solvers: years 2015-2018, days 1-25
// Year 2015
stress_solver!(Y2015D01, 2015, 1);
stress_solver!(Y2015D02, 2015, 2);
stress_solver!(Y2015D03, 2015, 3);
stress_solver!(Y2015D04, 2015, 4);
stress_solver!(Y2015D05, 2015, 5);
stress_solver!(Y2015D06, 2015, 6);
stress_solver!(Y2015D07, 2015, 7);
stress_solver!(Y2015D08, 2015, 8);
stress_solver!(Y2015D09, 2015, 9);
stress_solver!(Y2015D10, 2015, 10);
stress_solver!(Y2015D11, 2015, 11);
stress_solver!(Y2015D12, 2015, 12);
stress_solver!(Y2015D13, 2015, 13);
stress_solver!(Y2015D14, 2015, 14);
stress_solver!(Y2015D15, 2015, 15);
stress_solver!(Y2015D16, 2015, 16);
stress_solver!(Y2015D17, 2015, 17);
stress_solver!(Y2015D18, 2015, 18);
stress_solver!(Y2015D19, 2015, 19);
stress_solver!(Y2015D20, 2015, 20);
stress_solver!(Y2015D21, 2015, 21);
stress_solver!(Y2015D22, 2015, 22);
stress_solver!(Y2015D23, 2015, 23);
stress_solver!(Y2015D24, 2015, 24);
stress_solver!(Y2015D25, 2015, 25);

// Year 2016
stress_solver!(Y2016D01, 2016, 1);
stress_solver!(Y2016D02, 2016, 2);
stress_solver!(Y2016D03, 2016, 3);
stress_solver!(Y2016D04, 2016, 4);
stress_solver!(Y2016D05, 2016, 5);
stress_solver!(Y2016D06, 2016, 6);
stress_solver!(Y2016D07, 2016, 7);
stress_solver!(Y2016D08, 2016, 8);
stress_solver!(Y2016D09, 2016, 9);
stress_solver!(Y2016D10, 2016, 10);
stress_solver!(Y2016D11, 2016, 11);
stress_solver!(Y2016D12, 2016, 12);
stress_solver!(Y2016D13, 2016, 13);
stress_solver!(Y2016D14, 2016, 14);
stress_solver!(Y2016D15, 2016, 15);
stress_solver!(Y2016D16, 2016, 16);
stress_solver!(Y2016D17, 2016, 17);
stress_solver!(Y2016D18, 2016, 18);
stress_solver!(Y2016D19, 2016, 19);
stress_solver!(Y2016D20, 2016, 20);
stress_solver!(Y2016D21, 2016, 21);
stress_solver!(Y2016D22, 2016, 22);
stress_solver!(Y2016D23, 2016, 23);
stress_solver!(Y2016D24, 2016, 24);
stress_solver!(Y2016D25, 2016, 25);

// Year 2017
stress_solver!(Y2017D01, 2017, 1);
stress_solver!(Y2017D02, 2017, 2);
stress_solver!(Y2017D03, 2017, 3);
stress_solver!(Y2017D04, 2017, 4);
stress_solver!(Y2017D05, 2017, 5);
stress_solver!(Y2017D06, 2017, 6);
stress_solver!(Y2017D07, 2017, 7);
stress_solver!(Y2017D08, 2017, 8);
stress_solver!(Y2017D09, 2017, 9);
stress_solver!(Y2017D10, 2017, 10);
stress_solver!(Y2017D11, 2017, 11);
stress_solver!(Y2017D12, 2017, 12);
stress_solver!(Y2017D13, 2017, 13);
stress_solver!(Y2017D14, 2017, 14);
stress_solver!(Y2017D15, 2017, 15);
stress_solver!(Y2017D16, 2017, 16);
stress_solver!(Y2017D17, 2017, 17);
stress_solver!(Y2017D18, 2017, 18);
stress_solver!(Y2017D19, 2017, 19);
stress_solver!(Y2017D20, 2017, 20);
stress_solver!(Y2017D21, 2017, 21);
stress_solver!(Y2017D22, 2017, 22);
stress_solver!(Y2017D23, 2017, 23);
stress_solver!(Y2017D24, 2017, 24);
stress_solver!(Y2017D25, 2017, 25);

// Year 2018 - Day 1 has 50 parts to test many-part support
stress_solver_many_parts!(Y2018D01, 2018, 1, 50);
stress_solver!(Y2018D02, 2018, 2);
stress_solver!(Y2018D03, 2018, 3);
stress_solver!(Y2018D04, 2018, 4);
stress_solver!(Y2018D05, 2018, 5);
stress_solver!(Y2018D06, 2018, 6);
stress_solver!(Y2018D07, 2018, 7);
stress_solver!(Y2018D08, 2018, 8);
stress_solver!(Y2018D09, 2018, 9);
stress_solver!(Y2018D10, 2018, 10);
stress_solver!(Y2018D11, 2018, 11);
stress_solver!(Y2018D12, 2018, 12);
stress_solver!(Y2018D13, 2018, 13);
stress_solver!(Y2018D14, 2018, 14);
stress_solver!(Y2018D15, 2018, 15);
stress_solver!(Y2018D16, 2018, 16);
stress_solver!(Y2018D17, 2018, 17);
stress_solver!(Y2018D18, 2018, 18);
stress_solver!(Y2018D19, 2018, 19);
stress_solver!(Y2018D20, 2018, 20);
stress_solver!(Y2018D21, 2018, 21);
stress_solver!(Y2018D22, 2018, 22);
stress_solver!(Y2018D23, 2018, 23);
stress_solver!(Y2018D24, 2018, 24);
stress_solver!(Y2018D25, 2018, 25);
