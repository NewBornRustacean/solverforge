use super::*;
use solverforge_core::score::SoftScore;

#[derive(Clone, Debug)]
struct TestSolution {
    score: Option<SoftScore>,
}

impl PlanningSolution for TestSolution {
    type Score = SoftScore;

    fn score(&self) -> Option<Self::Score> {
        self.score
    }

    fn set_score(&mut self, score: Option<Self::Score>) {
        self.score = score;
    }
}

fn set_row(_s: &mut TestSolution, _idx: usize, _v: Option<i32>) {}

#[test]
fn test_simple_decider_creation() {
    let decider: SimpleDecider<TestSolution, i32> =
        SimpleDecider::new(0, "row", vec![0, 1, 2, 3], set_row);

    let debug = format!("{:?}", decider);
    assert!(debug.contains("SimpleDecider"));
    assert!(debug.contains("value_count: 4"));
}
