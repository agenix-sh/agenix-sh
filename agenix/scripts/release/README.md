# AGEniX Release Scripts

Automation scripts for creating synchronized releases across AGX, AGQ, and AGW.

## Scripts

### `bump-version.sh`

Updates version numbers in all component repos.

```bash
./bump-version.sh 0.2.0
```

**What it does:**
- Updates `version` in `Cargo.toml` for agx, agq, agw
- Updates `Cargo.lock` files
- Validates semver format
- Checks for uncommitted changes

**Requirements:**
- Clean git working directory in all repos
- Valid semver format (MAJOR.MINOR.PATCH)

### `build-all.sh`

Builds release binaries for all supported platforms.

```bash
./build-all.sh 0.2.0
```

**What it does:**
- Builds for aarch64-apple-darwin (macOS ARM)
- Builds for x86_64-apple-darwin (macOS Intel)
- Builds for x86_64-unknown-linux-gnu (Linux x64)
- Builds for aarch64-unknown-linux-gnu (Linux ARM)
- Creates versioned binaries: `agx-0.2.0-aarch64-apple-darwin`

**Requirements:**
- Rust toolchains installed: `rustup target add <target>`
- Version in `Cargo.toml` matches argument

### `publish.sh`

Creates GitHub releases and uploads binaries.

```bash
./publish.sh 0.2.0
```

**What it does:**
- Creates git tags: `v0.2.0`
- Pushes tags to GitHub
- Creates GitHub releases with CHANGELOG notes
- Uploads all platform binaries

**Requirements:**
- GitHub CLI (`gh`) authenticated
- Binaries built (run `build-all.sh` first)
- CHANGELOG.md updated with release notes

## Full Release Workflow

```bash
# 1. Bump versions
./bump-version.sh 0.2.0

# 2. Update CHANGELOGs manually
cd ../../agx && vi CHANGELOG.md
cd ../agq && vi CHANGELOG.md
cd ../agw && vi CHANGELOG.md

# 3. Commit version bumps
cd ../agx && git add Cargo.toml Cargo.lock CHANGELOG.md && git commit -m "chore: Release v0.2.0"
cd ../agq && git add Cargo.toml Cargo.lock CHANGELOG.md && git commit -m "chore: Release v0.2.0"
cd ../agw && git add Cargo.toml Cargo.lock CHANGELOG.md && git commit -m "chore: Release v0.2.0"

# 4. Push commits
cd ../agx && git push
cd ../agq && git push
cd ../agw && git push

# 5. Build binaries
cd ../agenix/scripts/release
./build-all.sh 0.2.0

# 6. Test binaries
../agx/target/release/agx-0.2.0-aarch64-apple-darwin --version
../agq/target/release/agq-0.2.0-aarch64-apple-darwin --version
../agw/target/release/agw-0.2.0-aarch64-apple-darwin --version

# 7. Publish releases
./publish.sh 0.2.0

# 8. Verify
gh release list
```

## Platform Support

| Platform | Target | Status |
|----------|--------|--------|
| macOS (ARM) | aarch64-apple-darwin | ✅ Supported |
| macOS (Intel) | x86_64-apple-darwin | ✅ Supported |
| Linux (x64) | x86_64-unknown-linux-gnu | ✅ Supported |
| Linux (ARM) | aarch64-unknown-linux-gnu | ✅ Supported |
| Windows | x86_64-pc-windows-msvc | 🚧 Future |

## Troubleshooting

### "Target not installed"

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

### "GitHub CLI not authenticated"

```bash
gh auth login
```

### "Version mismatch"

Ensure you ran `bump-version.sh` before `build-all.sh`:

```bash
grep '^version' ../../agx/Cargo.toml
grep '^version' ../../agq/Cargo.toml
grep '^version' ../../agw/Cargo.toml
```

## See Also

- [Release Process Documentation](../../docs/development/release-process.md)
- [GitHub Releases](https://github.com/agenix-sh/agx/releases)
