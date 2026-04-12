use std::fmt::{self, Debug};
use std::hash::Hash;

use solverforge_config::{ConstructionHeuristicConfig, ConstructionHeuristicType};
use solverforge_core::domain::{PlanningSolution, SolutionDescriptor};

use crate::descriptor_standard::{
    build_descriptor_construction, standard_target_matches, standard_work_remaining,
};
use crate::list_solver::build_list_construction;
use crate::phase::Phase;
use crate::scope::{ProgressCallback, SolverScope};

pub struct UnifiedConstruction<S, V>
where
    S: PlanningSolution,
    V: Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + 'static,
{
    config: Option<ConstructionHeuristicConfig>,
    descriptor: SolutionDescriptor,
    list_construction: Option<ListConstructionArgs<S, V>>,
    list_variable_name: Option<&'static str>,
}

impl<S, V> UnifiedConstruction<S, V>
where
    S: PlanningSolution,
    V: Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + 'static,
{
    pub(super) fn new(
        config: Option<ConstructionHeuristicConfig>,
        descriptor: SolutionDescriptor,
        list_construction: Option<ListConstructionArgs<S, V>>,
        list_variable_name: Option<&'static str>,
    ) -> Self {
        Self {
            config,
            descriptor,
            list_construction,
            list_variable_name,
        }
    }
}

impl<S, V> Debug for UnifiedConstruction<S, V>
where
    S: PlanningSolution,
    V: Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnifiedConstruction")
            .field("config", &self.config)
            .field("has_list_construction", &self.list_construction.is_some())
            .field("list_variable_name", &self.list_variable_name)
            .finish()
    }
}

impl<S, V, D, ProgressCb> Phase<S, D, ProgressCb> for UnifiedConstruction<S, V>
where
    S: PlanningSolution + 'static,
    V: Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
    D: solverforge_scoring::Director<S>,
    ProgressCb: ProgressCallback<S>,
{
    fn solve(&mut self, solver_scope: &mut SolverScope<'_, S, D, ProgressCb>) {
        let config = self.config.as_ref();
        let explicit_target = config.is_some_and(has_explicit_target);
        let entity_class = config.and_then(|cfg| cfg.target.entity_class.as_deref());
        let variable_name = config.and_then(|cfg| cfg.target.variable_name.as_deref());
        let standard_matches = config.is_some_and(|_| {
            standard_target_matches(&self.descriptor, entity_class, variable_name)
        });
        let list_matches = config.is_some_and(|cfg| {
            list_target_matches(
                cfg,
                &self.descriptor,
                self.list_construction.as_ref(),
                self.list_variable_name,
            )
        });

        if let Some(cfg) = config {
            if explicit_target && !standard_matches && !list_matches {
                panic!(
                    "construction heuristic matched no planning variables for entity_class={:?} variable_name={:?}",
                    cfg.target.entity_class,
                    cfg.target.variable_name
                );
            }

            let heuristic = cfg.construction_heuristic_type;
            if is_list_only_heuristic(heuristic) {
                assert!(
                    self.list_construction.is_some(),
                    "list construction heuristic {:?} configured against a solution with no planning list variable",
                    heuristic
                );
                assert!(
                    !explicit_target || list_matches,
                    "list construction heuristic {:?} does not match the targeted planning list variable for entity_class={:?} variable_name={:?}",
                    heuristic,
                    cfg.target.entity_class,
                    cfg.target.variable_name
                );
                self.solve_list(solver_scope);
                return;
            }

            if is_standard_only_heuristic(heuristic) {
                assert!(
                    !explicit_target || standard_matches,
                    "standard construction heuristic {:?} does not match targeted standard planning variables for entity_class={:?} variable_name={:?}",
                    heuristic,
                    cfg.target.entity_class,
                    cfg.target.variable_name
                );
                build_descriptor_construction(Some(cfg), &self.descriptor).solve(solver_scope);
                return;
            }
        }

        if self.list_construction.is_none() {
            build_descriptor_construction(config, &self.descriptor).solve(solver_scope);
            return;
        }

        let standard_remaining = standard_work_remaining(
            &self.descriptor,
            if explicit_target { entity_class } else { None },
            if explicit_target { variable_name } else { None },
            solver_scope.working_solution(),
        );
        let list_remaining = self
            .list_construction
            .as_ref()
            .map(|args| {
                (!explicit_target || list_matches)
                    && list_work_remaining(args, solver_scope.working_solution())
            })
            .unwrap_or(false);

        if standard_remaining {
            build_descriptor_construction(config, &self.descriptor).solve(solver_scope);
        }
        if list_remaining {
            self.solve_list(solver_scope);
        }
    }

    fn phase_type_name(&self) -> &'static str {
        "UnifiedConstruction"
    }
}

