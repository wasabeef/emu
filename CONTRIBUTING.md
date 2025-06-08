# Contributing to Emu

Thank you for your interest in contributing to Emu! This document provides guidelines and information for contributors.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Submitting Changes](#submitting-changes)
- [Review Process](#review-process)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## How to Contribute

### Types of Contributions

We welcome various types of contributions:

- **Bug Reports**: Help us identify and fix issues
- **Feature Requests**: Suggest new features or improvements
- **Code Contributions**: Bug fixes, new features, performance improvements
- **Documentation**: Improve or expand documentation
- **Testing**: Add or improve test coverage
- **Platform Support**: Enhance Windows/Linux support

### Before You Start

1. **Check existing issues**: Look for existing bug reports or feature requests
2. **Create an issue**: For new features or significant changes, create an issue first to discuss
3. **Fork the repository**: Create a personal fork to work on changes
4. **Create a branch**: Use descriptive branch names like `feature/device-creation` or `fix/android-state-detection`

## Development Setup

### Prerequisites

- **Rust**: Version 1.70 or later
- **Git**: For version control
- **Android SDK**: For Android development testing
- **Xcode** (macOS): For iOS development testing

### Initial Setup

```bash
# Clone your fork
git clone https://github.com/your-username/emu.git
cd emu

# Add upstream remote
git remote add upstream https://github.com/wasabeef/emu.git

# Install dependencies and build
cargo build

# Run tests to ensure everything works
cargo test

# Try running the application
cargo run
```

### Development Tools

#### Recommended Tools
```bash
# Install cargo-watch for live reload during development
cargo install cargo-watch

# Install clippy for linting
rustup component add clippy

# Install rustfmt for formatting
rustup component add rustfmt
```

#### Useful Commands
```bash
# Run with live reload
cargo watch -x run

# Run tests with live reload
cargo watch -x test

# Format code
cargo fmt

# Lint code
cargo clippy

# Run specific test
cargo test test_name

# Run test with output
cargo test test_name -- --nocapture
```

## Project Structure

### Core Modules

```
src/
├── app/                 # Application core
│   ├── mod.rs          # Main app logic, event loop
│   ├── state.rs        # AppState, device state management
│   ├── events.rs       # Event type definitions
│   └── actions.rs      # User action handlers
├── managers/           # Platform-specific device management
│   ├── common.rs       # DeviceManager trait
│   ├── android.rs      # Android AVD management
│   └── ios.rs          # iOS Simulator management
├── models/             # Data structures and types
│   ├── device.rs       # Device models (AndroidDevice, IosDevice)
│   ├── error.rs        # Error types and handling
│   └── platform.rs     # Platform enums
├── ui/                 # Terminal user interface
│   ├── render.rs       # Main rendering logic
│   ├── theme.rs        # Color themes and styling
│   └── widgets.rs      # Custom UI widgets
└── utils/              # Shared utilities
    ├── command.rs      # Command execution helpers
    └── logger.rs       # Logging utilities
```

### Key Design Patterns

#### Async Trait Pattern
All device managers implement the `DeviceManager` trait with async methods:

```rust
#[async_trait]
pub trait DeviceManager: Send + Sync + Clone {
    async fn list_devices(&self) -> Result<Vec<Device>>;
    async fn start_device(&self, id: &str) -> Result<()>;
    // ... other methods
}
```

#### State Management Pattern
Centralized state with thread-safe access:

```rust
pub struct App {
    state: Arc<Mutex<AppState>>,
    // ... other fields
}
```

#### Error Handling Pattern
Use `anyhow` for error propagation and `thiserror` for custom errors:

```rust
#[derive(thiserror::Error, Debug)]
pub enum DeviceError {
    #[error("Device not found: {name}")]
    NotFound { name: String },
    // ... other variants
}
```

## Coding Standards

### Rust Style Guidelines

#### Formatting
- Use `cargo fmt` for automatic formatting
- Follow standard Rust naming conventions
- Use 4 spaces for indentation

#### Code Quality
- Run `cargo clippy` and fix all warnings
- Use meaningful variable and function names
- Add documentation comments for public APIs
- Keep functions focused and reasonably sized

#### Error Handling
- Use `Result<T, E>` for fallible operations
- Provide helpful error messages with context
- Use `anyhow::Context` to add context to errors

```rust
// Good
fn parse_device_config(content: &str) -> Result<DeviceConfig> {
    serde_json::from_str(content)
        .with_context(|| format!("Failed to parse device config: {}", content))
}

// Avoid
fn parse_device_config(content: &str) -> DeviceConfig {
    serde_json::from_str(content).unwrap()
}
```

#### Async Programming
- Use `async/await` consistently
- Avoid blocking operations in async contexts
- Use `tokio::spawn` for background tasks
- Handle task cancellation properly

```rust
// Good
let handle = tokio::spawn(async move {
    // Long-running background task
});

// Cancel task when needed
if let Some(handle) = self.background_task.take() {
    handle.abort();
}
```

### Documentation Standards

#### Code Comments
- Use `///` for public API documentation
- Use `//` for implementation comments
- Include examples for complex functions

```rust
/// Creates a new Android Virtual Device with the specified configuration.
/// 
/// # Arguments
/// * `config` - Device configuration including name, API level, and hardware specs
/// 
/// # Returns
/// * `Ok(())` - Device created successfully
/// * `Err(DeviceError)` - Creation failed
/// 
/// # Example
/// ```rust
/// let config = DeviceConfig::new("Pixel_7_API_31", "pixel_7", "31");
/// manager.create_device(&config).await?;
/// ```
pub async fn create_device(&self, config: &DeviceConfig) -> Result<()> {
    // Implementation
}
```

#### Commit Messages
Use conventional commit format:

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Formatting changes
- `refactor`: Code refactoring
- `test`: Adding or modifying tests
- `perf`: Performance improvements

Examples:
```
feat(android): add device creation with custom RAM/storage
fix(ios): resolve simulator state detection issue
docs(readme): update installation instructions
test(device): add comprehensive device lifecycle tests
```

## Testing Guidelines

### Test Categories

#### Unit Tests
Located in the same file as the code being tested:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_name_validation() {
        assert!(is_valid_device_name("Valid_Name_123"));
        assert!(!is_valid_device_name("Invalid Name!"));
    }
}
```

#### Integration Tests
Located in `tests/` directory:

```rust
// tests/device_creation_test.rs
use emu::managers::AndroidManager;

#[tokio::test]
async fn test_complete_device_lifecycle() {
    let manager = AndroidManager::new().unwrap();
    // Test device creation, start, stop, delete
}
```

#### Performance Tests
Validate performance requirements:

```rust
#[tokio::test]
async fn test_startup_performance() {
    let start = std::time::Instant::now();
    let app = App::new(Config::default()).await?;
    let duration = start.elapsed();
    
    assert!(duration < std::time::Duration::from_millis(150));
}
```

### Testing Best Practices

1. **Write tests first** for new features (TDD)
2. **Test error conditions** as well as success paths
3. **Use meaningful test names** that describe what is being tested
4. **Mock external dependencies** using the `mockall` crate
5. **Test async code** using `#[tokio::test]`
6. **Validate performance** for critical paths

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test device_creation_test

# Run with output
cargo test -- --nocapture

# Run performance tests
cargo test startup_performance_test -- --nocapture

# Run tests for specific module
cargo test android::
```

## Submitting Changes

### Pull Request Process

1. **Update your fork**:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes** following the coding standards

4. **Test your changes**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```

5. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat(scope): description of changes"
   ```

6. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create a Pull Request** on GitHub

### Pull Request Requirements

#### Checklist
- [ ] Code follows the style guidelines
- [ ] Self-review of the code
- [ ] Tests added for new functionality
- [ ] All tests pass
- [ ] Documentation updated (if applicable)
- [ ] No clippy warnings
- [ ] Code is formatted with `cargo fmt`

#### PR Description Template
```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
Describe the tests you ran to verify your changes

## Screenshots (if applicable)
Add screenshots for UI changes

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] Documentation updated
```

## Review Process

### What We Look For

1. **Code Quality**
   - Clean, readable code
   - Proper error handling
   - Adequate test coverage
   - Performance considerations

2. **Design Consistency**
   - Follows existing patterns
   - Maintains architectural principles
   - Proper separation of concerns

3. **User Experience**
   - Intuitive interfaces
   - Clear error messages
   - Responsive performance

### Review Timeline

- **Initial Response**: Within 2-3 days
- **Full Review**: Within 1 week
- **Follow-up**: Within 2-3 days of updates

### Addressing Feedback

1. **Read feedback carefully** and ask questions if unclear
2. **Make requested changes** in additional commits
3. **Update tests** if needed
4. **Respond to comments** when changes are made
5. **Request re-review** when ready

## Getting Help

### Communication Channels

- **Issues**: For bug reports and feature requests
- **Discussions**: For questions and general discussion
- **Pull Requests**: For code review discussions

### Development Questions

If you need help with:
- **Setup Issues**: Create an issue with the "help wanted" label
- **Architecture Questions**: Start a discussion
- **Code Review**: Ask in the PR comments

## Recognition

Contributors are recognized in several ways:
- Listed in the project's contributors
- Mentioned in release notes for significant contributions
- Added to the CONTRIBUTORS.md file

Thank you for contributing to Emu! 🎉