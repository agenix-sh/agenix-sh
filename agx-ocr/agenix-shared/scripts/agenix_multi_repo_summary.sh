#!/usr/bin/env bash
set -euo pipefail

REPOS=("agx" "agq" "agw" "agx-ocr")
ORG="agenix-sh"
OUTDIR="agenix_projects_summary"
mkdir -p "$OUTDIR"

central="$OUTDIR/CENTRAL_ROADMAP.md"

echo "# AGEnix Unified Roadmap" > "$central"
echo "" >> "$central"
echo "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")" >> "$central"
echo "" >> "$central"

for repo in "${REPOS[@]}"; do
    DIR="$OUTDIR/$repo"
    mkdir -p "$DIR"
    FULLREPO="$ORG/$repo"

    echo "=== Processing $FULLREPO ==="

    # Repo metadata
    gh api "/repos/$FULLREPO" > "$DIR/repo.json"

    # Issues
    gh issue list -R "$FULLREPO" --state all --json number,title,state,labels,createdAt,updatedAt > "$DIR/issues.json"

    # PRs
    gh pr list -R "$FULLREPO" --state all --json number,title,state,labels,createdAt,updatedAt > "$DIR/prs.json"

    # Tree
    gh api "/repos/$FULLREPO/git/trees/HEAD?recursive=1" > "$DIR/tree.json"

    # Local summary
    {
        echo "# Summary for $repo"
        echo ""
        echo "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
        echo ""
        echo "## Repo Metadata"
        echo "\`\`\`"
        gh api "/repos/$FULLREPO"
        echo "\`\`\`"
        echo ""
        echo "## Open Issues"
        gh issue list -R "$FULLREPO" --state open
    } > "$DIR/summary.md"

    # Append to central roadmap
    echo "## $repo Roadmap" >> "$central"
    echo "" >> "$central"
    echo "- Repo: https://github.com/$FULLREPO" >> "$central"
    echo "- Open Issues:" >> "$central"
    gh issue list -R "$FULLREPO" --state open >> "$central" 2>/dev/null || true
    echo "" >> "$central"
done

echo "✓ Aggregated summary generated in $OUTDIR/"
