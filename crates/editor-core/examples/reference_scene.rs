#[path = "../tests/support/reference_scene.rs"]
mod fixture;
fn main() {
    let root = std::env::args_os()
        .nth(1)
        .expect("supply a new fixture directory");
    let root = std::path::Path::new(&root);
    assert!(!root.exists(), "fixture directory must not already exist");
    let f = fixture::seed(root, true);
    let p = f.project();
    println!(
        "--project-store {} --project-id {}",
        f.core.paths().projects_root().display(),
        f.id
    );
    println!(
        "{} revision {}: {} aliases, {} components, {} markers",
        f.dir().display(),
        p.revision,
        f.aliases.len(),
        p.components.len(),
        p.markers.len()
    );
}
