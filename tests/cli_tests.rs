use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Path to the compiled binary - uses CARGO_BIN_EXE environment variable
fn binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target");
    path.push("debug");
    path.push("mermaid-cli");
    path
}

/// Create a temporary .mmd file for testing
fn create_test_mmd(content: &str, name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(name);
    fs::write(&path, content).expect("Failed to write test file");
    path
}

/// Clean up test files
fn cleanup(paths: &[&PathBuf]) {
    for p in paths {
        let _ = fs::remove_file(p);
    }
}

#[test]
fn test_cli_render_from_file() {
    let input = create_test_mmd("graph TD; A-->B", "test_render_file_input.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_render_file_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG file was not created");

    let svg_content = fs::read_to_string(&output).expect("Failed to read output SVG");
    assert!(
        svg_content.contains("<svg"),
        "Output does not contain SVG tag"
    );

    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_from_stdin() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_render_stdin_output.svg");

    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    // Write input to stdin
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"graph TD; A-->B\n")
            .expect("Failed to write to stdin");
    }

    let output_result = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output_result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&output_result.stderr)
    );
    assert!(output.exists(), "Output SVG file was not created");

    let svg_content = fs::read_to_string(&output).expect("Failed to read output SVG");
    assert!(
        svg_content.contains("<svg"),
        "Output does not contain SVG tag"
    );

    cleanup(&[&output]);
}

#[test]
fn test_cli_output_to_stdout() {
    let input = create_test_mmd("graph TD; A-->B", "test_render_stdout_input.mmd");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("<svg"), "Output does not contain SVG tag");

    cleanup(&[&input]);
}

#[test]
fn test_cli_missing_file() {
    let result = Command::new(binary_path())
        .arg("nonexistent_file.mmd")
        .arg("-o")
        .arg("should_not_create.svg")
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "CLI should have exited with error"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error"),
        "Error message should be printed to stderr"
    );

    // Verify the output file was NOT created
    let output_file = PathBuf::from("should_not_create.svg");
    assert!(
        !output_file.exists(),
        "Output file should not have been created"
    );
}

#[test]
fn test_cli_help() {
    let result = Command::new(binary_path())
        .arg("--help")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("mermaid-cli"));
    assert!(stdout.contains("--help"));
}

#[test]
fn test_cli_version_flag() {
    let result = Command::new(binary_path())
        .arg("--version")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("0.1.0-alpha"),
        "Version flag should output version string"
    );
}

#[test]
fn test_cli_version_shorthand() {
    let result = Command::new(binary_path())
        .arg("-V")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("0.1.0-alpha"),
        "-V should output version string"
    );
}

#[test]
fn test_cli_help_shorthand() {
    let result = Command::new(binary_path())
        .arg("-h")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.contains("mermaid-cli"));
    assert!(stdout.contains("USAGE"));
}

#[test]
fn test_cli_missing_output_arg() {
    let result = Command::new(binary_path())
        .arg("input.mmd")
        .arg("-o")
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "CLI should exit with error when -o has no argument"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error"),
        "Error message should be on stderr"
    );
    assert!(stderr.contains("-o"), "Error should mention -o flag");
}

#[test]
fn test_cli_stdin_stdout() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"graph TD; A-->B\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "stdout should contain SVG output");
}

#[test]
fn test_cli_invalid_option() {
    let result = Command::new(binary_path())
        .arg("--invalid-flag")
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "CLI should exit with error for invalid option"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    // Should provide some error feedback
    assert!(
        !stderr.is_empty(),
        "stderr should contain error information"
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    // Should not produce any SVG output for invalid option
    assert!(!stdout.contains("<svg"), "stdout should not contain SVG");
}

#[test]
fn test_cli_exit_codes() {
    // Success case: valid render
    let input = create_test_mmd("graph TD; A-->B", "test_exit_success.mmd");
    let success = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(
        success.status.success(),
        "Valid render should exit with code 0"
    );
    cleanup(&[&input]);

    // Error case: missing file
    let error = Command::new(binary_path())
        .arg("nonexistent_file.mmd")
        .output()
        .expect("Failed to run CLI");
    assert!(
        !error.status.success(),
        "Missing file should exit with non-zero code"
    );
}
