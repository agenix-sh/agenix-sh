# Claude Code Instance Guidelines for Agenix Multi-Repo Workspace

## Purpose
This Claude Code instance is dedicated to **planning and integration testing** across the Agenix ecosystem.

## Repository Structure

This workspace `/Users/lewis/work/agenix-sh` contains multiple independent git repositories:

### Core Repositories
- **agenix** - Central meta repository (architecture, specs, docs, website)
- **agx** - CLI planner/orchestrator
- **agq** - Queue and scheduler for distributed execution
- **agw** - Worker process for executing plans

### Agentic Units (AU)
- **agx-ocr** - OCR Agentic Unit (first AU implementation)
- Additional AU repos to be added

### Dependencies
- **deepseek-ocr.rs** - Dependency for agx-ocr (ignore for documentation)

## Important Notes

### Git Workflow
- This **root directory is NOT a git repository**
- Each subdirectory is its own independent git repository
- When committing changes, commit to the specific repo (e.g., `cd agx && git commit`)
- Do not attempt git operations in `/Users/lewis/work/agenix-sh`

### Documentation Strategy
- **Canonical documentation** lives in `agenix/docs/`
- **Implementation-specific** documentation lives in component repos
- Cross-cutting concerns (architecture, schemas, contracts) centralized in agenix
- Component-specific details (Rust setup, platform guides) stay in their repos

### Multi-Instance Collaboration
This workspace supports multiple Claude Code and Codex instances:
- **Planning instance** (this one): Integration testing, architecture, cross-repo coordination
- **Development instances**: Working in specific repos on GitHub issues
- **Review instances**: PR reviews before merging

## Responsibilities of This Instance

1. **Planning**: Cross-repo architectural planning and coordination
2. **Integration Testing**: Testing interactions between agx, agq, agw
3. **Documentation Centralization**: Moving cross-cutting docs to agenix
4. **Architecture Decisions**: Recording ADRs in agenix/docs/adr/
5. **Coordination**: Ensuring consistency across repos

## What This Instance Should NOT Do

- Do not work on feature development in individual repos (use dedicated instances)
- Do not merge PRs (review only, flag for human approval)
- Do not make breaking changes without coordination across repos

## Reference Individual Repo Guidelines

Each component repo has its own CLAUDE.md with specific guidelines:
- `agx/CLAUDE.md` - AGX planner development
- `agq/CLAUDE.md` - AGQ queue development (most comprehensive security guidelines)
- `agw/CLAUDE.md` - AGW worker development
- `agx-ocr/CLAUDE.md` - OCR AU development

## Current Mission

**Documentation Centralization Project**
- Phase 1: Move canonical docs to agenix (execution-layers, job-schema)
- Phase 2: Create API documentation and ADRs
- Phase 3: AU development resources
- Phase 4: Individual repo cleanup
- Phase 5: Website foundation (Docusaurus)

## Key Architectural Principles

1. **Zero-Trust Execution** - Security-first design
2. **Dual-Model Planning** - Echo (fast) + Delta (thorough)
3. **RESP Protocol** - AGQ communication standard
4. **Agentic Units** - Composable, contractual tool integration
5. **Five Execution Layers** - Task → Plan → Job → Action → Workflow

## Quick Links

- Central docs: `agenix/docs/`
- Schemas: `agenix/specs/`
- Roadmap: `agenix/docs/roadmap/`
- Architecture: `agenix/docs/architecture/`
- Shared Skills/Agents: `agenix/.claude/`

## Claude Code Skills & Agents

This workspace uses Claude Code skills and agents to maintain consistency across repositories.

### Shared Configuration Location
**`agenix/.claude/`** - Single source of truth for shared development knowledge

### Available Skills (Auto-Activated)
- **agenix-architecture** - Architectural patterns and nomenclature (Task/Plan/Job/Action/Workflow)
- **agenix-security** - Security best practices, OWASP Top 10, zero-trust principles
- **agenix-testing** - TDD practices, coverage requirements, test organization
- **rust-agenix-standards** - Rust coding standards and idioms

Skills automatically activate when their descriptions match the work being done.

### Available Agents (Explicit or Auto-Activated)
- **rust-engineer** - Rust systems programming expert (async, performance, safety)
- **security-auditor** - Security vulnerability detection and prevention
- **github-manager** - Issue/PR creation, labeling, cross-repo coordination
- **multi-repo-coordinator** - Cross-repository change coordination

### Usage in Component Repos

Component repos (agx, agq, agw, agx-ocr) should add shared configuration via git submodule:

```bash
# In component repo
git submodule add ../../agenix.git agenix-shared
ln -s agenix-shared/.claude .claude
```

This ensures all Claude instances working in component repos have access to shared skills and agents.

For detailed documentation, see: `agenix/.claude/README.md`
