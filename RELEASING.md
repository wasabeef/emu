# Releasing Emu

This document describes the release process for Emu.

## Prerequisites

1. Ensure all tests are passing:
   ```bash
   cargo test --all-features
   cargo fmt -- --check
   cargo clippy --all-features -- -D warnings
   ```

2. Update version in `Cargo.toml`:
   ```toml
   [package]
   version = "0.1.0"  # Update this
   ```

3. Update `CHANGELOG.md`:
   - Move items from `[Unreleased]` to a new version section
   - Add release date
   - Ensure all changes are documented

4. Commit the version bump:
   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "chore: bump version to v0.1.0"
   ```

## Creating a Release

1. Create and push a tag:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. The GitHub Actions workflow will automatically:
   - Build binaries for all platforms
   - Create a GitHub release
   - Upload the binaries

3. Update the Homebrew formula:
   - The release workflow will attempt to update the tap automatically
   - If manual update is needed:
     ```bash
     # In wasabeef/homebrew-tap repository
     brew bump-formula-pr --url=https://github.com/wasabeef/emu/archive/v0.1.0.tar.gz emu
     ```

## Post-Release

1. Verify the release on GitHub
2. Test installation via Homebrew:
   ```bash
   brew tap wasabeef/tap
   brew install emu
   ```

3. Announce the release:
   - Twitter/X
   - Reddit (r/rust, r/androiddev)
   - Hacker News (if significant release)

## Version Numbering

We follow [Semantic Versioning](https://semver.org/):
- MAJOR version for incompatible API changes
- MINOR version for backwards-compatible functionality
- PATCH version for backwards-compatible bug fixes