impl<S, V> UnifiedConstruction<S, V>
where
    S: PlanningSolution + 'static,
    V: Copy + PartialEq + Eq + Hash + Into<usize> + Send + Sync + Debug + 'static,
{
    fn solve_list<D, ProgressCb>(&self, solver_scope: &mut SolverScope<'_, S, D, ProgressCb>)
    where
        D: solverforge_scoring::Director<S>,
        ProgressCb: ProgressCallback<S>,
    {
        let Some(args) = self.list_construction.as_ref() else {
            panic!("list construction configured against a standard-variable context");
        };
        let normalized = normalize_list_construction_config(self.config.as_ref());
        build_list_construction(
            normalized.as_ref(),
            args.element_count,
            args.assigned_elements,
            args.entity_count,
            args.list_len,
            args.list_insert,
            args.list_remove,
            args.index_to_element,
            args.descriptor_index,
            args.depot_fn,
            args.distance_fn,
            args.element_load_fn,
            args.capacity_fn,
            args.assign_route_fn,
            args.merge_feasible_fn,
            args.k_opt_get_route,
            args.k_opt_set_route,
            args.k_opt_depot_fn,
            args.k_opt_distance_fn,
            args.k_opt_feasible_fn,
        )
        .solve(solver_scope);
    }
}

pub struct ListConstructionArgs<S, V> {
    pub element_count: fn(&S) -> usize,
    pub assigned_elements: fn(&S) -> Vec<V>,
    pub entity_count: fn(&S) -> usize,
    pub list_len: fn(&S, usize) -> usize,
    pub list_insert: fn(&mut S, usize, usize, V),
    pub list_remove: fn(&mut S, usize, usize) -> V,
    pub index_to_element: fn(&S, usize) -> V,
    pub descriptor_index: usize,
    pub depot_fn: Option<fn(&S) -> usize>,
    pub distance_fn: Option<fn(&S, usize, usize) -> i64>,
    pub element_load_fn: Option<fn(&S, usize) -> i64>,
    pub capacity_fn: Option<fn(&S) -> i64>,
    pub assign_route_fn: Option<fn(&mut S, usize, Vec<V>)>,
    pub merge_feasible_fn: Option<fn(&S, &[usize]) -> bool>,
    pub k_opt_get_route: Option<fn(&S, usize) -> Vec<usize>>,
    pub k_opt_set_route: Option<fn(&mut S, usize, Vec<usize>)>,
    pub k_opt_depot_fn: Option<fn(&S, usize) -> usize>,
    pub k_opt_distance_fn: Option<fn(&S, usize, usize) -> i64>,
    pub k_opt_feasible_fn: Option<fn(&S, usize, &[usize]) -> bool>,
}

impl<S, V> Clone for ListConstructionArgs<S, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S, V> Copy for ListConstructionArgs<S, V> {}

pub(super) fn list_work_remaining<S, V>(args: &ListConstructionArgs<S, V>, solution: &S) -> bool
where
    S: PlanningSolution,
    V: Copy + PartialEq + Eq + Hash + Send + Sync + 'static,
{
    (args.assigned_elements)(solution).len() < (args.element_count)(solution)
}

pub(super) fn has_explicit_target(config: &ConstructionHeuristicConfig) -> bool {
    config.target.variable_name.is_some() || config.target.entity_class.is_some()
}

pub(super) fn is_list_only_heuristic(heuristic: ConstructionHeuristicType) -> bool {
    matches!(
        heuristic,
        ConstructionHeuristicType::ListRoundRobin
            | ConstructionHeuristicType::ListCheapestInsertion
            | ConstructionHeuristicType::ListRegretInsertion
            | ConstructionHeuristicType::ListClarkeWright
            | ConstructionHeuristicType::ListKOpt
    )
}

pub(super) fn is_standard_only_heuristic(heuristic: ConstructionHeuristicType) -> bool {
    matches!(
        heuristic,
        ConstructionHeuristicType::FirstFitDecreasing
            | ConstructionHeuristicType::WeakestFit
            | ConstructionHeuristicType::WeakestFitDecreasing
            | ConstructionHeuristicType::StrongestFit
            | ConstructionHeuristicType::StrongestFitDecreasing
            | ConstructionHeuristicType::AllocateEntityFromQueue
            | ConstructionHeuristicType::AllocateToValueFromQueue
    )
}

pub(super) fn list_target_matches<S, V>(
    config: &ConstructionHeuristicConfig,
    descriptor: &SolutionDescriptor,
    list_construction: Option<&ListConstructionArgs<S, V>>,
    list_variable_name: Option<&'static str>,
) -> bool
where
    S: PlanningSolution,
    V: Copy + PartialEq + Eq + Hash + Send + Sync + 'static,
{
    if !has_explicit_target(config) {
        return false;
    }

    let Some(list_variable_name) = list_variable_name else {
        return false;
    };
    let Some(list_construction) = list_construction else {
        return false;
    };
    let Some(list_entity_name) = descriptor
        .entity_descriptors
        .get(list_construction.descriptor_index)
        .map(|entity| entity.type_name)
    else {
        return false;
    };

    config
        .target
        .variable_name
        .as_deref()
        .is_none_or(|name| name == list_variable_name)
        && config
            .target
            .entity_class
            .as_deref()
            .is_none_or(|name| name == list_entity_name)
}

pub(super) fn normalize_list_construction_config(
    config: Option<&ConstructionHeuristicConfig>,
) -> Option<ConstructionHeuristicConfig> {
    let mut config = config.cloned()?;
    config.construction_heuristic_type = match config.construction_heuristic_type {
        ConstructionHeuristicType::FirstFit | ConstructionHeuristicType::CheapestInsertion => {
            ConstructionHeuristicType::ListCheapestInsertion
        }
        other => other,
    };
    Some(config)
}
