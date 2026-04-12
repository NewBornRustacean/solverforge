use super::*;
use solverforge_core::score::SoftScore;
use solverforge_scoring::Director;

#[derive(Clone, Debug)]
struct TestSolution {
    value: i32,
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

#[derive(Debug)]
struct IncrementValue;

impl ProblemChange<TestSolution> for IncrementValue {
    fn apply(&self, score_director: &mut dyn Director<TestSolution>) {
        score_director.working_solution_mut().value += 1;
    }
}

#[test]
fn handle_creation() {
    let (handle, _rx) = SolverHandle::<TestSolution>::new();
    assert!(!handle.is_solving());
}

#[test]
fn submit_change_when_solving() {
    let (handle, rx) = SolverHandle::<TestSolution>::new();
    handle.set_solving(true);

    let result = handle.add_problem_change(IncrementValue);
    assert_eq!(result, ProblemChangeResult::Queued);

    let changes = rx.drain_pending();
    assert_eq!(changes.len(), 1);
}

#[test]
fn submit_change_when_not_solving() {
    let (handle, _rx) = SolverHandle::<TestSolution>::new();

    let result = handle.add_problem_change(IncrementValue);
    assert_eq!(result, ProblemChangeResult::SolverNotRunning);
}

#[test]
fn multiple_changes() {
    let (handle, rx) = SolverHandle::<TestSolution>::new();
    handle.set_solving(true);

    handle.add_problem_change(IncrementValue);
    handle.add_problem_change(IncrementValue);
    handle.add_problem_change(IncrementValue);

    let changes = rx.drain_pending();
    assert_eq!(changes.len(), 3);
}

#[test]
fn terminate_early() {
    let (handle, rx) = SolverHandle::<TestSolution>::new();

    assert!(!rx.is_terminate_early_requested());
    handle.terminate_early();
    assert!(rx.is_terminate_early_requested());

    rx.clear_terminate_early();
    assert!(!rx.is_terminate_early_requested());
}

#[test]
fn handle_clone() {
    let (handle1, rx) = SolverHandle::<TestSolution>::new();
    let handle2 = handle1.clone();

    handle1.set_solving(true);
    assert!(handle2.is_solving());

    handle2.add_problem_change(IncrementValue);
    let changes = rx.drain_pending();
    assert_eq!(changes.len(), 1);
}
