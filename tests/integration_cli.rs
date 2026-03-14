use std::process::Command;

#[test]
fn info_command_prints_project_identity() {
    let output = Command::new(env!("CARGO_BIN_EXE_aetherdb"))
        .arg("info")
        .output()
        .expect("info command should execute");

    assert!(output.status.success(), "info command should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout should be valid utf-8");
    assert!(stdout.contains("AetherDB v"));
    assert!(stdout.contains("object storage"));
}
