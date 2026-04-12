use super::*;
use solverforge_config::{
    AcceptorConfig, LateAcceptanceConfig, SimulatedAnnealingConfig, TabuSearchConfig,
};
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

#[test]
fn test_acceptor_builder_hill_climbing() {
    let config = AcceptorConfig::HillClimbing;
    let _acceptor: AnyAcceptor<TestSolution> = AcceptorBuilder::build(&config);
}

#[test]
fn test_acceptor_builder_tabu_search() {
    let config = AcceptorConfig::TabuSearch(TabuSearchConfig {
        entity_tabu_size: Some(10),
        ..Default::default()
    });
    let _acceptor: AnyAcceptor<TestSolution> = AcceptorBuilder::build(&config);
}

#[test]
fn test_acceptor_builder_simulated_annealing() {
    let config = AcceptorConfig::SimulatedAnnealing(SimulatedAnnealingConfig {
        starting_temperature: Some("2".to_string()),
    });
    let _acceptor: AnyAcceptor<TestSolution> = AcceptorBuilder::build(&config);
}

#[test]
fn test_acceptor_builder_simulated_annealing_accepts_fractional_scalar() {
    let config = AcceptorConfig::SimulatedAnnealing(SimulatedAnnealingConfig {
        starting_temperature: Some("2.5".to_string()),
    });
    let _acceptor: AnyAcceptor<TestSolution> = AcceptorBuilder::build(&config);
}

#[test]
fn test_acceptor_builder_late_acceptance() {
    let config = AcceptorConfig::LateAcceptance(LateAcceptanceConfig {
        late_acceptance_size: Some(500),
    });
    let _acceptor: AnyAcceptor<TestSolution> = AcceptorBuilder::build(&config);
}
