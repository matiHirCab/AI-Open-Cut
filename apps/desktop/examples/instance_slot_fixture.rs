//! Run with a new disposable directory; never targets an existing project.
#[path = "../src/instance_slot_fixture.rs"]
mod fixture;
#[path = "../../../crates/editor-core/tests/support/rule_card.rs"]
mod rule_card_fixture_support;

fn main() {
    let directory = std::env::args_os()
        .nth(1)
        .expect("supply a new disposable directory");
    let directory = std::path::Path::new(&directory);
    assert!(!directory.exists(), "fixture directory must be new");
    std::fs::create_dir_all(directory).unwrap();
    let fixture = fixture::seed(directory);
    println!(
        "--project-store {} --project-id {}",
        fixture.core.paths().projects_root().display(),
        fixture.id
    );
    println!(
        "revision {} · instance0={} · instance1={}",
        fixture.project().revision,
        fixture.aliases["instance0"],
        fixture.aliases["instance1"]
    );
}
