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

3. Update `CHANGELOG.md` (optional - automated release notes are generated):
   - The release process will automatically generate release notes using git-cliff
   - Manual updates are only needed for major releases or special announcements

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
   - Generate release notes from commit history using git-cliff
   - Build binaries for all platforms
   - Create a GitHub release with auto-generated release notes
   - Upload the binaries

3. Homebrew formula update:
   - The release workflow automatically updates the formula
   - It calculates SHA256 for all platform binaries
   - Creates a commit in wasabeef/homebrew-emu-tap
   - No manual intervention required

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

## Commit Conventions

Use [Conventional Commits](https://www.conventionalcommits.org/) for automatic changelog generation:
- `feat:` New features
- `fix:` Bug fixes
- `docs:` Documentation changes
- `perf:` Performance improvements
- `refactor:` Code refactoring
- `test:` Test additions/changes
- `chore:` Maintenance tasks

Example:
```bash
git commit -m "feat: add iOS simulator log streaming"
git commit -m "fix: prevent crash on small terminal sizes"
git commit -m "docs: update installation instructions"
```