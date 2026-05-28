# StrayMark - Guidelines for GitHub Copilot

> This file configures GitHub Copilot CLI under StrayMark.

## Language Configuration

Check `.straymark/config.yml` for the project's language setting:

```yaml
language: en  # Options: en, es (default: en)
```

**Template paths based on language:**

| Language | Template Path |
|----------|---------------|
| `en` (default) | `.straymark/templates/TEMPLATE-*.md` |
| `es` | `.straymark/templates/i18n/es/TEMPLATE-*.md` |

If the config file doesn't exist or `language` is not set, use English (`en`) as default.

## Documentation Reporting

At the end of each task, you MUST report your StrayMark documentation status:

**If you created documentation:**
```
StrayMark: Created AILOG-2025-01-27-001-implement-auth.md
```

**If documentation was not needed:**
```
StrayMark: No documentation required (minor change / <10 lines)
```

**If you should have documented but didn't:**
```
StrayMark: Documentation pending - review required
```

This transparency helps users verify compliance with StrayMark rules.

## Fundamental Principle

> **"No significant change without a documented trace."**

## Your Identity as an Agent

When working on this project:

- **Identify yourself** as: `copilot-cli-v1.0`
- **Declare** your confidence level in decisions: `high | medium | low`
- **Record** your identification in the `agent:` field of the metadata

## When to Document

### MANDATORY (create document)

| Situation | Action |
|-----------|--------|
| >10 lines of code in business logic | Create AILOG |
| Decision between technical alternatives | Create AIDEC |
| Changes in security/authentication | Create AILOG + mark `risk_level: high` |
| Personal data (GDPR/PII) | Create AILOG + request ETH |
| Integration with external service | Create AILOG |
| Change in public API or DB schema | Create AILOG |

### DO NOT DOCUMENT

- Trivial changes (whitespace, typos, formatting)
- Sensitive information (credentials, tokens, API keys)

## File Naming Convention

```
[TYPE]-[YYYY-MM-DD]-[NNN]-[description].md
```

**Example**: `AILOG-2025-01-27-001-implement-oauth.md`

## Required Metadata

```yaml
---
id: AILOG-2025-01-27-001
title: Brief description
status: accepted
created: 2025-01-27
agent: copilot-cli-v1.0
confidence: high | medium | low
review_required: true | false
risk_level: low | medium | high | critical
---
```

## Autonomy Limits

| Type | Autonomy |
|------|----------|
| AILOG | Create freely |
| AIDEC | Create freely |
| ETH | Draft only → human approves |
| ADR | Create → requires review |
| REQ | Propose → human validates |
| TDE | Identify yes, prioritize no |

## Documentation Map (StrayMark)

```
.straymark/
├── 00-governance/          ← Policies (load if in doubt)
│   ├── AGENT-RULES.md      # Detailed rules
│   └── DOCUMENTATION-POLICY.md
├── 01-requirements/        ← REQ-*.md
├── 02-design/decisions/    ← ADR-*.md
├── 04-testing/             ← TES-*.md
├── 05-operations/incidents/← INC-*.md
├── 06-evolution/technical-debt/ ← TDE-*.md
├── 07-ai-audit/
│   ├── agent-logs/         ← AILOG-*.md (create here)
│   ├── decisions/          ← AIDEC-*.md (create here)
│   └── ethical-reviews/    ← ETH-*.md
└── templates/              ← Templates (load when creating)
```

## When to Load Templates

| Need to | Load |
|---------|------|
| Create AILOG | `.straymark/templates/TEMPLATE-AILOG.md` |
| Create AIDEC | `.straymark/templates/TEMPLATE-AIDEC.md` |
| Create ADR | `.straymark/templates/TEMPLATE-ADR.md` |
| Naming questions | `.straymark/00-governance/DOCUMENTATION-POLICY.md` |
| Autonomy questions | `.straymark/00-governance/AGENT-RULES.md` |

## Quick Type Reference

| Prefix | Name | Location |
|--------|------|----------|
| `AILOG` | AI Action Log | `.straymark/07-ai-audit/agent-logs/` |
| `AIDEC` | AI Decision | `.straymark/07-ai-audit/decisions/` |
| `ETH` | Ethical Review | `.straymark/07-ai-audit/ethical-reviews/` |
| `ADR` | Architecture Decision Record | `.straymark/02-design/decisions/` |
| `REQ` | Requirement | `.straymark/01-requirements/` |
| `TES` | Test Plan | `.straymark/04-testing/` |
| `INC` | Incident Post-mortem | `.straymark/05-operations/incidents/` |
| `TDE` | Technical Debt | `.straymark/06-evolution/technical-debt/` |

## Human Review Required

Mark `review_required: true` when:
- `confidence: low`
- `risk_level: high | critical`
- Security decisions
- Irreversible changes

---

*StrayMark v1.0.0 | [Strange Days Tech](https://strangedays.tech)*
