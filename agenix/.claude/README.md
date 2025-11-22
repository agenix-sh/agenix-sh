# AGEniX Claude Code Configuration

This directory contains shared Claude Code skills, agents, and commands for the AGEniX multi-repository ecosystem.

## Overview

The AGEniX project uses Claude Code skills and agents to maintain consistency across multiple repositories:
- **agenix** - Central documentation and architecture
- **agx** - Planner implementation
- **agq** - Queue manager implementation
- **agw** - Worker implementation
- **agx-ocr** - OCR Agentic Unit (reference implementation)

This `.claude/` directory serves as the **single source of truth** for shared development knowledge that applies across all component repositories.

## Directory Structure

```
.claude/
├── README.md           # This file
├── skills/             # Auto-activated capabilities
│   ├── agenix-architecture/
│   ├── agenix-security/
│   ├── agenix-testing/
│   └── rust-agenix-standards/
├── agents/             # Specialized subagents
│   ├── rust-engineer.md
│   ├── security-auditor.md
│   ├── github-manager.md
│   └── multi-repo-coordinator.md
├── commands/           # Slash commands (future)
└── hooks/              # Lifecycle hooks (future)
```

## Skills

**Skills** are auto-activated based on conversation context. Claude automatically loads and applies skills when their descriptions match the work being done.

### agenix-architecture
**Triggers**: Working with system design, component boundaries, execution layers

**Purpose**: Ensures consistent use of AGEniX architectural patterns and nomenclature (Task/Plan/Job/Action/Workflow)

**References**:
- `/Users/lewis/work/agenix-sh/agenix/docs/architecture/execution-layers.md`
- `/Users/lewis/work/agenix-sh/agenix/docs/architecture/system-overview.md`

**Usage**: Automatically activated when discussing architecture, component interactions, or execution model.

### agenix-security
**Triggers**: Security reviews, authentication code, input validation, RESP protocol

**Purpose**: Enforces security best practices including OWASP Top 10, zero-trust principles, and Rust security patterns

**References**:
- `/Users/lewis/work/agenix-sh/agenix/docs/development/security-guidelines.md`
- `/Users/lewis/work/agenix-sh/agenix/docs/zero-trust/zero-trust-execution.md`

**Usage**: Automatically activated for security-critical code, authentication, or vulnerability discussions.

### agenix-testing
**Triggers**: Writing tests, test reviews, coverage discussions, TDD

**Purpose**: Applies Test-Driven Development practices and ensures comprehensive test coverage (80% minimum, 100% for security-critical code)

**References**:
- `/Users/lewis/work/agenix-sh/agenix/docs/development/testing-strategy.md`
- `/Users/lewis/work/agenix-sh/agenix/docs/au-specs/testing-au.md`

**Usage**: Automatically activated when writing or reviewing tests, discussing coverage, or implementing TDD.

### rust-agenix-standards
**Triggers**: Writing Rust code, code reviews, error handling, async patterns

**Purpose**: Enforces consistent Rust coding standards and idioms across all AGEniX components

**References**:
- `/Users/lewis/work/agenix-sh/agenix/docs/development/security-guidelines.md`
- `/Users/lewis/work/agenix-sh/agenix/docs/development/testing-strategy.md`

**Usage**: Automatically activated when writing or reviewing Rust code.

## Agents

**Agents** are specialized subagents that can be explicitly invoked or auto-activated for specific tasks. They have custom system prompts and tool restrictions.

### rust-engineer
**Model**: Sonnet
**Tools**: Read, Write, Edit, Grep, Glob, Bash

**Purpose**: Deep Rust systems programming expertise for async patterns, performance optimization, and safe concurrent code

**Use Cases**:
- Implementing complex Rust features
- Reviewing Rust code for safety and performance
- Designing efficient data structures
- Optimizing hot paths

**Activation**: Explicitly request ("Use rust-engineer subagent") or auto-activated for complex Rust development tasks.

### security-auditor
**Model**: Sonnet
**Tools**: Read, Grep, Glob, Bash (read-only focus)

**Purpose**: Security vulnerability detection and prevention focusing on OWASP Top 10, command injection, and zero-trust compliance

**Use Cases**:
- Security code reviews
- Vulnerability audits
- Penetration testing planning
- Security test verification

**Activation**: Explicitly request for security reviews or auto-activated for security-sensitive changes.

### github-manager
**Model**: Haiku (fast and cost-effective)
**Tools**: Read, Bash, Grep, Glob

**Purpose**: GitHub workflow management including issue creation, PR generation, labeling, and cross-repo coordination

**Use Cases**:
- Creating well-structured GitHub issues
- Generating comprehensive PR descriptions
- Managing labels and milestones
- Tracking multi-repo features

**Activation**: Explicitly request when creating issues/PRs or managing GitHub workflows.

### multi-repo-coordinator
**Model**: Sonnet
**Tools**: Read, Grep, Glob, Bash

**Purpose**: Coordinate changes across AGEniX repositories ensuring architectural consistency and component boundary respect

**Use Cases**:
- Planning multi-repo features
- Validating cross-component changes
- Ensuring execution layer nomenclature compliance
- Coordinating releases

**Activation**: Automatically activated for multi-repo discussions or explicitly requested for coordination tasks.

## Usage Across Repositories

### For Local Development

**Option 1: Git Submodule (Recommended for Teams)**

