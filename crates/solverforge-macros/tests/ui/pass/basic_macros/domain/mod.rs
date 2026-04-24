solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/basic_macros/domain";

    mod task;
    mod worker;
    mod plan;

    pub use task::Task;
    pub use worker::Worker;
    pub use plan::Plan;
}
