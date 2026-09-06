//! Create a local synthetic project for desktop inspection: cargo run -p opencut-editor-core --example rule_card_fixture -- <new-directory>
#[path = "../tests/support/rule_card.rs"]
mod fixture;

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .expect("supply a new fixture directory");
    let path = std::path::Path::new(&path);
    assert!(!path.exists(), "fixture directory must not already exist");
    std::fs::create_dir_all(path).unwrap();
    let f = fixture::seed(path);
    // Exercise the same lifecycle before handing off the original state for inspection.
    f.move_parent();
    f.core.undo(&f.id, f.project().revision).unwrap();
    println!(
        "--project-store {} --project-id {}",
        f.core.paths().projects_root().display(),
        f.id
    );
}
