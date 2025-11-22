# Contributing to AGEniX

Thank you for your interest in contributing to AGEniX! This document provides guidelines for contributing to the AGEniX ecosystem.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Project Structure](#project-structure)
4. [Development Workflow](#development-workflow)
5. [Documentation](#documentation)
6. [Testing](#testing)
7. [Security](#security)
8. [License](#license)

---

## Code of Conduct

We are committed to providing a welcoming and inclusive environment. Please be respectful and constructive in all interactions.

**Expected behavior:**
- Be respectful and professional
- Welcome newcomers and help them get started
- Focus on what is best for the project and community
- Show empathy towards other community members

**Unacceptable behavior:**
- Harassment, discrimination, or personal attacks
- Trolling, insulting/derogatory comments
- Public or private harassment
- Publishing others' private information without permission

---

## Getting Started

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/) (1.70+ required)
- **Git**: For version control
- **GitHub account**: For pull requests

### Repository Organization

The AGEniX ecosystem consists of multiple repositories:

| Repository | Purpose | Language |
|------------|---------|----------|
| `agenix` | Central documentation, specs, contracts | Markdown |
| `agx` | Planner/orchestrator CLI | Rust |
| `agq` | Queue manager and scheduler | Rust |
| `agw` | Worker execution engine | Rust |
| `agx-ocr` | OCR Agentic Unit | Rust |
| `agx-*` | Future Agentic Units | Rust |

### Choosing Where to Contribute

**Documentation improvements** → `agenix` repository

**Core architecture changes** → `agenix` repository (ADRs, specs, then implementation)

**Planner features** → `agx` repository

**Queue/scheduling features** → `agq` repository

**Worker features** → `agw` repository

**New Agentic Unit** → Create new `agx-*` repository

---

## Project Structure

### Central Documentation (agenix)

Cross-cutting documentation lives here:

```
agenix/
├── docs/
│   ├── architecture/    # System design, execution layers, schemas
│   ├── api/            # RESP protocol, AGQ endpoints, worker registration
│   ├── adr/            # Architecture Decision Records
│   ├── au-specs/       # Agentic Unit contract specifications
│   ├── development/    # Testing, security guidelines
│   ├── deployment/     # Deployment patterns
│   ├── planning/       # Planner models, Echo/Delta
│   ├── roadmap/        # Project roadmap
│   ├── tools/          # Tool contracts (--describe spec)
│   └── zero-trust/     # Security model
├── specs/              # JSON schemas (plan, describe, registry)
└── website/            # Public documentation site
```

### Component Repositories

Each component repo has:

```
component/
├── README.md           # Component-specific overview
├── CLAUDE.md           # AI agent development guidelines (repo-specific)
├── src/                # Rust source code
├── tests/              # Integration tests
├── docs/               # Implementation-specific docs (link to agenix for shared)
├── LICENSE-MIT         # MIT license
├── LICENSE-APACHE      # Apache 2.0 license
└── Cargo.toml          # Rust dependencies
```

---

## Development Workflow

### 1. Find or Create an Issue

- Check existing issues in the relevant repository
- For bugs: Provide reproduction steps, expected vs actual behavior
- For features: Describe use case, proposed solution
- For questions: Tag with `question` label

### 2. Fork and Clone

```bash
# Fork via GitHub UI, then:
git clone https://github.com/YOUR_USERNAME/REPO_NAME.git
cd REPO_NAME
git remote add upstream https://github.com/agenix-sh/REPO_NAME.git
```

### 3. Create a Branch

```bash
# Branch naming convention:
# - feature/short-description
# - bugfix/issue-number-short-description
# - docs/short-description

git checkout -b feature/add-worker-pooling
```

### 4. Make Changes

**For Rust code:**
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` (no warnings allowed)
- Add tests for new functionality
- Update documentation

**For documentation:**
- Use clear, concise language
- Provide examples where helpful
- Link to related documentation
- Follow existing structure

**For ADRs:**
- Use the [ADR template](docs/adr/template.md)
- Explain context, decision, and consequences
- Consider alternatives
- Link to related decisions

### 5. Test Your Changes

**Rust projects:**
```bash
# Format code
cargo fmt

# Check for warnings
cargo clippy -- -D warnings

# Run tests
cargo test

# Run security audit
cargo audit
```

**Documentation:**
- Check markdown formatting
- Verify links work
- Ensure examples are accurate

### 6. Commit Your Changes

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
git add .
git commit -m "feat(agq): add worker pooling support

Implements worker pool management with configurable pool sizes.

- Add WorkerPool struct
- Implement pool size configuration
- Add tests for pool behavior
- Update documentation

Closes #123"
```

**Commit message format:**
```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Build/tooling changes

### 7. Push and Create Pull Request

```bash
git push origin feature/add-worker-pooling
```

Then create a pull request via GitHub:

**PR Title:** Same as commit message subject
**PR Description:**
```markdown
## Summary
Brief description of changes

## Changes
- Bullet list of specific changes

## Testing
- How you tested the changes
- Test coverage added

## Related Issues
Closes #123
```

### 8. Code Review

- Address review comments
- Push additional commits to same branch
- Request re-review when ready
- Be patient and respectful

### 9. Merge

Once approved:
- PR will be squash-merged to main
- Your branch will be deleted
- Celebrate! 🎉

---

## Documentation

### When to Update Documentation

**Always update documentation when:**
- Adding new features
- Changing API contracts
- Modifying architecture
- Adding new configuration options
- Fixing significant bugs

### Documentation Types

**Architecture docs** (`agenix/docs/architecture/`)
- System design changes
- Execution model updates
- Component boundaries

**API docs** (`agenix/docs/api/`)
- New RESP commands
- Protocol changes
- Endpoint specifications

**ADRs** (`agenix/docs/adr/`)
- Significant architectural decisions
- Trade-off analyses
- Alternative considerations

**Component docs** (component repos)
- Implementation details
- Setup instructions
- Troubleshooting

### Documentation Style

- **Clear and concise**: Avoid jargon, explain acronyms
- **Examples**: Show, don't just tell
- **Structure**: Use headings, lists, tables
- **Links**: Cross-reference related docs
- **Code blocks**: Use appropriate syntax highlighting

---

## Testing

### Testing Philosophy

AGEniX follows Test-Driven Development (TDD):

1. Write tests first
2. Implement functionality
3. Refactor
4. Repeat

### Test Coverage Requirements

- **Minimum**: 80% code coverage
- **Security-critical code**: 100% coverage
- **Public APIs**: Full test coverage
- **Edge cases**: Explicitly tested

### Test Types

**Unit Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_registration() {
        let worker = Worker::new("test-worker");
        assert!(worker.register().is_ok());
    }
}
```

**Integration Tests**
```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_full_job_lifecycle() {
    // Test complete flow: submit → queue → execute → complete
}
```

**Security Tests**
```rust
#[test]
fn test_command_injection_prevention() {
    let malicious_input = "worker; rm -rf /";
    assert!(validate_worker_id(malicious_input).is_err());
}
```

See [Testing Strategy](docs/development/testing-strategy.md) for comprehensive guidelines.

---

## Security

### Security-First Development

AGEniX handles sensitive data and executes user-provided plans. Security is paramount.

### Security Checklist

Before submitting a PR, verify:

- [ ] No user input flows to system commands without validation
- [ ] All input is validated and sanitized
- [ ] Authentication checks are in place
- [ ] No secrets in logs or error messages
- [ ] Timeouts set on all I/O operations
- [ ] Integer overflow cannot occur
- [ ] Deserialization is safe and bounded
- [ ] No `unsafe` code (or justified and documented)
- [ ] Constant-time comparison for secrets
- [ ] Error messages don't leak sensitive data

### Reporting Security Vulnerabilities

**DO NOT** open a public issue for security vulnerabilities.

Instead, email: `security@agenix.sh` (or create a private security advisory on GitHub)

Include:
- Description of vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours.

See [Security Guidelines](docs/development/security-guidelines.md) for detailed security practices.

---

## License

### Dual Licensing

All AGEniX software is dual-licensed under:

- **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- **MIT license** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

Users may choose which license to apply.

### Contribution Agreement

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

This means:
- You retain copyright to your contributions
- You grant dual license (MIT and Apache-2.0) to the project
- No additional CLA required
- Your contributions will be available under both licenses

### AI Model Licensing

AI model weights (if contributed) require separate licensing. See [ADR-0004](docs/adr/0004-licensing-strategy.md) for details.

---

## Development Resources

### Key Documentation

- [System Overview](docs/architecture/system-overview.md) - Architecture overview
- [Execution Layers](docs/architecture/execution-layers.md) - Canonical nomenclature
- [RESP Protocol](docs/api/resp-protocol.md) - Communication protocol
- [ADRs](docs/adr/) - Architecture decisions

### Component-Specific Guidelines

- **AGX**: See `agx/CLAUDE.md` for AGX-specific development
- **AGQ**: See `agq/CLAUDE.md` for AGQ security and testing requirements
- **AGW**: See `agw/CLAUDE.md` for AGW development
- **AUs**: See `docs/au-specs/agentic-unit-spec.md` for AU contract

### Communication

- **Issues**: GitHub issues in relevant repository
- **Discussions**: GitHub Discussions for questions, ideas
- **Pull Requests**: Code review and discussion

---

## Recognition

Contributors are recognized in:
- Git commit history (permanent record)
- Release notes (for significant contributions)
- `CONTRIBUTORS.md` (alphabetical list)

---

## Questions?

- Check [documentation](docs/)
- Search existing issues
- Open a new issue with `question` label
- Ask in GitHub Discussions

Thank you for contributing to AGEniX! 🚀