Add this directory as a submodule in component repos:

```bash
# In component repo (agq, agw, agx, agx-ocr)
cd /Users/lewis/work/agenix-sh/agq
git submodule add ../../agenix.git agenix-shared
ln -s agenix-shared/.claude .claude
git commit -m "Add shared Claude config via submodule"

# GitHub Actions automatically works:
- uses: actions/checkout@v3
  with:
    submodules: recursive
```

**Option 2: Git Worktree (Power Users)**

For active development with live bidirectional sync:

```bash
# In component repo
cd /Users/lewis/work/agenix-sh/agq
git worktree add .claude /Users/lewis/work/agenix-sh/agenix/.claude

# Changes made in agq/.claude/ automatically update agenix/.claude/
```

**Option 3: Personal Skills (Individual)**

Copy skills to personal directory for use across all projects:

```bash
cp -r /Users/lewis/work/agenix-sh/agenix/.claude/skills/* ~/.claude/skills/
```

### For GitHub Actions

**Method 1: Submodule (Automatic)**

```yaml
- uses: actions/checkout@v3
  with:
    submodules: recursive  # Automatically loads .claude/
```

**Method 2: Clone and Copy**

```yaml
- name: Setup Claude Config
  run: |
    git clone --depth 1 https://github.com/agenix-sh/agenix.git /tmp/agenix
    cp -r /tmp/agenix/.claude .claude
```

## Specialization Per Component

Component repos can extend shared configuration with local-specific skills/agents:

```
agq/
├── .claude/                    # From agenix submodule
└── .claude-local/              # AGQ-specific (optional)
    ├── skills/
    │   └── agq-resp-protocol/  # RESP-specific patterns
    └── agents/
        └── agq-reviewer.md     # AGQ-focused reviewer
```

Claude loads both:
1. Project `.claude/` (shared from agenix)
2. Project `.claude-local/` (component-specific)

Priority: Local > Shared

## Updating Shared Configuration

### Making Changes

1. **Edit in agenix repo** (single source of truth):
   ```bash
   cd /Users/lewis/work/agenix-sh/agenix/.claude/skills/agenix-security/
   # Edit SKILL.md
   git commit -m "Update security skill with new pattern"
   ```

2. **Component repos get updates**:
   - **Submodule users**: `git submodule update --remote`
   - **Worktree users**: Changes appear immediately
   - **GitHub Actions**: Automatic on next run

### Adding New Skills/Agents

1. Create in `agenix/.claude/skills/` or `agenix/.claude/agents/`
2. Follow naming convention: lowercase-hyphenated
3. Include YAML frontmatter with clear description
4. Document in this README
5. Commit to agenix repo
6. Component repos inherit automatically

## Progressive Disclosure

Skills use **progressive disclosure** to balance completeness with performance:

1. **SKILL.md**: Core instructions (always loaded)
2. **reference.md**: Detailed docs (loaded on-demand)
3. **examples/**: Code samples (loaded when referenced)
4. **Central docs**: Full details (Read tool on-demand)

This keeps skills concise while maintaining access to comprehensive documentation.

## Best Practices

### Skill Descriptions
- Include **functionality** and **activation triggers**
- Use specific terms users would mention
- Example: "Apply RESP protocol security patterns" not "Handle security"

### Agent Specialization
- Use **Sonnet** for complex reasoning (rust-engineer, security-auditor)
- Use **Haiku** for fast, cost-effective tasks (github-manager)
- Restrict tools appropriately (security-auditor is read-only)

### Documentation References
- Always reference central docs with absolute paths
- Use Read tool for on-demand loading
- Don't duplicate content from central docs

### Synchronization
- **For teams**: Use git submodules
- **For individuals**: Use worktrees or personal skills
- **For CI**: Submodules work automatically

## Troubleshooting

### Skills Not Loading
- Check skill description matches your query
- Verify `.claude/skills/` directory exists
- Ensure SKILL.md has valid YAML frontmatter

### Submodule Issues
```bash
# Initialize submodules
git submodule update --init --recursive

# Update to latest
git submodule update --remote

# Check submodule status
git submodule status
```

### Worktree Issues
```bash
# List worktrees
git worktree list

# Remove worktree
git worktree remove .claude

# Re-add worktree
git worktree add .claude /path/to/agenix/.claude
```

## Contributing

To improve shared skills/agents:

1. Make changes in `agenix/.claude/`
2. Test across multiple component repos
3. Update this README if adding new skills/agents
4. Commit with descriptive message
5. Component repos inherit via submodule update

## Related Documentation

- **Architecture**: `/Users/lewis/work/agenix-sh/agenix/docs/architecture/`
- **Development Guidelines**: `/Users/lewis/work/agenix-sh/agenix/docs/development/`
- **ADRs**: `/Users/lewis/work/agenix-sh/agenix/docs/adr/`
- **AU Specs**: `/Users/lewis/work/agenix-sh/agenix/docs/au-specs/`
- **Root CLAUDE.md**: `/Users/lewis/work/agenix-sh/CLAUDE.md` (instance guidelines)

## Version History

- **2025-11-17**: Initial `.claude/` structure with 4 skills and 4 agents
  - Skills: architecture, security, testing, rust-standards
  - Agents: rust-engineer, security-auditor, github-manager, multi-repo-coordinator

---

**Maintained by**: AGX Core Team
**Questions**: Open issue in agenix/agenix repository
