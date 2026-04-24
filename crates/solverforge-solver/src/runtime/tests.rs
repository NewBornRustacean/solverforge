use super::Construction;
use crate::builder::{
    ListVariableContext, ModelContext, ScalarVariableContext, ValueSource, VariableContext,
};
use crate::descriptor_scalar::{scalar_target_matches, scalar_work_remaining_with_frontier};
use crate::phase::Phase;
use crate::scope::SolverScope;
use crate::DefaultCrossEntityDistanceMeter;
use solverforge_config::{
    ConstructionHeuristicConfig, ConstructionHeuristicType, VariableTargetConfig,
};
use solverforge_core::domain::{
    EntityCollectionExtractor, EntityDescriptor, PlanningSolution, ProblemFactDescriptor,
    SolutionDescriptor, VariableDescriptor, VariableType,
};
use solverforge_core::score::SoftScore;
use solverforge_scoring::{Director, ScoreDirector};
use std::any::TypeId;

include!("tests/target_matching.rs");
include!("tests/scalar_runtime.rs");
include!("tests/queue_runtime.rs");
include!("tests/revision_runtime.rs");
include!("tests/multi_owner_runtime.rs");
include!("tests/mixed_target_runtime.rs");
