# CI/CD Setup Guide

This document explains how to set up and use the CI/CD pipeline for the montycat_rust project.

## Overview

The project uses GitHub Actions for continuous integration and deployment with three workflows:

1. **CI Workflow** (`ci.yml`) - Automated testing and code quality checks
2. **Publish Workflow** (`publish.yml`) - Automated deployment to crates.io
3. **Security Workflow** (`security.yml`) - Dependency vulnerability scanning

---

## Prerequisites

### 1. GitHub Repository

Ensure your code is pushed to a GitHub repository.

### 2. Crates.io Account

1. Create an account at [crates.io](https://crates.io/)
2. Generate an API token:
   - Go to [Account Settings](https://crates.io/settings/tokens)
   - Click "New Token"
   - Give it a name (e.g., "GitHub Actions")
   - Copy the token (you won't see it again!)

### 3. Configure GitHub Secrets

Add your crates.io token to GitHub repository secrets:

1. Go to your GitHub repository
2. Navigate to **Settings** → **Secrets and variables** → **Actions**
3. Click **New repository secret**
4. Name: `CARGO_REGISTRY_TOKEN`
5. Value: Paste your crates.io API token
6. Click **Add secret**

---

## Workflows

### CI Workflow

**Triggers:**
- Push to `main`, `master`, or `develop` branches
- Pull requests to `main`, `master`, or `develop` branches

**What it does:**
- ✅ Tests on Ubuntu, macOS, and Windows
- ✅ Tests with stable and beta Rust versions
- ✅ Checks code formatting (`cargo fmt`)
- ✅ Runs linter (`cargo clippy`)
- ✅ Builds with all features
- ✅ Runs all tests with different feature combinations
- ✅ Generates code coverage report (Ubuntu only)
- ✅ Uploads coverage to Codecov (optional)

**Caching:**
- Cargo registry, git index, and build artifacts are cached for faster builds

---

### Publish Workflow

**Triggers:**
- Push of version tags (e.g., `v0.1.3`, `v1.0.0`)

**What it does:**
1. ✅ Verifies tag version matches `Cargo.toml` version
2. ✅ Runs all tests
3. ✅ Checks formatting and linting
4. ✅ Builds release binary
5. ✅ Publishes to crates.io
6. ✅ Creates GitHub release with auto-generated notes

**How to publish a new version:**

```bash
# 1. Update version in Cargo.toml
# Edit the version field, e.g., version = "0.1.3"

# 2. Commit the version change
git add Cargo.toml
git commit -m "Bump version to 0.1.3"

# 3. Create and push a version tag
git tag v0.1.3
git push origin main
git push origin v0.1.3

# 4. The workflow will automatically:
#    - Run tests
#    - Publish to crates.io
#    - Create a GitHub release
```

---

### Security Workflow

**Triggers:**
- Every Monday at 9:00 AM UTC (scheduled)
- Push to `main` or `master` branches
- Pull requests to `main` or `master` branches

**What it does:**
- ✅ Runs `cargo audit` to check for known security vulnerabilities
- ✅ Checks for outdated dependencies with `cargo outdated`

---

## Badge Setup

Add these badges to your `README.md`:

```markdown
[![CI](https://github.com/YOUR_USERNAME/montycat_rust/workflows/CI/badge.svg)](https://github.com/YOUR_USERNAME/montycat_rust/actions/workflows/ci.yml)
[![Security Audit](https://github.com/YOUR_USERNAME/montycat_rust/workflows/Security%20Audit/badge.svg)](https://github.com/YOUR_USERNAME/montycat_rust/actions/workflows/security.yml)
[![Crates.io](https://img.shields.io/crates/v/montycat.svg)](https://crates.io/crates/montycat)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/montycat_rust/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/montycat_rust)
```

Replace `YOUR_USERNAME` with your GitHub username.

---

## Codecov Setup (Optional)

To enable code coverage reporting:

1. Go to [codecov.io](https://codecov.io/)
2. Sign in with GitHub
3. Add your repository
4. No additional configuration needed - the CI workflow already uploads coverage

---

## Local Testing

Before pushing, you can run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --all-targets --all-features -- -D warnings

# Run tests
cargo test --all-features

# Build
cargo build --release --all-features

# Security audit
cargo install cargo-audit
cargo audit

# Check outdated dependencies
cargo install cargo-outdated
cargo outdated
```

---

## Troubleshooting

### Publish fails with "already uploaded"

If you've already published a version, you cannot republish it. Increment the version number in `Cargo.toml` and create a new tag.

### Tests fail in CI but pass locally

- Check if you're using the same Rust version
- Ensure all features are enabled: `cargo test --all-features`
- Check platform-specific issues (Windows vs Unix paths, etc.)

### Token authentication fails

- Verify the `CARGO_REGISTRY_TOKEN` secret is set correctly
- Ensure the token hasn't expired
- Generate a new token if needed

### Version mismatch error

The tag version must match the version in `Cargo.toml`. For example:
- Tag: `v0.1.3`
- Cargo.toml: `version = "0.1.3"`

---

## Workflow Files

- [`.github/workflows/ci.yml`](file:///.github/workflows/ci.yml) - CI workflow
- [`.github/workflows/publish.yml`](file:///.github/workflows/publish.yml) - Publish workflow
- [`.github/workflows/security.yml`](file:///.github/workflows/security.yml) - Security workflow

---

## Best Practices

1. **Always test locally** before pushing
2. **Use semantic versioning** (MAJOR.MINOR.PATCH)
3. **Write meaningful commit messages** for releases
4. **Review the auto-generated release notes** before publishing
5. **Monitor security audit results** and update dependencies regularly
6. **Keep dependencies up to date** to avoid security vulnerabilities

---

## Release Checklist

- [ ] Update version in `Cargo.toml`
- [ ] Update `CHANGELOG.md` (if you have one)
- [ ] Run tests locally: `cargo test --all-features`
- [ ] Run clippy: `cargo clippy --all-targets --all-features`
- [ ] Commit version changes
- [ ] Create and push version tag
- [ ] Verify GitHub Actions workflow succeeds
- [ ] Check crates.io for successful publish
- [ ] Verify GitHub release was created
