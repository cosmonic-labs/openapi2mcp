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

### Auto-Labeling (`workflows/labeler.yml`)

Automatically labels pull requests based on changed files.

**Features:**
- **Smart Labeling**: Labels based on file paths and content
- **Category Detection**: Identifies OpenAPI, MCP, CLI, test, and documentation changes
- **Title-Based Labels**: Detects bug fixes, features, and breaking changes from PR titles

**Triggers:**
- Pull request events (opened, edited, synchronized)

## Issue Templates

### Bug Report (`ISSUE_TEMPLATE/bug_report.yml`)
Structured template for bug reports including:
- Version information
- Target language
- Reproduction steps
- OpenAPI specification
- System information

### Feature Request (`ISSUE_TEMPLATE/feature_request.yml`)
Template for feature requests including:
- Feature category
- Problem description
- Proposed solution
- Priority level
- Implementation willingness

### Documentation (`ISSUE_TEMPLATE/documentation.yml`)
Template for documentation improvements including:
- Documentation type
- Issue description
- Location information
- Suggested improvements

## Pull Request Template

Comprehensive PR template ensuring:
- Clear description and issue linking
- Type of change classification
- Testing checklist
- Generated code validation
- Documentation updates
- Code quality checks

## Dependabot Configuration (`dependabot.yml`)

Automated dependency updates for:
- **Rust Dependencies**: Weekly Cargo dependency updates
- **GitHub Actions**: Weekly workflow dependency updates
- **Smart Scheduling**: Updates on Mondays at 9 AM
- **Automatic Labeling**: Proper labels and assignees

## Labeler Configuration (`labeler.yml`)

File-based automatic labeling rules:
- **Source Code**: Labels based on modified Rust files
- **Examples**: Detects OpenAPI specification changes
- **Documentation**: Identifies README and doc changes
- **CI/CD**: Labels workflow modifications
- **Smart Detection**: Title-based feature/bug detection

## Security Considerations

- **Secrets Management**: Uses GitHub secrets for sensitive tokens
- **Permission Scoping**: Minimal required permissions for each workflow
- **Dependency Scanning**: Regular security audits with `cargo audit`
- **Code Review**: Required reviews for all changes

## Required Secrets

For full functionality, configure these repository secrets:

- `CARGO_REGISTRY_TOKEN`: For publishing to crates.io
- `CODECOV_TOKEN`: For code coverage reporting (optional)

## Maintenance

The CI/CD configuration is designed to be:
- **Self-Maintaining**: Dependabot keeps dependencies current
- **Scalable**: Easy to add new checks and validations
- **Reliable**: Comprehensive error handling and fallbacks
- **Fast**: Efficient caching and parallel execution