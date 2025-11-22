#!/bin/bash
# Bump version across AGX, AGQ, AGW repos
#
# Usage: ./bump-version.sh 0.2.0

set -e

NEW_VERSION=$1

if [ -z "$NEW_VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

# Validate version format (semver)
if ! echo "$NEW_VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$'; then
  echo "Error: Version must be in format MAJOR.MINOR.PATCH (e.g., 0.2.0)"
  exit 1
fi

WORKSPACE_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

echo "Bumping version to $NEW_VERSION across all repos..."
echo "Workspace root: $WORKSPACE_ROOT"
echo ""

for repo in agx agq agw; do
  REPO_PATH="$WORKSPACE_ROOT/$repo"

  if [ ! -d "$REPO_PATH" ]; then
    echo "⚠️  Skipping $repo (directory not found: $REPO_PATH)"
    continue
  fi

  echo "Updating $repo..."
  cd "$REPO_PATH"

  # Check if repo is clean
  if ! git diff-index --quiet HEAD --; then
    echo "❌ Error: $repo has uncommitted changes. Commit or stash first."
    exit 1
  fi

  # Update Cargo.toml
  if [ "$(uname)" = "Darwin" ]; then
    # macOS (BSD sed)
    sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
  else
    # Linux (GNU sed)
    sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
  fi

  # Update Cargo.lock
  cargo update -p "$repo" --quiet

  echo "✓ $repo updated to $NEW_VERSION"
  echo ""
done

echo "All repos updated to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "  1. Update CHANGELOG.md in each repo"
echo "  2. Review changes: git diff"
echo "  3. Commit: git add Cargo.toml Cargo.lock && git commit -m 'chore: Release v$NEW_VERSION'"
