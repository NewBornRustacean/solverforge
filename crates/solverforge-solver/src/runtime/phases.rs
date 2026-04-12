use std::fmt::{self, Debug};
use std::hash::Hash;

use solverforge_config::{ConstructionHeuristicConfig, PhaseConfig, SolverConfig};
use solverforge_core::domain::{PlanningSolution, SolutionDescriptor};
use solverforge_core::score::{ParseableScore, Score};

use super::construction::{ListConstructionArgs, UnifiedConstruction};
use crate::builder::ListContext;
use crate::heuristic::selector::nearby_list_change::CrossEntityDistanceMeter;
use crate::phase::{sequence::PhaseSequence, Phase};
use crate::unified_search::{
    build_unified_local_search, build_unified_vnd, UnifiedLocalSearch, UnifiedVnd,
};

pub enum RuntimePhase<C, LS, VND> {
    Construction(C),
    LocalSearch(LS),
    Vnd(VND),
}

impl<C, LS, VND> Debug for RuntimePhase<C, LS, VND>
where
    C: Debug,
    LS: Debug,
    VND: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Construction(phase) => write!(f, "RuntimePhase::Construction({phase:?})"),
            Self::LocalSearch(phase) => write!(f, "RuntimePhase::LocalSearch({phase:?})"),
            Self::Vnd(phase) => write!(f, "RuntimePhase::Vnd({phase:?})"),
        }
    }
}

impl<S, D, ProgressCb, C, LS, VND> Phase<S, D, ProgressCb> for RuntimePhase<C, LS, VND>
where
    S: PlanningSolution,
    D: solverforge_scoring::Director<S>,
    ProgressCb: crate::scope::ProgressCallback<S>,
    C: Phase<S, D, ProgressCb> + Debug,
    LS: Phase<S, D, ProgressCb> + Debug,
    VND: Phase<S, D, ProgressCb> + Debug,
{
    fn solve(&mut self, solver_scope: &mut crate::scope::SolverScope<'_, S, D, ProgressCb>) {
        match self {
            Self::Construction(phase) => phase.solve(solver_scope),
            Self::LocalSearch(phase) => phase.solve(solver_scope),
            Self::Vnd(phase) => phase.solve(solver_scope),
        }
    }

    fn phase_type_name(&self) -> &'static str {
        "RuntimePhase"
    }
}

pub type UnifiedRuntimePhase<S, V, DM, IDM> = RuntimePhase<
    UnifiedConstruction<S, V>,
    UnifiedLocalSearch<S, V, DM, IDM>,
    UnifiedVnd<S, V, DM, IDM>,
>;

pub fn build_phases<S, V, DM, IDM>(
    config: &SolverConfig,
    descriptor: &SolutionDescriptor,
    list_ctx: Option<&ListContext<S, V, DM, IDM>>,
    list_construction: Option<ListConstructionArgs<S, V>>,
    list_variable_name: Option<&'static str>,
) -> PhaseSequence<UnifiedRuntimePhase<S, V, DM, IDM>>
where
    S: PlanningSolution + 'static,
    S::Score: Score + ParseableScore,
    V: Clone + Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    let mut phases = Vec::new();

    if config.phases.is_empty() {
        phases.push(default_construction_phase(
            descriptor,
            list_construction.as_ref(),
            list_variable_name,
        ));
        phases.push(RuntimePhase::LocalSearch(build_unified_local_search(
            None,
            descriptor,
            list_ctx,
            config.random_seed,
        )));
        return PhaseSequence::new(phases);
    }

    for phase in &config.phases {
        match phase {
            PhaseConfig::ConstructionHeuristic(ch) => {
                phases.push(build_construction_phase(
                    ch,
                    descriptor,
                    list_construction.as_ref(),
                    list_variable_name,
                ));
            }
            PhaseConfig::LocalSearch(ls) => {
                phases.push(RuntimePhase::LocalSearch(build_unified_local_search(
                    Some(ls),
                    descriptor,
                    list_ctx,
                    config.random_seed,
                )));
            }
            PhaseConfig::Vnd(vnd) => {
                phases.push(RuntimePhase::Vnd(build_unified_vnd(
                    vnd,
                    descriptor,
                    list_ctx,
                    config.random_seed,
                )));
            }
            _ => {
                panic!("unsupported phase in the unified runtime");
            }
        }
    }

    PhaseSequence::new(phases)
}

fn default_construction_phase<S, V, DM, IDM>(
    descriptor: &SolutionDescriptor,
    list_construction: Option<&ListConstructionArgs<S, V>>,
    list_variable_name: Option<&'static str>,
) -> UnifiedRuntimePhase<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    S::Score: Score + ParseableScore,
    V: Clone + Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    RuntimePhase::Construction(UnifiedConstruction::new(
        None,
        descriptor.clone(),
        list_construction.copied(),
        list_variable_name,
    ))
}

fn build_construction_phase<S, V, DM, IDM>(
    config: &ConstructionHeuristicConfig,
    descriptor: &SolutionDescriptor,
    list_construction: Option<&ListConstructionArgs<S, V>>,
    list_variable_name: Option<&'static str>,
) -> UnifiedRuntimePhase<S, V, DM, IDM>
where
    S: PlanningSolution + 'static,
    S::Score: Score + ParseableScore,
    V: Clone + Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
    DM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
    IDM: CrossEntityDistanceMeter<S> + Clone + Debug + 'static,
{
    RuntimePhase::Construction(UnifiedConstruction::new(
        Some(config.clone()),
        descriptor.clone(),
        list_construction.copied(),
        list_variable_name,
    ))
}
