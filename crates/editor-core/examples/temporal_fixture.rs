//! Fresh synthetic stores for the approved temporal fixture and desktop workflow.
#[path = "../tests/support/temporal_fixture.rs"]
mod fixture;
use opencut_editor_core as core_facade;
fn main() {
    let root = std::env::args_os()
        .nth(1)
        .expect("supply a new fixture directory");
    let root = std::path::Path::new(&root);
    assert!(!root.exists(), "fixture directory must not already exist");
    for (name, b) in [("family-a", false), ("family-b", true)] {
        let f = fixture::seed(&root.join(name), b);
        println!(
            "{name}: --project-store {} --project-id {}\nrevision={} familyB={} overlay={} items={:?}",
            f.core.paths().projects_root().display(),
            f.id,
            f.project().revision,
            f.family_b,
            f.track_id,
            f.item_ids
        );
    }
}
