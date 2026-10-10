#![cfg(windows)]
use opencut_editor_core::dispose_owned_preview;
use std::{fs, path::Path, process::Command};
fn junction(link: &Path, target: &Path) {
    assert!(
        Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()
            .unwrap()
            .success()
    );
}
#[test]
fn final_reparse_target_is_refused_without_deleting_outside_content() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let project = "12345678-1234-1234-1234-123456789abc";
    let relative = "previews/preview-12345678-1234-1234-1234-123456789abc.png";
    fs::create_dir_all(root.path().join(project).join("previews")).unwrap();
    fs::write(outside.path().join("victim"), b"preserve").unwrap();
    let target = root.path().join(project).join(relative);
    junction(&target, outside.path());
    assert!(dispose_owned_preview(root.path(), project, relative).is_err());
    assert_eq!(
        fs::read(outside.path().join("victim")).unwrap(),
        b"preserve"
    );
    fs::remove_dir(&target).unwrap();
}
#[test]
fn trusted_parent_junction_is_supported_but_a_reparse_root_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let parent = tempfile::tempdir().unwrap();
    let project = "12345678-1234-1234-1234-123456789abc";
    let relative = "previews/preview-range-12345678-1234-1234-1234-123456789abc.mp4";
    let projects = root.path().join("projects");
    fs::create_dir_all(projects.join(project).join("previews")).unwrap();
    let path = projects.join(project).join(relative);
    fs::write(&path, b"owned").unwrap();
    let root_link = parent.path().join("root-link");
    junction(&root_link, &projects);
    assert!(dispose_owned_preview(&root_link, project, relative).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"owned");
    let alias = parent.path().join("parent-alias");
    junction(&alias, root.path());
    dispose_owned_preview(&alias.join("projects"), project, relative).unwrap();
    assert!(!path.exists());
    fs::remove_dir(root_link).unwrap();
    fs::remove_dir(alias).unwrap();
}
