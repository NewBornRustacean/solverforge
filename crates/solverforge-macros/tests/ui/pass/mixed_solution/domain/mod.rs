solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/mixed_solution/domain";

    mod worker;
    mod visit;
    mod route;
    mod plan;

    pub use worker::Worker;
    pub use visit::Visit;
    pub use route::Route;
    pub use plan::MixedPlan;
}
