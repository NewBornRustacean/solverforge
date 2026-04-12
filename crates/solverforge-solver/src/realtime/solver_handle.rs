// Solver handle for submitting problem changes during solving.

use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};
use std::sync::Arc;

use solverforge_core::domain::PlanningSolution;

use super::problem_change::BoxedProblemChange;
use super::ProblemChange;

// Result of a problem change submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemChangeResult {
    // Change was successfully queued.
    Queued,
    // Solver is not running, change was not queued.
    SolverNotRunning,
    // Change queue is full (solver is processing slowly).
    QueueFull,
}

/// Handle for interacting with a running solver.
///
/// The solver handle allows submitting problem changes to a solver
/// while it is running. Changes are queued and processed at step
/// boundaries.
///
/// # Example
///
/// ```
/// use solverforge_solver::realtime::{SolverHandle, ProblemChange, ProblemChangeResult};
/// use solverforge_scoring::Director;
/// use solverforge_core::domain::PlanningSolution;
/// use solverforge_core::score::SoftScore;
///
/// #[derive(Clone, Debug)]
/// struct Task { id: usize }
///
/// #[derive(Clone, Debug)]
/// struct Solution {
///     tasks: Vec<Task>,
///     score: Option<SoftScore>,
/// }
///
/// impl PlanningSolution for Solution {
///     type Score = SoftScore;
///     fn score(&self) -> Option<Self::Score> { self.score }
///     fn set_score(&mut self, score: Option<Self::Score>) { self.score = score; }
/// }
///
/// #[derive(Debug)]
/// struct AddTask { id: usize }
///
/// impl ProblemChange<Solution> for AddTask {
///     fn apply(&self, sd: &mut dyn Director<Solution>) {
///         sd.working_solution_mut().tasks.push(Task { id: self.id });
///     }
/// }
///
/// // Create a handle (normally obtained from RealtimeSolver)
/// let (handle, _rx) = SolverHandle::<Solution>::new();
///
/// // Submit a change while solver is "running"
/// handle.set_solving(true);
/// let result = handle.add_problem_change(AddTask { id: 42 });
/// assert_eq!(result, ProblemChangeResult::Queued);
///
/// // When solver stops, changes are rejected
/// handle.set_solving(false);
/// let result = handle.add_problem_change(AddTask { id: 43 });
/// assert_eq!(result, ProblemChangeResult::SolverNotRunning);
/// ```
pub struct SolverHandle<S: PlanningSolution> {
    // Channel for sending problem changes to the solver.
    change_tx: Sender<BoxedProblemChange<S>>,
    // Flag indicating whether solver is currently running.
    solving: Arc<AtomicBool>,
    // Flag to request early termination.
    terminate_early: Arc<AtomicBool>,
}

impl<S: PlanningSolution> SolverHandle<S> {
    /// Creates a new solver handle and its corresponding receiver.
    ///
    /// The receiver should be passed to the solver to receive changes.
    pub fn new() -> (Self, ProblemChangeReceiver<S>) {
        let (tx, rx) = mpsc::channel();
        let solving = Arc::new(AtomicBool::new(false));
        let terminate_early = Arc::new(AtomicBool::new(false));

        let handle = Self {
            change_tx: tx,
            solving: Arc::clone(&solving),
            terminate_early: Arc::clone(&terminate_early),
        };

        let receiver = ProblemChangeReceiver {
            change_rx: rx,
            solving,
            terminate_early,
        };

        (handle, receiver)
    }

    /// Submits a problem change to the solver.
    ///
    /// The change is queued and will be processed at the next step boundary.
    pub fn add_problem_change<P: ProblemChange<S> + 'static>(
        &self,
        change: P,
    ) -> ProblemChangeResult {
        if !self.solving.load(Ordering::SeqCst) {
            return ProblemChangeResult::SolverNotRunning;
        }

        match self.change_tx.send(Box::new(change)) {
            Ok(()) => ProblemChangeResult::Queued,
            Err(_) => ProblemChangeResult::QueueFull,
        }
    }

    /// Submits a boxed problem change to the solver.
    pub fn add_problem_change_boxed(&self, change: BoxedProblemChange<S>) -> ProblemChangeResult {
        if !self.solving.load(Ordering::SeqCst) {
            return ProblemChangeResult::SolverNotRunning;
        }

        match self.change_tx.send(change) {
            Ok(()) => ProblemChangeResult::Queued,
            Err(_) => ProblemChangeResult::QueueFull,
        }
    }

    pub fn is_solving(&self) -> bool {
        self.solving.load(Ordering::SeqCst)
    }

    /// Requests early termination of the solver.
    ///
    /// The solver will stop at the next step boundary.
    pub fn terminate_early(&self) {
        self.terminate_early.store(true, Ordering::SeqCst);
    }

    pub fn set_solving(&self, solving: bool) {
        self.solving.store(solving, Ordering::SeqCst);
    }
}

impl<S: PlanningSolution> Clone for SolverHandle<S> {
    fn clone(&self) -> Self {
        Self {
            change_tx: self.change_tx.clone(),
            solving: Arc::clone(&self.solving),
            terminate_early: Arc::clone(&self.terminate_early),
        }
    }
}

impl<S: PlanningSolution> Debug for SolverHandle<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SolverHandle")
            .field("solving", &self.solving.load(Ordering::SeqCst))
            .field(
                "terminate_early",
                &self.terminate_early.load(Ordering::SeqCst),
            )
            .finish()
    }
}

/// Receiver for problem changes, used by the solver.
pub struct ProblemChangeReceiver<S: PlanningSolution> {
    // Channel for receiving problem changes.
    change_rx: Receiver<BoxedProblemChange<S>>,
    // Shared solving flag.
    solving: Arc<AtomicBool>,
    // Shared terminate early flag.
    terminate_early: Arc<AtomicBool>,
}

impl<S: PlanningSolution> ProblemChangeReceiver<S> {
    /// Tries to receive a pending problem change without blocking.
    ///
    /// Returns `Some(change)` if a change is available, `None` otherwise.
    pub fn try_recv(&self) -> Option<BoxedProblemChange<S>> {
        match self.change_rx.try_recv() {
            Ok(change) => Some(change),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Receives all pending problem changes without blocking.
    ///
    pub fn drain_pending(&self) -> Vec<BoxedProblemChange<S>> {
        let mut changes = Vec::new();
        while let Some(change) = self.try_recv() {
            changes.push(change);
        }
        changes
    }

    pub fn is_terminate_early_requested(&self) -> bool {
        self.terminate_early.load(Ordering::SeqCst)
    }

    pub fn set_solving(&self, solving: bool) {
        self.solving.store(solving, Ordering::SeqCst);
    }

    /// Clears the terminate early flag.
    pub fn clear_terminate_early(&self) {
        self.terminate_early.store(false, Ordering::SeqCst);
    }
}

impl<S: PlanningSolution> Debug for ProblemChangeReceiver<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProblemChangeReceiver")
            .field("solving", &self.solving.load(Ordering::SeqCst))
            .finish()
    }
}

#[cfg(test)]
#[path = "solver_handle_tests.rs"]
mod tests;
