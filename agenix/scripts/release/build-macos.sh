#!/bin/bash
# Build release binaries for macOS targets only
#
# Usage: ./build-macos.sh 0.1.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.0"
  exit 1
fi

WORKSPACE_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

# macOS targets only
TARGETS=(
  "aarch64-apple-darwin"
  "x86_64-apple-darwin"
)

echo "Building macOS release binaries for version $VERSION..."
echo "Workspace root: $WORKSPACE_ROOT"
echo ""

# Check if targets are available
echo "Checking available targets..."
for target in "${TARGETS[@]}"; do
  if rustup target list | grep -q "^$target (installed)$"; then
    echo "✓ $target installed"
  else
    echo "⚠️  $target not installed. Installing..."
    rustup target add "$target"
  fi
done
echo ""

for repo in agx agq agw; do
  REPO_PATH="$WORKSPACE_ROOT/$repo"

  if [ ! -d "$REPO_PATH" ]; then
    echo "⚠️  Skipping $repo (directory not found)"
    continue
  fi

  echo "Building $repo..."
  cd "$REPO_PATH"

  # Verify version matches
  CARGO_VERSION=$(grep '^version =' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
  if [ "$CARGO_VERSION" != "$VERSION" ]; then
    echo "❌ Error: $repo Cargo.toml version ($CARGO_VERSION) doesn't match target version ($VERSION)"
    exit 1
  fi

  for target in "${TARGETS[@]}"; do
    echo "  Building for $target..."

    # Build for target
    cargo build --release --target "$target" --quiet

    # Copy binary with version and target suffix
    SOURCE="target/$target/release/$repo"
    DEST="target/release/${repo}-${VERSION}-${target}"

    if [ -f "$SOURCE" ]; then
      cp "$SOURCE" "$DEST"
      echo "  ✓ Created $DEST"
    else
      echo "  ❌ Binary not found: $SOURCE"
      exit 1
    fi
  done

  echo "✓ $repo built for all macOS targets"
  echo ""
done

echo "All macOS binaries built successfully!"
echo ""
echo "Binaries created:"
for repo in agx agq agw; do
  REPO_PATH="$WORKSPACE_ROOT/$repo"
  if [ -d "$REPO_PATH/target/release" ]; then
    ls -lh "$REPO_PATH/target/release/${repo}-${VERSION}"-* 2>/dev/null || true
  fi
done
echo ""
echo "Note: Linux binaries should be built via GitHub Actions CI/CD"
echo ""
echo "Next steps:"
echo "  1. Test binaries: ./target/release/${repo}-${VERSION}-<target> --version"
echo "  2. Create GitHub releases: ../agenix/scripts/release/publish.sh $VERSION"
