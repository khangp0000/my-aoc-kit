//! Result aggregator for ordering parallel solver results
//!
//! Buffers and orders results for streaming output using two min-heaps:
//! - One for expected keys (what we're waiting for)
//! - One for received results (buffered until their turn)

use crate::executor::SolverResult;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Key for ordering results (year, day, part) - ordered ascending
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct ResultKey {
    pub year: u16,
    pub day: u8,
    pub part: u8,
}

impl From<&SolverResult> for ResultKey {
    fn from(r: &SolverResult) -> Self {
        Self {
            year: r.year,
            day: r.day,
            part: r.part,
        }
    }
}

/// Wrapper for min-heap ordering of SolverResult
struct OrderedResult(SolverResult);

impl Ord for OrderedResult {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap (smallest first)
        ResultKey::from(&other.0).cmp(&ResultKey::from(&self.0))
    }
}

impl PartialOrd for OrderedResult {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for OrderedResult {}

impl PartialEq for OrderedResult {
    fn eq(&self, other: &Self) -> bool {
        ResultKey::from(&self.0) == ResultKey::from(&other.0)
    }
}

/// Aggregator that buffers results and emits them in sorted order
pub struct ResultAggregator {
    /// Min-heap of expected keys (next to output is at top)
    expected: BinaryHeap<Reverse<ResultKey>>,
    /// Min-heap of received results waiting to be output
    pending: BinaryHeap<OrderedResult>,
}

impl ResultAggregator {
    /// Create aggregator from list of expected keys
    pub fn new(expected_keys: Vec<ResultKey>) -> Self {
        Self {
            expected: expected_keys.into_iter().map(Reverse).collect(),
            pending: BinaryHeap::new(),
        }
    }

    /// Add a result and return any results ready for output (in order)
    pub fn add(&mut self, result: SolverResult) -> Vec<SolverResult> {
        self.pending.push(OrderedResult(result));

        // Emit results while pending min matches expected min
        let mut ready = Vec::new();
        while let (Some(Reverse(next_expected)), Some(top_pending)) =
            (self.expected.peek(), self.pending.peek())
        {
            if ResultKey::from(&top_pending.0) == *next_expected {
                self.expected.pop();
                ready.push(self.pending.pop().unwrap().0);
            } else {
                break;
            }
        }
        ready
    }

    /// Drain remaining results in order (for final output)
    pub fn drain(&mut self) -> Vec<SolverResult> {
        let mut results: Vec<_> = self.pending.drain().map(|o| o.0).collect();
        results.sort_by_key(|r| ResultKey::from(r));
        results
    }

    /// Check if all expected results have been received
    pub fn is_complete(&self) -> bool {
        self.expected.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeDelta;

    fn make_result(year: u16, day: u8, part: u8) -> SolverResult {
        SolverResult {
            year,
            day,
            part,
            answer: Ok(format!("{}_{}_{}", year, day, part)),
            solve_duration: TimeDelta::milliseconds(10),
            parse_duration: Some(TimeDelta::milliseconds(5)),
            submitted_at: None,
            submission: None,
            submission_wait: None,
        }
    }

    #[test]
    fn test_in_order_results() {
        let keys = vec![
            ResultKey {
                year: 2015,
                day: 1,
                part: 1,
            },
            ResultKey {
                year: 2015,
                day: 1,
                part: 2,
            },
        ];
        let mut agg = ResultAggregator::new(keys);

        // Add in order
        let ready = agg.add(make_result(2015, 1, 1));
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].part, 1);

        let ready = agg.add(make_result(2015, 1, 2));
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].part, 2);

        assert!(agg.is_complete());
    }

    #[test]
    fn test_out_of_order_results() {
        let keys = vec![
            ResultKey {
                year: 2015,
                day: 1,
                part: 1,
            },
            ResultKey {
                year: 2015,
                day: 1,
                part: 2,
            },
            ResultKey {
                year: 2015,
                day: 2,
                part: 1,
            },
        ];
        let mut agg = ResultAggregator::new(keys);

        // Add out of order - part 2 before part 1
        let ready = agg.add(make_result(2015, 1, 2));
        assert!(ready.is_empty()); // Waiting for part 1

        let ready = agg.add(make_result(2015, 2, 1));
        assert!(ready.is_empty()); // Still waiting for 2015/1/1

        // Now add the missing one
        let ready = agg.add(make_result(2015, 1, 1));
        assert_eq!(ready.len(), 3); // All three should be ready now
        assert_eq!(ready[0].part, 1);
        assert_eq!(ready[0].day, 1);
        assert_eq!(ready[1].part, 2);
        assert_eq!(ready[1].day, 1);
        assert_eq!(ready[2].part, 1);
        assert_eq!(ready[2].day, 2);
    }

    #[test]
    fn test_drain_remaining() {
        let keys = vec![
            ResultKey {
                year: 2015,
                day: 1,
                part: 1,
            },
            ResultKey {
                year: 2015,
                day: 1,
                part: 2,
            },
        ];
        let mut agg = ResultAggregator::new(keys);

        // Add only part 2 (out of order)
        agg.add(make_result(2015, 1, 2));

        // Drain should return it
        let remaining = agg.drain();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].part, 2);
    }
}
