---
id: AILOG-2026-05-28-001
title: Migrate documentation framework from DevTrail to StrayMark
status: accepted
created: 2026-05-28
agent: claude-opus-4-7-v1.0
confidence: high
review_required: true
risk_level: medium
tags: [framework, rebranding, governance, monorepo, tooling]
related: []
---

# AILOG: Migrate documentation framework from DevTrail to StrayMark

## Summary

Migrated the monorepo's documentation framework from **DevTrail v1.0.0**
to **StrayMark v4.19.0** (framework) + `straymark` CLI 3.16.0. The framework
was renamed upstream (`strangedaystech/devtrail-framework` →
`StrangeDaysTech/straymark`); no automated migration tool was provided
because the adopter set was small. This AILOG records the manual migration
performed in branch `chore/migrate-devtrail-to-straymark`.

## Context

The lnxdrive monorepo adopted DevTrail before the rebranding. The
StrayMark CLI is already installed system-wide and supersedes the old
shell scripts. The renamed framework introduces a substantially expanded
governance structure (more templates, ISO 42001 / EU AI Act / NIST AI RMF
alignment, optional China regulatory scope, Charter pattern, security/AI
model dirs, and project-local skills under `.claude/`, `.gemini/`,
`.codex/`, `.agent/`).

## Decision boundaries (agreed with user before execution)

1. **Historical AILOGs and AIDEC**: left intact. The 57 AILOGs under
   `07-ai-audit/agent-logs/` and the 1 AIDEC under
   `07-ai-audit/decisions/` continue to mention "DevTrail" in their
   bodies — they are immutable audit records of the state of the world
   when written.
2. **Subproject shell scripts**: removed. The `straymark` CLI replaces
   `scripts/devtrail-new.sh`, `scripts/devtrail-status.sh`,
   `scripts/pre-commit-docs.sh` and `scripts/validate-docs.ps1`
   (28 files across 7 subprojects).
3. **Global user-level skills**: not present on this machine
   (`~/.claude/skills/` does not exist; `~/.gemini/skills/` does not
   exist). The inventory phase initially mislocated them; the actual
   `devtrail-*` skills lived in `.claude/skills/` and `.gemini/skills/`
   inside the repo and have been replaced by the `straymark-*`
   counterparts installed by `straymark repair`.
4. **Root `CLAUDE.md` / `GEMINI.md`**: manually merged. The new
   `STRAYMARK.md` file is the canonical source of governance rules;
   `CLAUDE.md` and `GEMINI.md` keep only monorepo-specific sections
   (project structure, agent-logs organization by component, design
   guide navigation, Context7) and a pointer to `STRAYMARK.md` via the
   `<!-- straymark:begin -->...<!-- straymark:end -->` block.

## Migration steps executed

### Fase 0 — Preparation
- Created branch `chore/migrate-devtrail-to-straymark` from `main`.
- Tagged pre-migration state as `pre-straymark-migration` (local only).
- Saved `CLAUDE.md` snapshot to `/tmp/lnxdrive-CLAUDE-pre-migration.md`
  for manual merging in Fase 2.

### Fase 1 — Rename and framework restore
- `git mv .devtrail .straymark` (preserves blame on 92 files).
- `straymark repair` restored 183 framework files: `STRAYMARK.md` at the
  repo root, expanded `00-governance/` (AI-GOVERNANCE-POLICY,
  CHINA-REGULATORY-FRAMEWORK, NIST guides, ISO-25010-2023, etc.), new
  directories `08-security/`, `09-ai-models/`, `audit-prompts/`,
  `hooks/`, `schemas/`, `scripts/`, project-local skills under
  `.claude/skills/`, `.gemini/skills/`, `.codex/skills/`,
  `.agent/workflows/`, and CI workflow `.github/workflows/docs-validation.yml`
  at the repo root.
- `straymark status` confirmed all 17 framework items present;
  AILOG=57, AIDEC=1.

### Fase 2 — Reconciliation
- Rewrote `CLAUDE.md` and `GEMINI.md` to follow the StrayMark
  `<!-- straymark:begin -->/<!-- straymark:end -->` convention while
  preserving the monorepo-specific sections. Eliminated DRY
  duplication of governance rules now owned by `STRAYMARK.md`.
