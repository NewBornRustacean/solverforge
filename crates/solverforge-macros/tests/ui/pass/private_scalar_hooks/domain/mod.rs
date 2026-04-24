solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/private_scalar_hooks/domain";

    mod plan;
    mod task;
    mod worker;

    pub use plan::Plan;
    pub use task::Task;
    pub use worker::Worker;
}
