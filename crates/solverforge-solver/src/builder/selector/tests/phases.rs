
#[test]
#[should_panic(expected = "move selector configuration produced no neighborhoods")]
fn empty_model_does_not_synthesize_scalar_neighborhoods() {
    let _ =
        build_move_selector::<MixedPlan, usize, NoopMeter, NoopMeter>(None, &empty_model(), None);
}

#[test]
fn default_scalar_local_search_uses_scalar_streaming_defaults() {
    let phase = build_local_search::<MixedPlan, usize, NoopMeter, NoopMeter>(
        None,
        &scalar_only_model(),
        Some(7),
    );
    let debug = format!("{phase:?}");

    assert!(debug.contains("SimulatedAnnealing"));
    assert!(debug.contains("accepted_count_limit: 1"));
}

#[test]
fn default_list_and_mixed_local_search_use_list_streaming_defaults() {
    let list_phase = build_local_search::<MixedPlan, usize, NoopMeter, NoopMeter>(
        None,
        &list_only_model(),
        None,
    );
    let list_debug = format!("{list_phase:?}");
    assert!(list_debug.contains("LateAcceptance"));
    assert!(list_debug.contains("accepted_count_limit: 4"));

    let mixed_phase =
        build_local_search::<MixedPlan, usize, NoopMeter, NoopMeter>(None, &mixed_model(), None);
    let mixed_debug = format!("{mixed_phase:?}");
    assert!(mixed_debug.contains("LateAcceptance"));
    assert!(mixed_debug.contains("accepted_count_limit: 4"));
}

#[test]
fn explicit_acceptor_and_forager_configs_override_defaults() {
    let config = LocalSearchConfig {
        acceptor: Some(AcceptorConfig::LateAcceptance(LateAcceptanceConfig {
            late_acceptance_size: Some(17),
        })),
        forager: Some(ForagerConfig::FirstBestScoreImproving),
        move_selector: None,
        termination: None,
    };

    let phase = build_local_search::<MixedPlan, usize, NoopMeter, NoopMeter>(
        Some(&config),
        &scalar_only_model(),
        None,
    );
    let debug = format!("{phase:?}");

    assert!(debug.contains("LateAcceptance"));
    assert!(debug.contains("size: 17"));
    assert!(debug.contains("BestScoreImproving"));
}

#[test]
fn local_search_phase_uses_configured_step_count_limit() {
    let config = LocalSearchConfig {
        acceptor: None,
        forager: None,
        move_selector: None,
        termination: Some(TerminationConfig {
            step_count_limit: Some(3),
            ..TerminationConfig::default()
        }),
    };

    let phase = build_local_search::<MixedPlan, usize, NoopMeter, NoopMeter>(
        Some(&config),
        &scalar_only_model(),
        None,
    );
    let debug = format!("{phase:?}");

    assert!(debug.contains("step_limit: Some(3)"));
}
