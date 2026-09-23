//! Create the native rules screen for desktop inspection in a fresh directory.
//! cargo run -p opencut-editor-core --example rules_screen_fixture -- <new-directory>
#[path = "../tests/support/rules_screen.rs"]
mod fixture;

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .expect("supply a new fixture directory");
    let path = std::path::Path::new(&path);
    assert!(!path.exists(), "fixture directory must not already exist");
    std::fs::create_dir_all(path).unwrap();
    let f = fixture::seed(path);
    assert_eq!((f.width, f.height), (1920, 1080));
    f.resize_impact_word();
    f.core.undo(&f.id, f.project().revision).unwrap();
    println!(
        "--project-store {} --project-id {}",
        f.core.paths().projects_root().display(),
        f.id
    );
}
