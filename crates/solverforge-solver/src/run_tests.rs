use super::load_solver_config_from;
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn temp_config_path() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir()
        .join(format!(
            "solverforge-run-tests-{}-{suffix}",
            std::process::id()
        ))
        .join("solver.toml")
}

#[test]
fn load_solver_config_from_preserves_file_settings() {
    let path = temp_config_path();
    let parent = path.parent().expect("temp file should have a parent");
    fs::create_dir_all(parent).expect("temp directory should be created");
    fs::write(
        &path,
        r#"
random_seed = 41

[termination]
seconds_spent_limit = 5

[[phases]]
type = "construction_heuristic"
construction_heuristic_type = "first_fit"
"#,
    )
    .expect("solver.toml should be written");

    let config = load_solver_config_from(&path);

    assert_eq!(config.random_seed, Some(41));
    assert_eq!(config.time_limit(), Some(Duration::from_secs(5)));
    assert_eq!(config.phases.len(), 1);

    fs::remove_dir_all(parent).expect("temp directory should be removed");
}
