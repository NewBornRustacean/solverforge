solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/chained_scalar_metadata/domain";

    mod plan;
    mod task;
    mod worker;

    pub use plan::Plan;
    pub use task::Task;
    pub use worker::Worker;
}
