use std::any::{Any, TypeId};

use solverforge_core::domain::{
    EntityCollectionExtractor, EntityDescriptor, PlanningSolution, ProblemFactDescriptor,
    SolutionDescriptor, VariableDescriptor,
};
use solverforge_core::score::SoftScore;
use solverforge_scoring::ScoreDirector;

use crate::heuristic::selector::move_selector::MoveSelector;

use super::{build_descriptor_move_selector, standard_work_remaining};

#[derive(Clone, Debug)]
struct Worker;

#[derive(Clone, Debug)]
struct Task {
    worker_idx: Option<usize>,
}

#[derive(Clone, Debug)]
struct Plan {
    workers: Vec<Worker>,
    tasks: Vec<Task>,
    score: Option<SoftScore>,
}

impl PlanningSolution for Plan {
    type Score = SoftScore;

    fn score(&self) -> Option<Self::Score> {
        self.score
    }

    fn set_score(&mut self, score: Option<Self::Score>) {
        self.score = score;
    }
}

fn get_worker_idx(entity: &dyn Any) -> Option<usize> {
    entity
        .downcast_ref::<Task>()
        .expect("task expected")
        .worker_idx
}

fn set_worker_idx(entity: &mut dyn Any, value: Option<usize>) {
    entity
        .downcast_mut::<Task>()
        .expect("task expected")
        .worker_idx = value;
}

fn descriptor() -> SolutionDescriptor {
    SolutionDescriptor::new("Plan", TypeId::of::<Plan>())
        .with_entity(
            EntityDescriptor::new("Task", TypeId::of::<Task>(), "tasks")
                .with_extractor(Box::new(EntityCollectionExtractor::new(
                    "Task",
                    "tasks",
                    |s: &Plan| &s.tasks,
                    |s: &mut Plan| &mut s.tasks,
                )))
                .with_variable(
                    VariableDescriptor::genuine("worker_idx")
                        .with_allows_unassigned(true)
                        .with_value_range("workers")
                        .with_usize_accessors(get_worker_idx, set_worker_idx),
                ),
        )
        .with_problem_fact(
            ProblemFactDescriptor::new("Worker", TypeId::of::<Worker>(), "workers").with_extractor(
                Box::new(EntityCollectionExtractor::new(
                    "Worker",
                    "workers",
                    |s: &Plan| &s.workers,
                    |s: &mut Plan| &mut s.workers,
                )),
            ),
        )
}

#[test]
fn solution_level_value_range_generates_standard_work() {
    let descriptor = descriptor();
    let plan = Plan {
        workers: vec![Worker, Worker, Worker],
        tasks: vec![Task { worker_idx: None }],
        score: None,
    };

    assert!(standard_work_remaining(
        &descriptor,
        Some("Task"),
        Some("worker_idx"),
        &plan
    ));
}

#[test]
fn solution_level_value_range_builds_change_moves() {
    let descriptor = descriptor();
    let plan = Plan {
        workers: vec![Worker, Worker, Worker],
        tasks: vec![Task { worker_idx: None }],
        score: None,
    };
    let director = ScoreDirector::simple(plan, descriptor.clone(), |s, _| s.tasks.len());
    let selector = build_descriptor_move_selector::<Plan>(None, &descriptor);

    assert_eq!(selector.size(&director), 3);
}
