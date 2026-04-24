#[path = "scalar_multi_module/domain/mod.rs"]
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
    let _scalar_variable_count = task_descriptor.genuine_variable_descriptors().count();
}
