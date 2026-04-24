use super::*;
use std::any::TypeId;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use solverforge_core::domain::SolutionDescriptor;
use solverforge_core::score::SoftScore;
use solverforge_scoring::Director;

use crate::heuristic::selector::EntityReference;
use crate::heuristic::selector::{FromSolutionEntitySelector, StaticValueSelector};
use crate::manager::{
    Solvable, SolverEvent, SolverLifecycleState, SolverManager, SolverRuntime, SolverTerminalReason,
};
use crate::phase::construction::{
    BestFitForager, FirstFeasibleForager, FirstFitForager, Placement, QueuedEntityPlacer,
};
use crate::test_utils::{
    create_simple_nqueens_director, get_queen_row, set_queen_row, NQueensSolution,
};

include!("tests/support.rs");
include!("tests/selection.rs");
include!("tests/lifecycle.rs");
