# Agent Workspace

This folder is the shared, versioned home for repo-specific agent workflows.

## Layout

- `.agents/skills/` - reusable skills with `SKILL.md` entrypoints.
- `.agents/rules/` - machine-readable workflow rules.
- `.agents/hooks/` - reusable check scripts called by git hooks.
- `.agents/local/` - local scratch docs and notes, ignored by git.

## Compatibility

Codex reads the root `AGENTS.md` file for project instructions. Put durable
Codex-facing guidance there.

Claude-specific files in `.claude/` are legacy local configuration. When a
Claude setting is useful across agents, translate it into `AGENTS.md` or a
rule under `.agents/rules/`.
