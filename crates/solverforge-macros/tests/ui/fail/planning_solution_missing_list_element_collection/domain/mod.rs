solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/fail/planning_solution_missing_list_element_collection/domain";

    mod route;
    mod plan;

    pub use route::Route;
    pub use plan::Plan;
}
