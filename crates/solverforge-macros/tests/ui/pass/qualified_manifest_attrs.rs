#[path = "qualified_manifest_attrs/domain/mod.rs"]
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
    let _worker = task_descriptor.find_variable("worker");
}
