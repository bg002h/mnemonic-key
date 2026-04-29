# Agent reports archive

Verbatim final reports from subagent dispatches that produced commits, parallel-batch buckets, or other multi-step work in the `mnemonic-key` repo. Persisted to disk so the audit trail survives beyond the controller's conversation context. Mirrors the convention established in the `descriptor-mnemonic` (md1) repo.

## Why

`design/FOLLOWUPS.md` captures the *outcome* of each agent run (deferred items, closing commits). This directory captures the *raw report* — the implementer's or reviewer's full reasoning, including parts that didn't make it into FOLLOWUPS or commit messages but are useful for later audit.

If a future review asks "why did Task X take that approach?", the answer should be readable from the report alone, without `git show` archaeology.

## File naming

```
design/agent-reports/
├── README.md                       # this file
├── v0-1-spec-review-<N>.md         # closure-design or spec review (round N)
├── v0-1-plan-review-<N>.md         # implementation-plan review (round N)
├── v0-1-phase-<P>-task-<X>.md      # single-agent task report
├── v0-1-phase-<P>-bucket-<X>.md    # parallel-batch bucket report
├── v0-1-phase-<P>-review-<commit>.md  # phase review report
└── v0-1-phase-<P>-fixup-<commit>.md   # fix-up implementer report
```

Versioned prefix (`v0-1-...`) keeps reports grouped by release. Future versions roll the prefix.

## Convention for agent dispatches

When the controller dispatches an implementer or reviewer subagent, the prompt SHOULD include:

> Save your final report (the same text you return to me) to `design/agent-reports/<filename>.md` as part of your commit. Use the file-naming convention in `design/agent-reports/README.md`.

For **parallel-batch dispatches**, each agent saves to a distinct file (no conflicts since filenames embed the bucket id). The controller's post-batch aggregation reads these files and appends to FOLLOWUPS.

For **single-agent dispatches**, the report file lands alongside the work commit; durable independently.

## Format

Reports are Markdown. Header block required; body conventional.

```markdown
# <Title — phase + task or bucket id>

**Status:** DONE | DONE_WITH_CONCERNS | BLOCKED | NEEDS_CONTEXT
**Commit:** <SHA(s) — every commit produced; for review-only reports, the SHA being reviewed>
**Reviewer / Implementer:** Claude Opus 4.7 (1M context) | other
**Date:** YYYY-MM-DD
**File(s):** <every file path read or modified, one per line if multiple>
**Role:** implementer | reviewer (spec) | reviewer (plan) | reviewer (code) | fixup

## Summary
<1-3 sentences>

## <body sections — issues, confirmations, observations, etc.>
```
