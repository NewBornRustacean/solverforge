solverforge::planning_model! {
    root = "crates/solverforge-macros/tests/ui/pass/list_multi_module/domain";

    mod item;
    mod container;
    mod plan;

    pub use item::Item;
    pub use container::Container;
    pub use plan::Plan;
}
