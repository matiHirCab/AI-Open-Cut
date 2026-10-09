#[path = "../tests/support/narration_fixture.rs"]
mod fixture;
fn main() {
    let root = std::env::args_os()
        .nth(1)
        .expect("supply a new fixture directory");
    let root = std::path::Path::new(&root);
    assert!(!root.exists(), "fixture directory must not already exist");
    let fixture = fixture::seed(root, true);
    println!(
        "--project-store {} --project-id {}",
        fixture.core.paths().projects_root().display(),
        fixture.id
    );
    println!(
        "revision {} · synthetic estimated alignment",
        fixture.project().revision
    );
    println!(
        "project {} · saved asset {} · explicit-alignment source {} · {} creation aliases",
        fixture.dir().display(),
        fixture.asset,
        fixture.plain_asset,
        fixture.aliases.len()
    );
    println!("{} owned files", fixture::inventory(root).len());
}
