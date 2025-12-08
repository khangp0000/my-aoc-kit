use std::str::FromStr;
use anyhow::anyhow;
use aoc_solver::{AocParser, ParseError, PartSolver, SolveError};
use aoc_solver_macros::{AocSolver, AutoRegisterSolver};

#[derive(AocSolver, AutoRegisterSolver)]
#[aoc_solver(max_parts = 2)]
#[aoc(year = 2025, day = 1, tags = ["khangp0000", "wip"])]
pub struct Solver;

#[derive(Debug)]
pub struct SharedData {
    parsed_input: Vec<i16>,
    common_result: Option<CommonResult>,
}

#[derive(Debug)]
pub struct CommonResult {
    zero_counts: u16,
    pass_zero_counts: u16,
}

impl AocParser for Solver {
    type SharedData<'a> = SharedData;

    fn parse<'a>(input: &'a str) -> Result<Self::SharedData<'a>, ParseError> {
        input.trim()
            .lines()
            .map(|line| -> Result<i16, anyhow::Error> {
                let negative = match line.as_bytes().first() {
                    Some(b'L') => true,
                    Some(b'R') => false,
                    _ => return Err(anyhow!("first character need to be 'L' or 'R'")),
                };

                <i16 as FromStr>::from_str(&line[1..])
                    .map_err(anyhow::Error::from)
                    .and_then(|val| {
                        if val < 0 {
                            Err(anyhow!("Rotate value must be non negative"))
                        } else if negative {
                            Ok(-val)
                        } else {
                            Ok(val)
                        }
                    })
            }).enumerate()
            .map(|(line_idx, line_val_res)| line_val_res.map_err(|e| anyhow!("(line {}) {}", line_idx + 1, e)))
            .try_fold(Vec::new(), |mut vec, line_val_res| {
                line_val_res.map(|val| {
                    vec.push(val);
                    vec
                })
            })
            .map(|vec| SharedData {
                parsed_input: vec,
                common_result: None,
            })
            .map_err(|e| ParseError::InvalidFormat(e.to_string()))
    }
}

impl PartSolver<1> for Solver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(solve_once_for_both(shared).zero_counts.to_string())
    }
}

impl PartSolver<2> for Solver {
    fn solve(shared: &mut Self::SharedData<'_>) -> Result<String, SolveError> {
        Ok(solve_once_for_both(shared).pass_zero_counts.to_string())
    }
}

fn solve_once_for_both(shared: &mut SharedData) -> &CommonResult {
    shared.common_result.get_or_insert_with(|| {
        let (_, zero_counts, pass_zero_counts) = shared.parsed_input.iter()
            .fold((50i16, 0u16, 0_u16), |(mut dial_value, mut zero_counts, mut pass_zero_counts), rotate_val| {
                let old_dial_value = dial_value;
                dial_value += rotate_val;
                if dial_value <= 0 && old_dial_value != 0 {
                    pass_zero_counts += 1;
                }
                pass_zero_counts += (dial_value / 100).unsigned_abs();
                dial_value %= 100;
                if dial_value < 0 {
                    dial_value += 100;
                }
                if dial_value == 0 {
                    zero_counts += 1;
                }
                (dial_value, zero_counts, pass_zero_counts)
            });

        CommonResult {
            zero_counts,
            pass_zero_counts
        }
    })
}