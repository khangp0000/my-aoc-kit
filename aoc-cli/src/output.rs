//! Output formatting for solver results

use crate::executor::{SolverResult, SubmissionOutcome};
use chrono::TimeDelta;

/// Output formatter for solver results
pub struct OutputFormatter {
    quiet: bool,
    start_time: std::time::Instant,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet,
            start_time: std::time::Instant::now(),
        }
    }

    /// Format and print a single result
    pub fn print_result(&self, result: &SolverResult) {
        if self.quiet {
            self.print_quiet(result);
        } else {
            self.print_full(result);
        }
    }

    /// Print in quiet mode (just the answer)
    fn print_quiet(&self, result: &SolverResult) {
        match &result.answer {
            Ok(answer) => println!("{}", answer),
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    /// Print full output with timing and submission info
    fn print_full(&self, result: &SolverResult) {
        let prefix = format!("{}/{:02} Part {}", result.year, result.day, result.part);

        match &result.answer {
            Ok(answer) => {
                let parse_timing = result
                    .parse_duration
                    .map(|d| format!("parse: {}, ", format_duration(d)))
                    .unwrap_or_default();
                let solve_timing = format_duration(result.solve_duration);

                let submission_info = match &result.submission {
                    Some(outcome) => {
                        let time_str = result
                            .submitted_at
                            .map(|t| t.format("%H:%M:%S").to_string())
                            .unwrap_or_default();
                        format!(", submitted {}: {}", time_str, format_outcome(outcome))
                    }
                    None => String::new(),
                };

                println!(
                    "{}: {} ({}solve: {}{})",
                    prefix, answer, parse_timing, solve_timing, submission_info
                );
            }
            Err(e) => {
                eprintln!("{}: Error - {}", prefix, e);
            }
        }
    }

    /// Print a summary after all results
    /// Shows both total solve time (sum of durations) and actual elapsed wall-clock time
    pub fn print_summary(&self, results: &[SolverResult]) {
        if self.quiet {
            return;
        }

        let total = results.len();
        let successes = results.iter().filter(|r| r.answer.is_ok()).count();
        let failures = total - successes;

        let total_parse_time: TimeDelta = results
            .iter()
            .filter(|r| r.answer.is_ok())
            .filter_map(|r| r.parse_duration)
            .sum();
        let total_solve_time: TimeDelta = results
            .iter()
            .filter(|r| r.answer.is_ok())
            .map(|r| r.solve_duration)
            .sum();
        let total_compute_time = total_parse_time + total_solve_time;
        let elapsed_time = self.start_time.elapsed();

        println!();
        println!("--- Summary ---");
        println!("Solvers: {} solved, {} failed", successes, failures);
        println!("Total parse time: {}", format_duration(total_parse_time));
        println!("Total solve time: {}", format_duration(total_solve_time));
        println!(
            "Elapsed wall-clock time: {}",
            format_std_duration(elapsed_time)
        );
        if !elapsed_time.is_zero() {
            let total_compute_secs =
                total_compute_time.num_microseconds().unwrap_or(0) as f64 / 1_000_000.0;
            let speedup = total_compute_secs / elapsed_time.as_secs_f64();
            println!("Speedup factor: {:.2}x", speedup);
        }
    }
}

/// Format a TimeDelta for display
fn format_duration(d: TimeDelta) -> String {
    let Some(micros) = d.num_microseconds() else {
        return "N/A".to_string();
    };

    if micros < 0 {
        return format!("-{}", format_duration(-d));
    }

    if micros < 1000 {
        format!("{}µs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1000.0)
    } else {
        format!("{:.2}s", micros as f64 / 1_000_000.0)
    }
}

/// Format a std::time::Duration for display (used for wall-clock time)
fn format_std_duration(d: std::time::Duration) -> String {
    let micros = d.as_micros();
    if micros < 1000 {
        format!("{}µs", micros)
    } else if micros < 1_000_000 {
        format!("{:.2}ms", micros as f64 / 1000.0)
    } else {
        format!("{:.2}s", d.as_secs_f64())
    }
}

/// Format a submission outcome for display
fn format_outcome(outcome: &SubmissionOutcome) -> String {
    match outcome {
        SubmissionOutcome::Correct => "✓ Correct".to_string(),
        SubmissionOutcome::Incorrect => "✗ Incorrect".to_string(),
        SubmissionOutcome::AlreadyCompleted => "⏭ Already completed".to_string(),
        SubmissionOutcome::Throttled { wait_time } => match wait_time {
            Some(d) => format!("⏳ Throttled (wait {})", format_duration(*d)),
            None => "⏳ Throttled".to_string(),
        },
        SubmissionOutcome::Error(msg) => format!("⚠ Error: {}", msg),
    }
}
