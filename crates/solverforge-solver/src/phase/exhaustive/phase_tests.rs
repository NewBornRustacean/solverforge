use super::*;
use crate::phase::exhaustive::decider::SimpleDecider;
use crate::phase::exhaustive::exploration_type::ExplorationType;
use solverforge_core::domain::PlanningSolution;
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
fn test_exploration_type_display() {
    assert_eq!(format!("{}", ExplorationType::DepthFirst), "DepthFirst");
    assert_eq!(format!("{}", ExplorationType::BreadthFirst), "BreadthFirst");
    assert_eq!(format!("{}", ExplorationType::ScoreFirst), "ScoreFirst");
    assert_eq!(
        format!("{}", ExplorationType::OptimisticBoundFirst),
        "OptimisticBoundFirst"
    );
}

#[test]
fn test_exploration_type_default() {
    assert_eq!(ExplorationType::default(), ExplorationType::DepthFirst);
}

#[test]
fn test_config_default() {
    let config = ExhaustiveSearchConfig::default();
    assert_eq!(config.exploration_type, ExplorationType::DepthFirst);
    assert_eq!(config.node_limit, Some(10_000));
    assert!(config.depth_limit.is_none());
    assert!(config.enable_pruning);
}

#[test]
fn test_phase_type_name() {
    let decider: SimpleDecider<TestSolution, i32> =
        SimpleDecider::new(0, "row", vec![0, 1, 2, 3], set_row);
    let phase = ExhaustiveSearchPhase::depth_first(decider);

    assert_eq!(phase.phase_type_name(), "ExhaustiveSearch");
}

#[test]
fn test_phase_debug() {
    let decider: SimpleDecider<TestSolution, i32> =
        SimpleDecider::new(0, "row", vec![0, 1, 2, 3], set_row);
    let phase = ExhaustiveSearchPhase::depth_first(decider);

    let debug = format!("{:?}", phase);
    assert!(debug.contains("ExhaustiveSearchPhase"));
    assert!(debug.contains("DepthFirst"));
}
