# Contributing Guide

> How to contribute to Highlander Forge Blade.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Testing](#testing)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

---

## Code of Conduct

This project adheres to a code of conduct:

- Be respectful and inclusive
- Welcome newcomers and help them learn
- Focus on constructive criticism
- Respect differing viewpoints and experiences
- Prioritize user safety and security

---

## Getting Started

### Prerequisites

- Rust 1.78+ (install via [rustup](https://rustup.rs/))
- Git
- Windows 10/11 (for full testing) or Linux (for unit tests with mocks)
- PowerShell 5.1+ (for Windows integration)

### Fork and Clone

```bash
# Fork on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/highlander-forge-blade.git
cd highlander-forge-blade

# Add upstream remote
git remote add upstream https://github.com/highlanderviralcenter-lab/highlander-forge-blade.git
```

### Build

```bash
# Debug build (TUI only)
cargo build --features tui

# Release build (all features)
cargo build --release --all-features

# Run tests
cargo test --all-features
```

---

## Development Environment

### VS Code Setup

```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["tui"],
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.extraArgs": ["--all-features"],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ / RustRover

1. Import project as Cargo project
2. Set `Build → Build Options → Features` to `tui`
3. Enable `org.rust.cargo.emulate.terminal` for colored output

### Pre-commit Hooks

```bash
# Install cargo hooks
cargo install cargo-husky
cargo husky install

# Or manually:
# .git/hooks/pre-commit
cargo fmt -- --check || exit 1
cargo clippy --all-features -- -D warnings || exit 1
cargo test --lib --features tui || exit 1
```

---

## Testing

### Test Structure

```
tests/
├── integration/
│   ├── audit_flow.rs        # Audit with mocks (Linux OK)
│   ├── state_migration.rs   # State v0→v1 migration
│   └── headless_output.rs   # Headless mode output validation
└── fixtures/
    ├── state_v0.json        # Legacy state for migration test
    └── state_v1.json        # Current state format
```

### Running Tests

```bash
# Unit tests (Linux compatible, uses mocks)
cargo test --lib --features tui

# Integration tests (Linux compatible)
cargo test --test integration --features tui

# All tests (requires Windows for platform tests)
cargo test --all-features

# Specific test
cargo test test_audit_with_mocks -- --nocapture

# Benchmarks
cargo bench
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::MockSystemInfoProvider;

    #[tokio::test]
    async fn test_feature_description() {
        // Arrange
        let mut mock = MockSystemInfoProvider::new();
        mock.expect_cpu()
            .times(1)
            .returning(|| Ok(CpuInfo::default()));

        // Act
        let result = function_under_test(&mock).await;

        // Assert
        assert!(result.is_ok());
    }
}
```

---

## Documentation

### Inline Documentation

```rust
/// Collects CPU information from the system.
///
/// # Arguments
///
/// * `provider` - System information provider (WMI or mock)
///
/// # Returns
///
/// `Ok(CpuInfo)` on success, `Err(CoreError::Wmi)` on failure.
///
/// # Examples
///
/// ```
/// use hfb::core::traits::SystemInfoProvider;
///
/// # async fn example(provider: &dyn SystemInfoProvider) -> Result<(), hfb::core::error::CoreError> {
/// let cpu = provider.cpu().await?;
/// println!("CPU: {}", cpu.name);
/// # Ok(())
/// # }
/// ```
pub async fn collect_cpu(provider: &dyn SystemInfoProvider) -> Result<CpuInfo, CoreError> {
    // ...
}
```

### Architecture Decision Records (ADRs)

For significant architectural decisions, create an ADR in `docs/decisions/`:

```markdown
# ADR-NNN: Title

## Status

Accepted / Proposed / Deprecated / Superseded

## Context

What is the issue that we're seeing that is motivating this decision?

## Decision

What is the change that we're proposing or have agreed to implement?

## Consequences

What becomes easier or more difficult to do and any risks introduced?

## Alternatives Considered

What other options were evaluated and why were they rejected?
```

---

## Pull Request Process

1. **Create an issue** (or comment on existing) to discuss the change
2. **Fork and branch** from `develop`: `git checkout -b feature/description`
3. **Make changes** with tests and documentation
4. **Run checks** locally:
   ```bash
   cargo fmt
   cargo clippy --all-features -- -D warnings
   cargo test --all-features
   ```
5. **Push** and create PR to `develop`
6. **Address review** feedback
7. **Merge** (squash for features, merge commit for releases)

### PR Checklist

- [ ] Issue referenced in description
- [ ] Tests added/updated
- [ ] Documentation updated (if API changes)
- [ ] CHANGELOG.md updated (if user-facing)
- [ ] `cargo fmt` applied
- [ ] `cargo clippy` passes
- [ ] CI passes
- [ ] No merge conflicts

---

## Release Process

### For Maintainers

1. **Create release branch** from `develop`
2. **Update version** in `Cargo.toml`
3. **Update CHANGELOG.md**
4. **Run full test suite** on Windows
5. **Build release binary** and sign
6. **Create PR** to `main`
7. **After merge**, tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
8. **Push tag**: `git push origin vX.Y.Z`
9. **GitHub Actions** creates release automatically
10. **Verify** release assets and signatures

---

*Last updated: 2026-06-20 | Document version: 1.0*
