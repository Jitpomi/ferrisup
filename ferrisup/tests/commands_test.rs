//! Integration tests for FerrisUp commands

use anyhow::Result;
use std::process::{Command, Stdio};

mod common;

#[test]
fn test_preview_command() -> Result<()> {
    // Test the preview command with a specific template
    let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args(["preview", "--component-type", "minimal"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Print output for debugging
    // Preview command executed successfully
    if !stderr.is_empty() {
        // Check stderr if needed
    }

    assert!(output.status.success(), "Preview failed: {stderr}");

    assert!(stdout.contains("Component: minimal"));
    assert!(stdout.contains("Files:"));
    assert!(stdout.contains("Cargo.toml"));

    Ok(())
}

#[test]
fn test_list_command() -> Result<()> {
    // Test the list command
    let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .arg("list")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    // Check that the command executed successfully
    assert!(output.status.success(), "List command failed");

    // Verify expected content in output
    assert!(
        stdout.contains("minimal"),
        "Output should list 'minimal' template"
    );
    assert!(
        stdout.contains("library"),
        "Output should list 'library' template"
    );

    Ok(())
}

#[test]
fn test_new_command() -> Result<()> {
    // Create a temp directory for the test
    let temp_dir = common::create_test_dir()?;
    let dir_path = temp_dir.path();

    // Test the new command with the current command structure
    // Using --component-type instead of --template and adding --no-interactive
    let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args(&[
            "new",
            "test_project",
            "--component-type",
            "minimal",
            "--no-interactive",
        ])
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Print output for debugging
    // New command executed
    if !stderr.is_empty() {
        // Check stderr if needed
    }

    assert!(
        output.status.success(),
        "New command failed: {stderr}\n{stdout}"
    );

    // Check that the project directory was created
    let project_path = dir_path.join("test_project");
    // Verify project was created correctly

    assert!(project_path.is_dir(), "Project directory was not created");
    assert!(
        project_path.join("Cargo.toml").is_file(),
        "Cargo.toml is missing"
    );
    assert!(
        project_path.join("src/main.rs").is_file(),
        "src/main.rs is missing"
    );

    common::cleanup_test_dir(temp_dir)?;
    Ok(())
}

#[test]
fn test_new_command_refuses_existing_destination() -> Result<()> {
    let temp_dir = common::create_test_dir()?;
    let project_path = temp_dir.path().join("existing_project");
    std::fs::create_dir(&project_path)?;
    let sentinel = project_path.join("keep.txt");
    std::fs::write(&sentinel, "do not overwrite")?;

    let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args([
            "new",
            "existing_project",
            "--component-type",
            "minimal",
            "--no-interactive",
        ])
        .current_dir(temp_dir.path())
        .output()?;

    assert!(!output.status.success());
    assert_eq!(std::fs::read_to_string(sentinel)?, "do not overwrite");
    Ok(())
}

#[test]
fn test_new_command_rejects_path_as_name() -> Result<()> {
    let temp_dir = common::create_test_dir()?;
    let escaped_name = format!(
        "../escaped-{}",
        temp_dir.path().file_name().unwrap().to_string_lossy()
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args([
            "new",
            &escaped_name,
            "--component-type",
            "minimal",
            "--no-interactive",
        ])
        .current_dir(temp_dir.path())
        .output()?;

    assert!(!output.status.success());
    assert!(
        !temp_dir
            .path()
            .parent()
            .unwrap()
            .join(escaped_name.trim_start_matches("../"))
            .exists()
    );
    Ok(())
}

#[test]
fn test_workspace_command() -> Result<()> {
    // Create a temp directory for the test
    let temp_dir = common::create_test_dir()?;
    let dir_path = temp_dir.path();

    // Initialize a workspace
    let init_output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args(&["workspace", "--action", "init", "--path", "."])
        .current_dir(dir_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let _init_stdout = String::from_utf8_lossy(&init_output.stdout).to_string();
    let init_stderr = String::from_utf8_lossy(&init_output.stderr).to_string();

    // Print output for debugging
    // Workspace init completed
    if !init_stderr.is_empty() {
        // Check stderr if needed
    }

    assert!(
        init_output.status.success(),
        "Workspace init failed: {init_stderr}"
    );

    // Verify workspace file was created
    // Verify workspace Cargo.toml
    assert!(
        dir_path.join("Cargo.toml").is_file(),
        "Workspace Cargo.toml is missing"
    );

    // List workspace members
    // List workspace members
    let list_output = Command::new(env!("CARGO_BIN_EXE_ferrisup"))
        .args(["workspace", "--action", "list", "--path", "."])
        .current_dir(dir_path)
        .output()?;

    let list_stdout = String::from_utf8_lossy(&list_output.stdout).to_string();
    assert!(list_output.status.success(), "Workspace list failed");
    assert!(list_stdout.contains("client/*"));

    common::cleanup_test_dir(temp_dir)?;
    Ok(())
}
