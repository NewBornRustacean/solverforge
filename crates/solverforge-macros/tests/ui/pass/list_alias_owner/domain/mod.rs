solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/list_alias_owner/domain";

    mod visit;
    mod route;
    mod plan;

    pub use visit::Visit;
    pub use route::Route as VehicleRoute;
    pub use plan::Plan;
}
