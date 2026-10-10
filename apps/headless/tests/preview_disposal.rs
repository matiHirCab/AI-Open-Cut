use std::{
    fs,
    io::Write,
    process::{Command, Stdio},
};
fn call(root: &std::path::Path, input: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_opencut-headless"))
        .arg("--dispose-owned-preview")
        .env_clear()
        .env("OPENCUT_PROJECTS_DIR", root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn private_native_adapter_is_bounded_and_preserves_public_protocol() {
    let root = tempfile::tempdir().unwrap();
    let project = "12345678-1234-1234-1234-123456789abc";
    let relative = "previews/preview-12345678-1234-1234-1234-123456789abc.png";
    fs::create_dir_all(root.path().join(project).join("previews")).unwrap();
    let path = root.path().join(project).join(relative);
    fs::write(&path, b"owned").unwrap();
    let input =
        serde_json::to_vec(&serde_json::json!({"projectId":project,"relativePath":relative}))
            .unwrap();
    let result = call(root.path(), &input);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(!path.exists());
    assert!(result.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&result.stdout).unwrap(),
        serde_json::json!({"type":"result","result":{"disposed":true}})
    );
    assert!(call(root.path(), &input).status.success());
    for invalid in [
        b"{}".to_vec(),
        vec![b'x'; 1025],
        b"{\"projectId\":\"secret/path\",\"relativePath\":\"../../foreign\"}".to_vec(),
        b"{\"operation\":\"dispose_owned_preview\"}".to_vec(),
    ] {
        let result = call(root.path(), &invalid);
        assert!(!result.status.success());
        let result: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(result["error"]["code"], "VALIDATION_FAILED");
        assert_eq!(
            result["error"]["message"],
            "Preview artifact disposal is unavailable or unsafe"
        );
    }
}