- Updated `.straymark/config.yml` header and upstream URL.
- Removed `.straymark/QUICK-REFERENCE.md` (legacy DevTrail v1 file,
  superseded by `.straymark/00-governance/QUICK-REFERENCE.md`).
- Replaced 8 legacy v1 templates (`TEMPLATE-{ADR,AIDEC,AILOG,ETH,INC,REQ,TDE,TES}.md`)
  with v4 framework versions.
- Replaced 4 legacy v1 governance docs (`AGENT-RULES`, `DOCUMENTATION-POLICY`,
  `GIT-BRANCHING-STRATEGY`, `PRINCIPLES`) with v4 framework versions.

### Fase 3 — Subprojects
- `git rm` of 28 obsolete shell/PS scripts (4 scripts × 7 subprojects:
  cosmic, engine, gnome, gtk3, guide, packaging, plasma).
- Bulk `sed` rebranding in 14 files (7 × `.github/copilot-instructions.md` +
  7 × `.github/workflows/docs-validation.yml`).

### Fase 4 — Repo-wide sweep
- Removed 10 legacy project-local `devtrail-*` skill directories
  (5 in `.claude/skills/`, 5 in `.gemini/skills/`).
- Bulk `sed` rebranding in 21 remaining files (root `CONTRIBUTING.md`,
  `README.md`, `ayuda.md`, `lnxdrive.spdx`, 7 subproject `.cursorrules`,
  `lnxdrive-engine/.specify/memory/constitution.md`, multiple
  `lnxdrive-guide/04-Componentes/*.md` and `lnxdrive-engine/specs/*.md`).
- Cleaned 5 footers in `.straymark/02-design/risk-analysis/*.md`.

### Fase 5 — Global user skills
- Not applicable: no `~/.claude/skills/` or `~/.gemini/skills/`
  directories exist on this machine.

### Fase 6 — Verification
- `straymark status` → all 17 items present, AILOG=57, AIDEC=1.
- `straymark validate` → **0 errors, 3 warnings**. The 3 warnings are
  `[SEC-001]` false positives on the strings `Bearer` and `token:`
  appearing in narrative documentation context inside historical AILOGs
  (`AILOG-2026-02-03-006`, `-007-implement-download-upload-operations`,
  `-007-implement-graph-delta-api`). No remediation required.
- `grep -rli devtrail . --exclude-dir=.straymark --exclude-dir=.git` → 0
  matches.
- Remaining `devtrail` references inside `.straymark/` (54 files: 53
  AILOGs + 1 AIDEC) are intentional historical audit records.

## Risks and mitigations

| Risk | Mitigation |
|------|------------|
| Lost `git blame` after rename | `git mv` used for the directory rename; subsequent edits show as `RM` (rename + modified), preserving history through `git log --follow`. |
| Loss of monorepo-specific guidance in `CLAUDE.md` | Pre-migration snapshot kept at `/tmp/lnxdrive-CLAUDE-pre-migration.md`; merged manually preserving Project Structure, Agent-Logs Organization, Design Guide Navigation, Context7 sections. |
| Stale frontmatter in 57 historical AILOGs vs StrayMark v4 schema | `straymark validate` reports 0 errors against existing schemas — v1 frontmatter remains forward-compatible. No bulk frontmatter rewrite was performed. |
| CI workflows in subprojects (`lnxdrive-*/.github/workflows/docs-validation.yml`) reference `.straymark/**` paths but live in subdirectories of the monorepo (only the **root** `.github/workflows/` is executed by GitHub Actions). | Out of scope for this PR. Subproject workflows are inert in the monorepo; consolidating to the root workflow can be addressed in a follow-up TDE if needed. |

## Verification artifacts

- Branch: `chore/migrate-devtrail-to-straymark`
- Pre-migration tag (local): `pre-straymark-migration`
- `straymark status`: 17/17 items present
- `straymark validate`: 0 errors, 3 informational warnings
- Repo-wide `devtrail` reference count outside `.straymark/`: **0**
