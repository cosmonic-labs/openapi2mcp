# GitHub Configuration

This directory contains GitHub-specific configuration files for the openapi2mcp project.

## Workflows

### CI Pipeline (`workflows/ci.yml`)

The main continuous integration pipeline that runs on every push and pull request to the `main` branch.

**Features:**
- **Multi-Rust Testing**: Tests against stable, beta, and nightly Rust versions
- **Code Quality**: Formatting checks (`cargo fmt`) and linting (`cargo clippy`)
- **Comprehensive Testing**: Unit tests, integration tests, and example validation
- **Multi-Platform Builds**: Linux, Windows, and macOS
- **Security Auditing**: Dependency vulnerability scanning with `cargo audit`
- **Code Coverage**: Coverage reporting with `cargo-llvm-cov` and Codecov integration
- **Artifact Generation**: Binary artifacts for all platforms

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch

### Release Pipeline (`workflows/release.yml`)

Automated release process triggered by version tags.

**Features:**
- **Multi-Platform Binaries**: Builds release binaries for Linux, macOS, and Windows
- **GitHub Releases**: Automatically creates GitHub releases with binaries
- **Cargo Publishing**: Publishes to crates.io (requires `CARGO_REGISTRY_TOKEN` secret)
- **Release Notes**: Auto-generated release notes with feature highlights

**Triggers:**
- Tags matching `v*` pattern (e.g., `v1.0.0`)

### PR Validation (`workflows/pr-validation.yml`)

Additional validation for pull requests to ensure generated code quality.

**Features:**
- **Generated Code Testing**: Validates that generated TypeScript and Rust code compiles
- **Breaking Change Detection**: Uses `cargo-semver-checks` to detect API breaking changes
- **All Examples Validation**: Tests all example OpenAPI specifications
- **Artifact Upload**: Uploads generated code samples for review

**Triggers:**
- Pull requests to `main` branch

### Performance Benchmarks (`workflows/benchmark.yml`)

Performance monitoring and regression detection.

**Features:**
- **Parsing Benchmarks**: Measures OpenAPI parsing performance
- **Generation Benchmarks**: Measures code generation performance for both targets
- **PR Comments**: Posts benchmark results as comments on pull requests
- **Trend Tracking**: Helps identify performance regressions

**Triggers:**
- Push to `main` branch
- Pull requests to `main` branch

## Security Considerations

- **Secrets Management**: Uses GitHub secrets for sensitive tokens
- **Permission Scoping**: Minimal required permissions for each workflow
- **Dependency Scanning**: Regular security audits with `cargo audit`
- **Code Review**: Required reviews for all changes
