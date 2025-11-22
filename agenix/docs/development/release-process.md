# AGEniX Release Process

**Version:** 1.0
**Status:** Active
**Applies to:** AGX, AGQ, AGW

---

## Overview

The AGEniX ecosystem uses **synchronized versioning** across core components (AGX, AGQ, AGW) to ensure compatibility and simplify deployment. This document defines the release process for creating, testing, and distributing new versions.

## Versioning Strategy

### Semantic Versioning

All components follow [Semantic Versioning 2.0.0](https://semver.org/):

```
MAJOR.MINOR.PATCH

Examples:
- 0.1.0  (Initial development)
- 0.2.0  (New features, backward compatible)
- 1.0.0  (Production-ready, stable API)
- 1.1.0  (New features)
- 1.1.1  (Bug fixes)
```

**Version semantics:**
- **MAJOR**: Breaking changes to APIs or protocols
- **MINOR**: New features, backward compatible
- **PATCH**: Bug fixes, security patches

### Synchronized Releases

**AGX, AGQ, and AGW maintain the same version number** to ensure ecosystem compatibility.

**Example:**
```
AGX v0.2.0 requires AGQ v0.2.0 and AGW v0.2.0
```

**Why synchronized?**
- RESP protocol changes require matching AGX ↔ AGQ versions
- Job schema changes require matching AGQ ↔ AGW versions
- Simplifies deployment and documentation

### Version Sources

All components use `CARGO_PKG_VERSION` from `Cargo.toml`:

```rust
// Automatically populated by Cargo
env!("CARGO_PKG_VERSION")  // "0.1.0"
```

**Required:**
- All `Cargo.toml` files must have matching `version` fields
- All binaries must print version with `--version` or `-v`

---

## Release Workflow

### Phase 1: Pre-Release

#### 1. Version Bump

Update `version` in all `Cargo.toml` files:

```bash
# Coordinate across repos
cd agx && sed -i '' 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
cd ../agq && sed -i '' 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
cd ../agw && sed -i '' 's/^version = ".*"/version = "0.2.0"/' Cargo.toml
```

**Use the release script (recommended):**
```bash
./agenix/scripts/release/bump-version.sh 0.2.0
```

#### 2. Update CHANGELOG

Add release notes to each repo's `CHANGELOG.md`:

```markdown
## [0.2.0] - 2025-11-20

### Added
- New REPL operational commands (AGX-058)
- JOBS.LIST, WORKERS.LIST, QUEUE.STATS (AGQ-043)

### Changed
- Improved error messages in plan validation

### Fixed
- LREM command compilation error (AGQ-023)

### Security
- Constant-time session key comparison
```

#### 3. Run Full Test Suite

```bash
# In each repo
cargo test --all-features
cargo clippy -- -D warnings
cargo audit
```

#### 4. Integration Testing

Test cross-component compatibility:

```bash
# Start AGQ
cd agq && ./target/release/agq --bind 127.0.0.1:6379 --session-key testkey &

# Test AGX communication
cd agx && AGQ_ADDR="127.0.0.1:6379" AGQ_SESSION_KEY="testkey" ./target/release/agx QUEUE stats

# Test AGW (when implemented)
cd agw && ./target/release/agw --agq-addr 127.0.0.1:6379
```

#### 5. Create Release Commits

```bash
# In each repo (agx, agq, agw)
git add Cargo.toml Cargo.lock CHANGELOG.md
git commit -m "chore: Release v0.2.0

- Bump version to 0.2.0
- Update CHANGELOG

See CHANGELOG.md for full release notes."
```

### Phase 2: Build Release Binaries

#### Build for Target Platforms

```bash
# macOS (aarch64-apple-darwin)
cargo build --release --target aarch64-apple-darwin

# macOS (x86_64-apple-darwin)
cargo build --release --target x86_64-apple-darwin

# Linux (x86_64-unknown-linux-gnu)
cargo build --release --target x86_64-unknown-linux-gnu

# Linux (aarch64-unknown-linux-gnu)
cargo build --release --target aarch64-unknown-linux-gnu
```

**Use the release script:**
```bash
./agenix/scripts/release/build-all.sh 0.2.0
```

This creates:
```
./target/release/
  agx-0.2.0-aarch64-apple-darwin
  agx-0.2.0-x86_64-apple-darwin
  agx-0.2.0-x86_64-unknown-linux-gnu
  agx-0.2.0-aarch64-unknown-linux-gnu
  (same for agq, agw)
```

#### Verify Binaries

```bash
# Test version output
./target/release/agx-0.2.0-aarch64-apple-darwin --version
# Output: agx 0.2.0

./target/release/agq-0.2.0-aarch64-apple-darwin --version
# Output: agq 0.2.0

./target/release/agw-0.2.0-aarch64-apple-darwin --version
# Output: agw 0.2.0
```

### Phase 3: Create GitHub Releases

#### 1. Tag Releases

```bash
# In each repo
git tag -a v0.2.0 -m "Release v0.2.0

See CHANGELOG.md for full release notes."

git push origin v0.2.0
```

#### 2. Create GitHub Releases

```bash
# In each repo
gh release create v0.2.0 \
  --title "AGX v0.2.0" \
  --notes-file CHANGELOG.md \
  --target main \
  ./target/release/agx-0.2.0-*

# Repeat for agq, agw
```

**Use the release script:**
```bash
./agenix/scripts/release/publish.sh 0.2.0
```

### Phase 4: Post-Release

#### 1. Verify Installations

Test the unified install script:

```bash
# Test install script (from GitHub)
curl -fsSL https://raw.githubusercontent.com/agenix-sh/agenix/v0.2.0/install.sh | bash

# Verify installations
agx --version  # Should show: agx 0.2.0
agq --version  # Should show: agq 0.2.0
agw --version  # Should show: agw 0.2.0
```

Or test individual component downloads:

```bash
# Download and test individual component
curl -L https://github.com/agenix-sh/agx/releases/download/v0.2.0/agx-0.2.0-$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]') -o agx
chmod +x agx
./agx --version
```

#### 2. Update Documentation

- Update `agenix/README.md` with new version
- Update installation instructions
- Announce on relevant channels

#### 3. Monitor Issues

Watch for:
- Installation problems
- Version mismatch errors
- Breaking changes discovered post-release

---

## Release Scripts

All release scripts live in `agenix/scripts/release/`:

### `bump-version.sh`

Updates version numbers across all repos:

```bash
#!/bin/bash
# Usage: ./bump-version.sh 0.2.0

set -e

NEW_VERSION=$1
if [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

echo "Bumping version to $NEW_VERSION..."

for repo in agx agq agw; do
  cd "../$repo"
  echo "Updating $repo..."

  # Update Cargo.toml
  sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml

  # Update Cargo.lock
  cargo update -p "$repo"

  echo "✓ $repo updated to $NEW_VERSION"
done

echo "All repos updated to $NEW_VERSION"
```

### `build-all.sh`

Builds release binaries for all targets:

```bash
#!/bin/bash
# Usage: ./build-all.sh 0.2.0

set -e

VERSION=$1
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
  "x86_64-unknown-linux-gnu"
  "aarch64-unknown-linux-gnu"
)

for repo in agx agq agw; do
  cd "../$repo"

  for target in "${TARGETS[@]}"; do
    echo "Building $repo for $target..."
    cargo build --release --target "$target"

    # Copy with version suffix
    cp "target/$target/release/$repo" \
       "target/release/${repo}-${VERSION}-${target}"
  done
done

echo "All binaries built successfully"
```

### `publish.sh`

Creates GitHub releases and uploads artifacts:

```bash
#!/bin/bash
# Usage: ./publish.sh 0.2.0

set -e

VERSION=$1

for repo in agx agq agw; do
  cd "../$repo"

  echo "Publishing $repo v$VERSION..."

  # Create and push tag
  git tag -a "v$VERSION" -m "Release v$VERSION"
  git push origin "v$VERSION"

  # Create GitHub release
  gh release create "v$VERSION" \
    --title "$repo v$VERSION" \
    --notes-file CHANGELOG.md \
    --target main \
    ./target/release/${repo}-${VERSION}-*

  echo "✓ $repo v$VERSION published"
done

echo "All releases published successfully"
```

---

## Version Compatibility Matrix

| AGX Version | AGQ Version | AGW Version | Status | Notes |
|-------------|-------------|-------------|--------|-------|
| 0.1.0 | 0.1.0 | 0.1.0 | Development | Initial implementation |
| 0.2.0 | 0.2.0 | 0.2.0 | Planned | REPL + operational visibility |
| 1.0.0 | 1.0.0 | 1.0.0 | Future | Production-ready |

**Compatibility rules:**
- **MAJOR versions must match** (e.g., 1.x.x ↔ 1.x.x only)
- **MINOR versions should match** for full feature support
- **PATCH versions can differ** (bug fixes don't break compatibility)

---

## Installation Methods

### Method 1: Install Script (Recommended)

The unified install script automatically detects your platform and installs all components:

```bash
# Install to ~/.local/bin (default)
curl -fsSL https://agenix.sh/install.sh | bash

# Or use raw GitHub URL
curl -fsSL https://raw.githubusercontent.com/agenix-sh/agenix/main/install.sh | bash

# Install to custom directory
curl -fsSL https://agenix.sh/install.sh | bash -s -- --dir /usr/local/bin

# Install specific version
curl -fsSL https://agenix.sh/install.sh | bash -s -- --version 0.2.0

# Verify installation
agx --version
agq --version
agw --version
```

**Features:**
- Automatic platform detection (macOS/Linux, x64/ARM)
- Installs to `~/.local/bin` by default
- Adds to PATH automatically
- Version-locked downloads (all components match)
- Verifies executable integrity

### Method 2: Manual Download

Download individual components from GitHub Releases:

```bash
# Detect platform
PLATFORM=$(uname -m)-$(uname -s | tr '[:upper:]' '[:lower:]')
VERSION=0.2.0

# Download binaries
curl -L "https://github.com/agenix-sh/agx/releases/download/v${VERSION}/agx-${VERSION}-${PLATFORM}" -o agx
curl -L "https://github.com/agenix-sh/agq/releases/download/v${VERSION}/agq-${VERSION}-${PLATFORM}" -o agq
curl -L "https://github.com/agenix-sh/agw/releases/download/v${VERSION}/agw-${VERSION}-${PLATFORM}" -o agw

# Make executable
chmod +x agx agq agw

# Install to PATH
sudo mv agx agq agw /usr/local/bin/

# Verify
agx --version
agq --version
agw --version
```

### Method 3: Cargo Install

```bash
cargo install --git https://github.com/agenix-sh/agx --tag v0.2.0
cargo install --git https://github.com/agenix-sh/agq --tag v0.2.0
cargo install --git https://github.com/agenix-sh/agw --tag v0.2.0
```

### Method 3: Build from Source

```bash
git clone https://github.com/agenix-sh/agx && cd agx
git checkout v0.2.0
cargo build --release
./target/release/agx --version
```

---

## Troubleshooting

### Version Mismatch Errors

**Symptom:** AGX reports "AGQ version incompatible"

**Solution:**
1. Check versions: `agx -v` and `agq -v`
2. Ensure MAJOR.MINOR match
3. Upgrade/downgrade as needed

### Binary Not Found

**Symptom:** `agx: command not found`

**Solution:**
1. Verify installation: `which agx`
2. Check PATH: `echo $PATH | grep /usr/local/bin`
3. Reinstall to correct location

### Version Shows Old Number

**Symptom:** `agx -v` shows old version after upgrade

**Solution:**
1. Check binary location: `which agx`
2. Remove old binary: `sudo rm $(which agx)`
3. Reinstall new version

---

## Security Considerations

### Release Signing

**Future:** Releases will be signed with GPG keys

```bash
# Verify signature (future)
curl -L https://github.com/agenix-sh/agx/releases/download/v0.2.0/agx-0.2.0.asc -o agx.asc
gpg --verify agx.asc agx
```

### Checksum Verification

**Future:** SHA256 checksums for all binaries

```bash
# Verify checksum (future)
curl -L https://github.com/agenix-sh/agx/releases/download/v0.2.0/SHA256SUMS -o SHA256SUMS
shasum -a 256 -c SHA256SUMS
```

---

## Rollback Procedure

If a release has critical issues:

### 1. Immediate Actions

```bash
# Mark release as pre-release
gh release edit v0.2.0 --prerelease

# Add warning to release notes
gh release edit v0.2.0 --notes "⚠️ Known issue: [describe]. Use v0.1.0 instead."
```

### 2. Hotfix Release

```bash
# Create hotfix branch
git checkout -b hotfix/0.2.1 v0.2.0

# Fix issue
# ... make changes ...

# Release patch
./scripts/release/bump-version.sh 0.2.1
./scripts/release/build-all.sh 0.2.1
./scripts/release/publish.sh 0.2.1
```

### 3. Yanking Broken Release

```bash
# Delete broken release (last resort)
gh release delete v0.2.0 --yes
git push origin --delete v0.2.0
```

---

## Checklist

Use this checklist for every release:

### Pre-Release
- [ ] All tests passing (`cargo test`)
- [ ] Lints clean (`cargo clippy`)
- [ ] Security audit clean (`cargo audit`)
- [ ] Integration tests passing
- [ ] CHANGELOG.md updated in all repos
- [ ] Version bumped in all `Cargo.toml` files
- [ ] Commits created with release message

### Build
- [ ] Release binaries built for all targets
- [ ] Version output verified for all binaries
- [ ] Binaries tested on target platforms

### Publish
- [ ] Tags created and pushed
- [ ] GitHub releases created
- [ ] Binaries uploaded to releases
- [ ] Release notes published

### Post-Release
- [ ] Documentation updated
- [ ] Installation verified
- [ ] Announcement posted
- [ ] Monitoring for issues

---

## References

- [Semantic Versioning](https://semver.org/)
- [Cargo Book: Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github)

---

**Last Updated:** 2025-11-19
**Next Review:** After first production release (v1.0.0)
