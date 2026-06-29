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

// ============================================================
// 序列图 CLI 测试
// ============================================================

#[test]
fn test_cli_sequence_from_stdin() {
    let stdin_data = "sequenceDiagram\n    Alice->Bob: Hello\n    Bob-->Alice: Hi";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_seq.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "Sequence diagram should render successfully");
    let svg = fs::read_to_string("/tmp/test_cli_seq.svg").unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Bob"));
    let _ = fs::remove_file("/tmp/test_cli_seq.svg");
}

#[test]
fn test_cli_sequence_from_file() {
    let input = create_test_mmd(
        "sequenceDiagram\n    participant Alice\n    Alice->Bob: Hello",
        "seq_test",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_seq_file.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render sequence from file");
    let svg = fs::read_to_string("/tmp/test_cli_seq_file.svg").unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Bob"));
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_seq_file.svg");
}

#[test]
fn test_cli_sequence_with_blocks() {
    let input = create_test_mmd(
        "sequenceDiagram\n    alt ok\n        A->B: yes\n    else no\n        A->B: no\n    end",
        "seq_blocks",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_seq_blocks.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render sequence with blocks");
    let svg = fs::read_to_string("/tmp/test_cli_seq_blocks.svg").unwrap();
    assert!(svg.contains("yes"));
    assert!(svg.contains("no"));
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_seq_blocks.svg");
}

#[test]
fn test_cli_sequence_with_note() {
    let input = create_test_mmd(
        "sequenceDiagram\n    participant Alice\n    Note right of Alice: This is a note",
        "seq_note",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_seq_note.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render sequence with note");
    let svg = fs::read_to_string("/tmp/test_cli_seq_note.svg").unwrap();
    assert!(svg.contains("This is a note"));
    assert!(svg.contains("#fffde7"), "Note should have yellow background");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_seq_note.svg");
}

#[test]
fn test_cli_sequence_with_activation() {
    let input = create_test_mmd(
        "sequenceDiagram\n    participant Alice\n    activate Alice\n    Alice->Bob: Hello\n    deactivate Alice",
        "seq_activation",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_seq_activation.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render sequence with activation");
    let svg = fs::read_to_string("/tmp/test_cli_seq_activation.svg").unwrap();
    assert!(svg.contains("Alice"));
    assert!(svg.contains("Hello"));
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_seq_activation.svg");
}

// ============================================================
// 饼图 CLI 测试
// ============================================================

#[test]
fn test_cli_pie_from_file() {
    let input = create_test_mmd(
        "pie title Pets\n\"Dogs\" : 386\n\"Cats\" : 85\n\"Rats\" : 15",
        "pie_test.mmd",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_pie.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render pie chart from file");
    let svg = fs::read_to_string("/tmp/test_cli_pie.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("Pets"), "Should contain title");
    assert!(svg.contains("Dogs"), "Should contain slice label");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_pie.svg");
}

#[test]
fn test_cli_pie_from_stdin() {
    let stdin_data = "pie title Sales\n\"Q1\" : 100\n\"Q2\" : 200\n\"Q3\" : 150";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_pie_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "Pie chart should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_pie_stdin.svg").unwrap();
    assert!(svg.contains("Sales"), "Should contain title");
    assert!(svg.contains("Q1"), "Should contain slice label");
    let _ = fs::remove_file("/tmp/test_cli_pie_stdin.svg");
}

#[test]
fn test_cli_pie_check() {
    let input = create_test_mmd(
        "pie title Test\n\"A\" : 50\n\"B\" : 50",
        "pie_check.mmd",
    );
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid pie chart should pass check");
    cleanup(&[&input]);
}

// ============================================================
// 类图 CLI 测试
// ============================================================

#[test]
fn test_cli_class_from_file() {
    let input = create_test_mmd(
        "classDiagram\nclass Animal {\n+String name\n+isMammal() bool\n}",
        "class_test.mmd",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_class.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render class diagram from file");
    let svg = fs::read_to_string("/tmp/test_cli_class.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("Animal"), "Should contain class name");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_class.svg");
}

#[test]
fn test_cli_class_from_stdin() {
    let stdin_data = "classDiagram\nclass Dog\nAnimal <|-- Dog";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_class_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "Class diagram should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_class_stdin.svg").unwrap();
    assert!(svg.contains("Dog"), "Should contain class name");
    let _ = fs::remove_file("/tmp/test_cli_class_stdin.svg");
}

#[test]
fn test_cli_class_check() {
    let input = create_test_mmd(
        "classDiagram\nclass Animal",
        "class_check.mmd",
    );
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid class diagram should pass check");
    cleanup(&[&input]);
}

// ============================================================
// 状态图 CLI 测试
// ============================================================

#[test]
fn test_cli_state_from_file() {
    let input = create_test_mmd(
        "stateDiagram-v2\n[*] --> Still\nStill --> Moving\nMoving --> Still\nMoving --> Crash\nCrash --> [*]",
        "state_test.mmd",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_state.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render state diagram from file");
    let svg = fs::read_to_string("/tmp/test_cli_state.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("Still"), "Should contain state name");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_state.svg");
}

#[test]
fn test_cli_state_from_stdin() {
    let stdin_data = "stateDiagram-v2\n[*] --> A\nA --> B\nB --> [*]";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_state_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "State diagram should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_state_stdin.svg").unwrap();
    assert!(svg.contains("A"), "Should contain state name");
    let _ = fs::remove_file("/tmp/test_cli_state_stdin.svg");
}

#[test]
fn test_cli_state_check() {
    let input = create_test_mmd(
        "stateDiagram-v2\n[*] --> A\nA --> [*]",
        "state_check.mmd",
    );
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid state diagram should pass check");
    cleanup(&[&input]);
}

// ============================================================
// ER 图 CLI 测试
// ============================================================

#[test]
fn test_cli_er_from_file() {
    let input = create_test_mmd(
        "erDiagram\nCUSTOMER {\nstring name\n}\nCUSTOMER ||--o{ ORDER : places",
        "er_test.mmd",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_er.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render ER diagram from file");
    let svg = fs::read_to_string("/tmp/test_cli_er.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("CUSTOMER"), "Should contain entity name");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_er.svg");
}

#[test]
fn test_cli_er_from_stdin() {
    let stdin_data = "erDiagram\nCUSTOMER ||--o{ ORDER : places\nORDER ||--|{ LINE-ITEM : contains";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_er_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "ER diagram should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_er_stdin.svg").unwrap();
    assert!(svg.contains("CUSTOMER"), "Should contain entity name");
    let _ = fs::remove_file("/tmp/test_cli_er_stdin.svg");
}

#[test]
fn test_cli_er_check() {
    let input = create_test_mmd(
        "erDiagram\nCUSTOMER ||--o{ ORDER : places",
        "er_check.mmd",
    );
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid ER diagram should pass check");
    cleanup(&[&input]);
}

// ============================================================
// 甘特图 CLI 测试
// ============================================================

#[test]
fn test_cli_gantt_from_file() {
    let input = create_test_mmd(
        "gantt\n    title Project\n    section Design\n    Task 1 :t1, 2024-01-01, 10d",
        "gantt_test.mmd",
    );
    let output = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg("/tmp/test_cli_gantt.svg")
        .output()
        .expect("Failed to run CLI");
    assert!(output.status.success(), "Should render Gantt chart from file");
    let svg = fs::read_to_string("/tmp/test_cli_gantt.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    assert!(svg.contains("Task 1"), "Should contain task name");
    cleanup(&[&input]);
    let _ = fs::remove_file("/tmp/test_cli_gantt.svg");
}

#[test]
fn test_cli_quiet_mode_from_file() {
    let input = create_test_mmd("graph TD; A-->B", "quiet_test.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("quiet_test_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .arg("-q")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "CLI should succeed with --quiet");
    // stdout should be empty in quiet mode
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(stdout.is_empty(), "Quiet mode should produce no stdout: {:?}", stdout);
    assert!(output.exists(), "Output file should be created even in quiet mode");

    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_quiet_short_flag() {
    let input = create_test_mmd("graph TD; A-->B", "quiet_short_test.mmd");
    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-q")
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "CLI should succeed with -q");
    cleanup(&[&input]);
}

#[test]
fn test_cli_gantt_from_stdin() {
    let stdin_data = "gantt\n    title Project\n    section S\n    Task :t1, 1, 5d";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_gantt_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "Gantt chart should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_gantt_stdin.svg").unwrap();
    assert!(svg.contains("Task"), "Should contain task name");
    let _ = fs::remove_file("/tmp/test_cli_gantt_stdin.svg");
}

#[test]
fn test_cli_gantt_check() {
    let input = create_test_mmd(
        "gantt\n    Task :t1, 1, 5d",
        "gantt_check.mmd",
    );
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid Gantt chart should pass check");
    cleanup(&[&input]);
}

// ============================================================
// --cssFile CLI 测试
// ============================================================

#[test]
fn test_cli_css_file() {
    let input = create_test_mmd("graph TD; A-->B", "css_test_input.mmd");
    let css_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_custom.css");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("css_test_output.svg");

    // Create a test CSS file
    fs::write(&css_path, "/* Custom CSS */\n.highlight { fill: red; }")
        .expect("Failed to write CSS file");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--cssFile")
        .arg(css_path.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "CLI should succeed with --cssFile");
    assert!(output.exists(), "Output SVG should exist");
    let svg_content = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg_content.contains("Custom CSS"), "SVG should contain custom CSS");

    cleanup(&[&input, &css_path, &output]);
}

#[test]
fn test_cli_css_file_not_found() {
    let input = create_test_mmd("graph TD; A-->B", "css_nofile_input.mmd");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--cssFile")
        .arg("nonexistent.css")
        .output()
        .expect("Failed to run CLI");

    assert!(!result.status.success(), "CLI should fail with missing CSS file");
    cleanup(&[&input]);
}

// ============================================================
// --jobs CLI 测试
// ============================================================

#[test]
fn test_cli_jobs_single_file() {
    let input = create_test_mmd("graph TD; A-->B", "jobs_single.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("jobs_single_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-j")
        .arg("4")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "CLI should work with --jobs flag");
    assert!(output.exists(), "Output should be created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("<svg"), "Should produce valid SVG");

    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_jobs_default_value() {
    let input = create_test_mmd("graph TD; A-->B", "jobs_default.mmd");

    // --jobs with no argument should error
    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--jobs")
        .output()
        .expect("Failed to run CLI");

    assert!(!result.status.success(), "--jobs without argument should fail");
    cleanup(&[&input]);
}

// ============================================================
// Markdown 输入 CLI 测试
// ============================================================

#[test]
fn test_cli_render_from_markdown() {
    let md_content = "# Project Diagram\n\n```mermaid\ngraph TD\nA[Start]-->B[End]\n```\n\nMore text";
    let input = create_test_mmd(md_content, "test_markdown.md");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_markdown_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "CLI should render from .md file");
    assert!(output.exists(), "Output SVG file was not created");
    let svg_content = fs::read_to_string(&output).expect("Failed to read output SVG");
    assert!(svg_content.contains("<svg"), "Output should contain SVG tag");
    assert!(svg_content.contains("Start"), "Should contain diagram content");

    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_from_markdown_stdin() {
    let stdin_data = "# Doc\n\n```mermaid\ngraph TD\nA-->B\n```\n";
    let mut child = Command::new(binary_path())
        .args(["--stdin", "-o", "/tmp/test_cli_md_stdin.svg"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    child.stdin.as_mut().unwrap().write_all(stdin_data.as_bytes()).unwrap();
    let result = child.wait_with_output().unwrap();

    assert!(result.status.success(), "Markdown with mermaid block should render from stdin");
    let svg = fs::read_to_string("/tmp/test_cli_md_stdin.svg").unwrap();
    assert!(svg.contains("<svg"), "Output should be valid SVG");
    let _ = fs::remove_file("/tmp/test_cli_md_stdin.svg");
}

#[test]
fn test_cli_render_from_markdown_sequence() {
    let md = "# Sequence\n\n```mermaid\nsequenceDiagram\nA->B: Hello\n```";
    let input = create_test_mmd(md, "test_seq_md.md");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_seq_md_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Should render sequence from markdown");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("Hello"), "Should contain message content");

    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_flowchart_with_theme_dark() {
    let input = create_test_mmd("graph TD; A-->B", "theme_dark_input.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("theme_dark_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--theme")
        .arg("dark")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Theme dark should work");
    assert!(output.exists());
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_with_width_height() {
    let input = create_test_mmd("graph TD; A-->B", "wh_input.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("wh_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-w")
        .arg("1024")
        .arg("-H")
        .arg("768")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Width/height flags should work");
    assert!(output.exists());
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_with_scale() {
    let input = create_test_mmd("graph TD; A-->B", "scale_input.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("scale_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--scale")
        .arg("2.0")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Scale flag should work");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_with_background_color() {
    let input = create_test_mmd("graph TD; A-->B", "bg_input.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("bg_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-b")
        .arg("lightyellow")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Background color flag should work");
    assert!(output.exists());
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("lightyellow"), "SVG should contain background color");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_show_fixes_output() {
    let input = create_test_mmd("grpah TD; A-->B", "fixes_show_input.mmd");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--show-fixes")
        .output()
        .expect("Failed to run CLI");

    // --show-fixes may fail parsing after fixes, but should still output something
    let all_output = format!("{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr));
    assert!(!all_output.is_empty(), "Should produce some output");
    cleanup(&[&input]);
}

#[test]
fn test_cli_render_complex_flowchart() {
    let input = create_test_mmd(
        "graph TD\nA[Start]-->B{Decision}\nB-->|Yes|C[Result]\nB-->|No|D[Other]",
        "complex_flow.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("complex_flow_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-t")
        .arg("neutral")
        .arg("-w")
        .arg("1200")
        .arg("-H")
        .arg("800")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Complex flowchart should render");
    assert!(output.exists());
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_render_all_types_to_stdout() {
    let types = vec![
        ("graph", "graph TD; A-->B"),
        ("sequence", "sequenceDiagram\nA->B: Hi"),
        ("pie", "pie\n\"X\" : 100"),
        ("class", "classDiagram\nclass A"),
    ];
    for (name, code) in &types {
        let input = create_test_mmd(code, &format!("type_{}.mmd", name));
        let result = Command::new(binary_path())
            .arg(input.to_str().unwrap())
            .output()
            .expect("Failed to run CLI");
        assert!(result.status.success(), "{} should render to stdout", name);
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("<svg"), "{} SVG output", name);
        cleanup(&[&input]);
    }
}

#[test]
fn test_cli_check_flowchart() {
    let input = create_test_mmd("graph TD; A-->B", "check_valid.mmd");
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(result.status.success(), "Valid file should pass check");
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_invalid() {
    let input = create_test_mmd("invalid syntax here", "check_invalid.mmd");
    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");
    assert!(!result.status.success(), "Invalid file should fail check");
    cleanup(&[&input]);
}

#[test]
fn test_cli_fix_output() {
    let input = create_test_mmd("grpah TD; A-->B", "fix_test.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fix_test_output.mmd");

    let result = Command::new(binary_path())
        .arg("fix")
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(result.status.success(), "Fix should work");
    let fixed_content = fs::read_to_string(&output).expect("Failed to read fixed file");
    assert!(fixed_content.contains("graph"), "Fixed content should contain graph");
    cleanup(&[&input, &output]);
}
use std::io::Write;

// =========================================================================
// Additional tests: theme, chart types, edge cases, combined flags, etc.
// =========================================================================

#[test]
fn test_cli_theme_forest() {
    let input = create_test_mmd(
        "graph TD; A[Start]-->B[End]; B-->C[Finish]",
        "test_theme_forest.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_theme_forest_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--theme")
        .arg("forest")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("<svg"), "Output should contain SVG tag");
    assert!(svg.contains("Start"), "SVG should contain label 'Start'");
    assert!(svg.contains("Finish"), "SVG should contain label 'Finish'");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_theme_neutral() {
    let input = create_test_mmd(
        "graph LR; X-->Y; Y-->Z",
        "test_theme_neutral.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_theme_neutral_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--theme")
        .arg("neutral")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("<svg"), "Output should contain SVG tag");
    assert!(svg.contains("X"), "SVG should contain node X");
    assert!(svg.contains("Z"), "SVG should contain node Z");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_theme_default() {
    let input = create_test_mmd(
        "graph TD; A[Alpha]-->B[Beta]; B-->C[Gamma]; C-->D[Delta]",
        "test_theme_default.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_theme_default_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--theme")
        .arg("default")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("Alpha"), "SVG should contain label 'Alpha'");
    assert!(svg.contains("Delta"), "SVG should contain label 'Delta'");
    assert!(svg.contains("<svg"), "Output should contain SVG tag");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_svgid_filter() {
    let input = create_test_mmd(
        "graph TD; A-->B",
        "test_svgid_filter.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_svgid_filter_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--svgId")
        .arg("nonexistent")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    // SVG should still be valid XML even with empty content
    assert!(svg.contains("<svg"), "SVG should have svg tag");
    assert!(svg.contains("</svg>"), "SVG should have closing tag");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_output_format_svg() {
    let input = create_test_mmd(
        "graph TD; A-->B",
        "test_output_format_svg.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_output_format_svg_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--outputFormat")
        .arg("svg")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI exited with error: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("<svg"), "Output should contain SVG tag");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_stdin_state_diagram() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"stateDiagram\nA-->B\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "stateDiagram should render successfully: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("<svg"),
        "stdout should contain SVG: {}",
        stdout.len()
    );
}

#[test]
fn test_cli_stdin_er_diagram() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"erDiagram\nCUSTOMER ||--o{ ORDER : places\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "erDiagram should render successfully: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("<svg"),
        "stdout should contain SVG: {}",
        stdout.len()
    );
}

#[test]
fn test_cli_stdin_class_diagram() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"classDiagram\nclass Animal {\n}\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "classDiagram should render: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "stdout should contain SVG");
}

#[test]
fn test_cli_stdin_gantt_diagram() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"gantt\ntitle A Gantt\nsection S\nTask: 1, 2\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "gantt should render: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "stdout should contain SVG");
}

#[test]
fn test_cli_stdin_pie_diagram() {
    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"pie\n\"Dogs\" : 42\n\"Cats\" : 58\n")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "pie should render: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "stdout should contain SVG");
}

#[test]
fn test_cli_combined_flags() {
    let input = create_test_mmd(
        "graph TD; A[Start]-->B{Decision}; B-->|Yes|C[End]; B-->|No|D[Retry]",
        "test_combined_flags.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_combined_flags_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-t")
        .arg("dark")
        .arg("-w")
        .arg("1200")
        .arg("-H")
        .arg("800")
        .arg("-b")
        .arg("lightblue")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI should ignore unknown flags: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("Start"), "SVG should contain 'Start'");
    assert!(svg.contains("Decision"), "SVG should contain 'Decision'");
    assert!(svg.contains("End"), "SVG should contain 'End'");
    assert!(svg.contains("Retry"), "SVG should contain 'Retry'");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_background_color_hex() {
    let input = create_test_mmd(
        "graph TD; A[Hello]-->B[World]",
        "test_background_hex.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_background_hex_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("--backgroundColor")
        .arg("#ffeedd")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI should ignore --backgroundColor: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("Hello"), "SVG should contain 'Hello'");
    assert!(svg.contains("World"), "SVG should contain 'World'");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_empty_file_input() {
    let input = create_test_mmd("", "test_empty_file.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_empty_file_output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "Empty file should cause parse error"
    );
    assert!(
        !output.exists(),
        "Output should not be created for empty input"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "stderr should contain error message: {}",
        stderr
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_nonexistent_output_dir() {
    let input = create_test_mmd(
        "graph TD; A-->B",
        "test_nonexistent_dir.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("nonexistent_subdir_xyz")
        .join("output.svg");

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "CLI should fail when output directory does not exist"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "stderr should contain error: {}",
        stderr
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_very_long_file_path() {
    let long_name = format!("test_long_{}.mmd", "a".repeat(180));
    let input = create_test_mmd(
        "graph TD; A-->B",
        &long_name,
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join(format!("test_long_output_{}.svg", "b".repeat(180)));

    let result = Command::new(binary_path())
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI should handle long file paths: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG was not created for long path");
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_check_sequence_diagram() {
    let input = create_test_mmd(
        "sequenceDiagram\nAlice->>Bob: Hello\nBob-->>Alice: Hi",
        "test_check_sequence.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for sequenceDiagram: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_pie_diagram() {
    let input = create_test_mmd(
        "pie\n\"Alpha\" : 30\n\"Beta\" : 70",
        "test_check_pie.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for pie: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_class_diagram() {
    let input = create_test_mmd(
        "classDiagram\nclass Vehicle {\n}\n",
        "test_check_class.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for classDiagram: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_state_diagram() {
    let input = create_test_mmd(
        "stateDiagram\n[*] --> Idle\nIdle --> Active",
        "test_check_state.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for stateDiagram: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_er_diagram() {
    let input = create_test_mmd(
        "erDiagram\nCAR ||--o{ PART : contains",
        "test_check_er.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for erDiagram: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_check_gantt_diagram() {
    let input = create_test_mmd(
        "gantt\ntitle Test\nsection S\nTask1: t1, 1, 5d",
        "test_check_gantt.mmd",
    );

    let result = Command::new(binary_path())
        .arg("check")
        .arg(input.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "Check should pass for gantt: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        stdout.contains("Valid") || stdout.contains("valid"),
        "stdout should indicate valid: {}",
        stdout
    );
    cleanup(&[&input]);
}

#[test]
fn test_cli_fix_sequnce_typo() {
    let input = create_test_mmd(
        "sequnceDiagram\nAlice->>Bob: Hello",
        "test_fix_sequnce.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_fix_sequnce_output.mmd");

    let result = Command::new(binary_path())
        .arg("fix")
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "fix command should succeed: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Fixed output file should exist");
    let fixed = fs::read_to_string(&output).expect("Failed to read fixed file");
    assert!(
        fixed.contains("sequenceDiagram"),
        "sequnceDiagram should be fixed to sequenceDiagram"
    );
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_fix_partcipant_typo() {
    let input = create_test_mmd(
        "sequenceDiagram\npartcipant Alice",
        "test_fix_partcipant.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_fix_partcipant_output.mmd");

    let result = Command::new(binary_path())
        .arg("fix")
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "fix command should succeed: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Fixed output file should exist");
    let fixed = fs::read_to_string(&output).expect("Failed to read fixed file");
    assert!(
        fixed.contains("participant"),
        "partcipant should be fixed to participant"
    );
    cleanup(&[&input, &output]);
}

#[test]
fn test_cli_markdown_input_stdin() {
    let markdown = "\
Some text before.

```mermaid
graph TD; A-->B
```

Some text after.";

    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(markdown.as_bytes())
            .expect("Failed to write markdown to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "Markdown with mermaid block should render: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("<svg"), "stdout should contain SVG");
}

#[test]
fn test_cli_quiet_combined_stdin() {
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_quiet_stdin_output.svg");

    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .arg("--quiet")
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"graph LR; A-->B; B-->C")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "CLI should succeed with --quiet: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output SVG should be created");
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("A"), "SVG should contain node A");
    assert!(svg.contains("C"), "SVG should contain node C");
    cleanup(&[&output_path]);
}

#[test]
fn test_cli_jobs_multiple_files() {
    let input1 = create_test_mmd(
        "graph TD; A-->B",
        "test_jobs_file1.mmd",
    );
    let input2 = create_test_mmd(
        "graph LR; X-->Y",
        "test_jobs_file2.mmd",
    );
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_jobs_output.svg");

    // --jobs 4 placed after the first file but before the second
    let result = Command::new(binary_path())
        .arg(input1.to_str().unwrap())
        .arg("--jobs")
        .arg("4")
        .arg(input2.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("Failed to run CLI");

    assert!(
        result.status.success(),
        "CLI should render first file and ignore --jobs: {:?}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(output.exists(), "Output SVG should be created");
    let svg = fs::read_to_string(&output).expect("Failed to read SVG");
    assert!(svg.contains("A"), "SVG should contain node A from first file");
    assert!(svg.contains("B"), "SVG should contain node B from first file");
    cleanup(&[&input1, &input2, &output]);
}

#[test]
fn test_cli_show_fixes_with_stdin() {
    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_show_fixes_stdin_output.svg");

    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .arg("--show-fixes")
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    // Use content that triggers a fix (missing 'end')
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"graph TD; A-->B")
            .expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait for CLI");
    assert!(
        output.status.success(),
        "CLI should succeed with --show-fixes: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "Output SVG should be created");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("fix") || stdout.contains("end"),
        "stdout should mention fix: {}",
        stdout
    );
    let svg = fs::read_to_string(&output_path).expect("Failed to read SVG");
    assert!(svg.contains("<svg"), "SVG output should be valid");
    cleanup(&[&output_path]);
}

#[test]
fn test_cli_invalid_theme_as_file() {
    // --theme is not a recognized command/flag that accepts an argument
    // when placed first it is treated as a file path
    let result = Command::new(binary_path())
        .arg("--theme")
        .arg("nonexistent")
        .output()
        .expect("Failed to run CLI");

    assert!(
        !result.status.success(),
        "CLI should fail when --theme is interpreted as a file path"
    );
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error") || stderr.contains("error"),
        "stderr should contain error: {}",
        stderr
    );
    assert!(
        stderr.contains("--theme"),
        "stderr should mention the file that was not found"
    );
}


// ============================================================
// GitGraph CLI 测试
// ============================================================

#[test]
fn test_cli_gitgraph_from_stdin() {
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("test_gitgraph_stdin_output.svg");

    let mut child = Command::new(binary_path())
        .arg("--stdin")
        .arg("-o")
        .arg(output.to_str().unwrap())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn CLI");

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(b"gitGraph\n  commit\n  commit\n  commit\n")
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
    assert!(
        svg_content.contains("circle"),
        "GitGraph SVG should contain circle elements"
    );
    assert!(
        svg_content.contains("line"),
        "GitGraph SVG should contain line elements"
    );

    cleanup(&[&output]);
}


// ============================================================
// 新增图表 CLI 测试
// ============================================================

#[test]
fn test_cli_mindmap_from_file() {
    let input = create_test_mmd("mindmap\n  Root\n    Branch\n      Leaf", "cli_mindmap.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_mindmap_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Mindmap should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_timeline_from_file() {
    let input = create_test_mmd("timeline\n  2020: Event", "cli_timeline.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_timeline_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Timeline should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_journey_from_file() {
    let input = create_test_mmd("journey\n  section S\n  T:1:U", "cli_journey.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_journey_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Journey should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_venn_from_file() {
    let input = create_test_mmd("venn\n  a : A\n  b : B", "cli_venn.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_venn_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Venn should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_packet_from_file() {
    let input = create_test_mmd("packet\n  0-7: Header", "cli_packet.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_packet_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Packet should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_radar_from_file() {
    let input = create_test_mmd("radar\n  Speed: 50\n  Power: 80", "cli_radar.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_radar_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Radar should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_architecture_from_file() {
    let input = create_test_mmd("architecture\n  service api(API)", "cli_arch.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_arch_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Architecture should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_block_from_file() {
    let input = create_test_mmd("block\n  A\n    B", "cli_block.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_block_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Block should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_ishikawa_from_file() {
    let input = create_test_mmd("ishikawa\n  root R\n  category C\n    cause", "cli_ishi.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_ishi_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success(), "Ishikawa should render");
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

// ============================================================
// 更多图表 CLI 测试 - 标志组合
// ============================================================

#[test]
fn test_cli_sankey_from_file() {
    let input = create_test_mmd("sankey\n  A -> B: 100", "cli_sankey.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_sankey_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_quadrant_from_file() {
    let input = create_test_mmd("quadrantChart\n  title T\n  x-axis X\n  y-axis Y\n  P: [0.5,0.5]", "cli_quad.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_quad_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_c4_from_file() {
    let input = create_test_mmd("C4Context\n  Person(u, \"User\", \"\")", "cli_c4.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_c4_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_requirement_from_file() {
    let input = create_test_mmd("requirementDiagram\n  requirement R {\n    id: 1\n    text: test\n    risk: high\n    verifymethod: test\n  }", "cli_req.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_req_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_zenuml_from_file() {
    let input = create_test_mmd("zenuml\n  Alice->Bob: Hello", "cli_zenuml.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_zenuml_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_dark_theme_sequence() {
    let input = create_test_mmd("sequenceDiagram\n  A->B: Hi", "cli_dark_seq.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_dark_seq_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-t").arg("dark").arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_forest_theme_pie() {
    let input = create_test_mmd("pie\n  \"A\": 50\n  \"B\": 50", "cli_forest_pie.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_forest_pie_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-t").arg("forest").arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_width_height_sequence() {
    let input = create_test_mmd("sequenceDiagram\n  A->B: Hi", "cli_wh_seq.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_wh_seq_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-w").arg("1200").arg("-H").arg("800").arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_background_class() {
    let input = create_test_mmd("classDiagram\n  class A", "cli_bg_class.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_bg_class_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-b").arg("lightblue").arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}

#[test]
fn test_cli_quiet_gantt() {
    let input = create_test_mmd("gantt\n  Task: t1, 1, 5d", "cli_quiet_gantt.mmd");
    let output = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("cli_quiet_gantt_output.svg");
    let result = Command::new(binary_path()).arg(input.to_str().unwrap()).arg("-q").arg("-o").arg(output.to_str().unwrap()).output().expect("Failed to run CLI");
    assert!(result.status.success());
    if output.exists() { let _ = fs::remove_file(&output); }
    cleanup(&[&input]);
}
