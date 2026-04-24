solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/field_only_solution/domain";

    mod visit;
    mod route;
    mod plan;

    pub use visit::Visit;
    pub use route::Route;
    pub use plan::Plan;
}
