## ADDED Requirements

### Requirement: GitHub Actions test workflow
The system SHALL automatically run tests on every push and pull request to verify code quality.

#### Scenario: Workflow triggers on push
- **WHEN** code is pushed to any branch
- **THEN** GitHub Actions `test` workflow automatically runs and executes all unit and integration tests

#### Scenario: Workflow displays in PR checks
- **WHEN** a pull request is created or updated
- **THEN** the `test` workflow runs and results appear in the PR checks, blocking merge if tests fail

#### Scenario: Caching for faster builds
- **WHEN** test workflow runs
- **THEN** Cargo cache is used to speed up compilation (no rebuilding unchanged dependencies)

### Requirement: GitHub Actions build workflow
The system SHALL automatically compile release binaries on every push to verify cross-platform compatibility.

#### Scenario: Build on Linux
- **WHEN** code is pushed
- **THEN** GitHub Actions compiles a release binary for x86_64-unknown-linux-gnu on ubuntu-latest

#### Scenario: Build on macOS
- **WHEN** code is pushed
- **THEN** GitHub Actions compiles release binaries for both x86_64-apple-darwin and aarch64-apple-darwin on macos-latest

#### Scenario: Build on Windows
- **WHEN** code is pushed
- **THEN** GitHub Actions compiles a release binary for x86_64-pc-windows-gnu on windows-latest

#### Scenario: Build artifacts are available
- **WHEN** a build completes successfully
- **THEN** the binary artifact is available for download from the workflow run

### Requirement: GitHub Actions release workflow
The system SHALL automatically create GitHub Releases with compiled binaries when a version tag is pushed.

#### Scenario: Release on version tag
- **WHEN** a git tag matching `v*` (e.g., `v0.1.0`) is pushed
- **THEN** GitHub Actions triggers the release workflow to compile binaries for all platforms

#### Scenario: Binary naming convention
- **WHEN** release workflow compiles binaries
- **THEN** each binary is named `mermaid-cli-<version>-<target>` (e.g., `mermaid-cli-v0.1.0-x86_64-unknown-linux-gnu`)

#### Scenario: Release created with artifacts
- **WHEN** all platform binaries are compiled successfully
- **THEN** GitHub Actions creates a GitHub Release with tag name as the version, includes all compiled binaries, and marks it as a release (not a pre-release)

#### Scenario: Release download
- **WHEN** a GitHub Release is created
- **THEN** users can download the compiled binaries directly from the release page without needing to build from source

### Requirement: Cross-platform compatibility
The system SHALL compile and run successfully on Linux, macOS, and Windows without platform-specific code.

#### Scenario: Consistent behavior across platforms
- **WHEN** the same Mermaid code is rendered on Linux, macOS, and Windows
- **THEN** all platforms produce identical SVG output

#### Scenario: CLI arguments work consistently
- **WHEN** CLI commands are run on any platform (with appropriate path separators)
- **THEN** `-o`, `--stdin`, `--help` flags work identically on all platforms
