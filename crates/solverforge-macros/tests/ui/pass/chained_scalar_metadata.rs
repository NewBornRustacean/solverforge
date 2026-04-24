#[path = "chained_scalar_metadata/domain/mod.rs"]
mod domain;

use domain::*;

fn main() {
    let _ = Plan {
        workers: Vec::new(),
        tasks: Vec::new(),
        score: None,
    };
    let descriptor = Plan::descriptor();
    let task_descriptor = &descriptor.entity_descriptors[0];
    let previous = task_descriptor
        .find_variable("previous")
        .expect("chained variable descriptor");
    let worker = task_descriptor
        .find_variable("worker")
        .expect("scalar variable descriptor");
    assert!(previous.usize_getter.is_none());
    assert!(previous.usize_setter.is_none());
    assert!(worker.usize_getter.is_some());
    assert!(worker.usize_setter.is_some());
}
