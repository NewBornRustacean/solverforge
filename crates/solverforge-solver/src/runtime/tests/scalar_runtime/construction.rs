
#[test]
fn scalar_runtime_frontier_marks_kept_optional_none_as_complete() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![ScalarRuntimeWorker],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::new(plan, descriptor.clone());
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut targeted_phase = Construction::new(
        Some(config(
            ConstructionHeuristicType::CheapestInsertion,
            Some("Task"),
            Some("worker_idx"),
        )),
        descriptor.clone(),
        scalar_runtime_model(),
    );
    targeted_phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, None);
    assert_eq!(solver_scope.stats().step_count, 1);
    assert!(
        !scalar_work_remaining_with_frontier(
            &descriptor,
            solver_scope.construction_frontier(),
            solver_scope.solution_revision(),
            None,
            None,
            solver_scope.working_solution(),
        ),
        "completed optional None should not be treated as remaining scalar work",
    );

    let mut untargeted_phase = Construction::new(None, descriptor, scalar_runtime_model());
    untargeted_phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, None);
    assert_eq!(solver_scope.stats().step_count, 1);
    assert_eq!(solver_scope.stats().moves_accepted, 0);
}

#[test]
fn no_op_runtime_construction_still_seeds_score_and_best_solution() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![ScalarRuntimeWorker],
        tasks: vec![ScalarRuntimeTask {
            worker_idx: Some(0),
        }],
    };
    let director = ScalarRuntimeDirector::new(plan, descriptor.clone());
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(None, descriptor, scalar_runtime_model());
    phase.solve(&mut solver_scope);

    assert_eq!(
        solver_scope.current_score().copied(),
        Some(SoftScore::of(-1))
    );
    assert_eq!(solver_scope.best_score().copied(), Some(SoftScore::of(-1)));
}

#[test]
fn scalar_runtime_first_fit_keeps_none_when_optional_baseline_is_not_beaten() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [-5, -1, -2],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(None, descriptor, scalar_runtime_model());
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, None);
    assert_eq!(
        solver_scope.current_score().copied(),
        Some(SoftScore::of(0))
    );
    assert_eq!(solver_scope.stats().moves_accepted, 0);
    assert_eq!(solver_scope.stats().step_count, 1);
}

#[test]
fn scalar_runtime_first_fit_skips_worse_candidate_for_later_improvement() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [-5, 7, -1],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(None, descriptor, scalar_runtime_model());
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, Some(1));
    assert_eq!(
        solver_scope.current_score().copied(),
        Some(SoftScore::of(7))
    );
    assert_eq!(solver_scope.stats().moves_accepted, 1);
}

#[test]
fn scalar_runtime_first_fit_takes_first_improving_candidate() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [7, -5, 3],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(None, descriptor, scalar_runtime_model());
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, Some(0));
    assert_eq!(
        solver_scope.current_score().copied(),
        Some(SoftScore::of(7))
    );
    assert_eq!(solver_scope.stats().moves_accepted, 1);
}

#[test]
fn scalar_runtime_first_fit_required_slot_still_assigns_first_doable() {
    let descriptor = scalar_runtime_descriptor_with_allows_unassigned(false);
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [-5, -1, -2],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(
        None,
        descriptor,
        scalar_runtime_model_with_allows_unassigned(false),
    );
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, Some(0));
    assert_eq!(solver_scope.stats().moves_accepted, 1);
}

#[test]
fn scalar_runtime_weakest_fit_keeps_none_from_runtime_value_order_hook() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [0, 7, 3],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(
        Some(config(
            ConstructionHeuristicType::WeakestFit,
            Some("Task"),
            Some("worker_idx"),
        )),
        descriptor,
        scalar_runtime_model_with_hooks(true, Some(scalar_runtime_value_order_key)),
    );
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, None);
}

#[test]
fn scalar_runtime_weakest_fit_assigns_from_runtime_value_order_hook() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [1, 7, 3],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(
        Some(config(
            ConstructionHeuristicType::WeakestFit,
            Some("Task"),
            Some("worker_idx"),
        )),
        descriptor,
        scalar_runtime_model_with_hooks(true, Some(scalar_runtime_value_order_key)),
    );
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, Some(0));
}

#[test]
fn scalar_runtime_strongest_fit_keeps_none_from_runtime_value_order_hook() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [7, 3, 0],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(
        Some(config(
            ConstructionHeuristicType::StrongestFit,
            Some("Task"),
            Some("worker_idx"),
        )),
        descriptor,
        scalar_runtime_model_with_hooks(true, Some(scalar_runtime_value_order_key)),
    );
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, None);
}

#[test]
fn scalar_runtime_strongest_fit_assigns_from_runtime_value_order_hook() {
    let descriptor = scalar_runtime_descriptor();
    let plan = ScalarRuntimePlan {
        score: None,
        workers: vec![
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
            ScalarRuntimeWorker,
        ],
        tasks: vec![ScalarRuntimeTask { worker_idx: None }],
    };
    let director = ScalarRuntimeDirector::with_score_mode(
        plan,
        descriptor.clone(),
        ScalarRuntimeScoreMode::ByWorker {
            unassigned_score: 0,
            assigned_scores: [-5, 3, 7],
        },
    );
    let mut solver_scope = SolverScope::new(director);
    solver_scope.start_solving();

    let mut phase = Construction::new(
        Some(config(
            ConstructionHeuristicType::StrongestFit,
            Some("Task"),
            Some("worker_idx"),
        )),
        descriptor,
        scalar_runtime_model_with_hooks(true, Some(scalar_runtime_value_order_key)),
    );
    phase.solve(&mut solver_scope);

    assert_eq!(solver_scope.working_solution().tasks[0].worker_idx, Some(2));
}
