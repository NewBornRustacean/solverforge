solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/multi_list_owners/domain";

    mod route_task;
    mod shift_task;
    mod route;
    mod shift;
    mod plan;

    pub use route_task::RouteTask;
    pub use shift_task::ShiftTask;
    pub use route::Route;
    pub use shift::Shift;
    pub use plan::Plan;
}
