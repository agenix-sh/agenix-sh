#!/bin/bash
# Create GitHub releases and upload binaries
#
# Usage: ./publish.sh 0.2.0

set -e

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
  exit 1
fi

WORKSPACE_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

echo "Publishing version $VERSION to GitHub..."
echo "Workspace root: $WORKSPACE_ROOT"
echo ""

for repo in agx agq agw; do
  REPO_PATH="$WORKSPACE_ROOT/$repo"

  if [ ! -d "$REPO_PATH" ]; then
    echo "⚠️  Skipping $repo (directory not found)"
    continue
  fi

  echo "Publishing $repo v$VERSION..."
  cd "$REPO_PATH"

  # Check if binaries exist
  BINARIES=("$REPO_PATH/target/release/${repo}-${VERSION}"-*)
  if [ ! -f "${BINARIES[0]}" ]; then
    echo "❌ Error: No binaries found for $repo v$VERSION"
    echo "   Run ./build-all.sh $VERSION first"
    exit 1
  fi

  # Check if CHANGELOG.md exists and mentions this version
  if [ ! -f "CHANGELOG.md" ]; then
    echo "⚠️  Warning: CHANGELOG.md not found for $repo"
  elif ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo "⚠️  Warning: $VERSION not found in CHANGELOG.md"
  fi

  # Create and push tag
  TAG="v$VERSION"
  if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "⚠️  Tag $TAG already exists, skipping tag creation"
  else
    echo "Creating tag $TAG..."
    git tag -a "$TAG" -m "Release v$VERSION

See CHANGELOG.md for full release notes."
    git push origin "$TAG"
    echo "✓ Tag $TAG created and pushed"
  fi

  # Create GitHub release
  echo "Creating GitHub release..."
  if gh release view "$TAG" >/dev/null 2>&1; then
    echo "⚠️  Release $TAG already exists, skipping release creation"
  else
    # Extract release notes from CHANGELOG if available
    if [ -f "CHANGELOG.md" ]; then
      RELEASE_NOTES=$(awk "/^## \[$VERSION\]/,/^## \[/" CHANGELOG.md | sed '1d;$d')
    else
      RELEASE_NOTES="Release v$VERSION"
    fi

    gh release create "$TAG" \
      --title "$(echo $repo | tr '[:lower:]' '[:upper:]') v$VERSION" \
      --notes "$RELEASE_NOTES" \
      --target main \
      "$REPO_PATH/target/release/${repo}-${VERSION}"-*

    echo "✓ GitHub release created for $repo v$VERSION"
  fi

  echo ""
done

# Publish install.sh to agenix repo release
echo "Publishing install.sh to agenix repo..."
AGENIX_PATH="$WORKSPACE_ROOT/agenix"
cd "$AGENIX_PATH"

TAG="v$VERSION"
if git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "⚠️  Tag $TAG already exists in agenix repo"
else
  echo "Creating tag $TAG in agenix repo..."
  git tag -a "$TAG" -m "Release v$VERSION - Installation script

This release provides the unified install.sh script that downloads
and installs AGX, AGQ, and AGW binaries for the matched version.

Usage: curl -fsSL https://agenix.sh/install.sh | bash"
  git push origin "$TAG"
  echo "✓ Tag $TAG created in agenix repo"
fi

if gh release view "$TAG" >/dev/null 2>&1; then
  echo "⚠️  Agenix release $TAG already exists, uploading install.sh..."
  gh release upload "$TAG" "$AGENIX_PATH/install.sh" --clobber
else
  echo "Creating agenix release with install.sh..."
  gh release create "$TAG" \
    --title "AGEniX v$VERSION" \
    --notes "# AGEniX v$VERSION Installation

One-line installer for AGX, AGQ, and AGW:

\`\`\`bash
curl -fsSL https://agenix.sh/install.sh | bash
\`\`\`

Or install to custom directory:

\`\`\`bash
curl -fsSL https://agenix.sh/install.sh | bash -s -- --dir /usr/local/bin
\`\`\`

See individual component releases for detailed changelogs:
- [AGX v$VERSION](https://github.com/agenix-sh/agx/releases/tag/v$VERSION)
- [AGQ v$VERSION](https://github.com/agenix-sh/agq/releases/tag/v$VERSION)
- [AGW v$VERSION](https://github.com/agenix-sh/agw/releases/tag/v$VERSION)" \
    --target main \
    "$AGENIX_PATH/install.sh"
fi

echo "✓ install.sh published to agenix release"
echo ""

echo "All releases published successfully!"
echo ""
echo "Installation:"
echo "  curl -fsSL https://raw.githubusercontent.com/agenix-sh/agenix/v$VERSION/install.sh | bash"
echo ""
echo "Next steps:"
echo "  1. Verify releases: gh release list"
echo "  2. Test installation script"
echo "  3. Update documentation with new version"
echo "  4. Announce release"
