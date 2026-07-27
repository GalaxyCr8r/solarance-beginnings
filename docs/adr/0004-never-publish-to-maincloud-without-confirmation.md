---
status: accepted
date: 2026-07-21
prompted-by: "operator directive"
---

# Never publish to maincloud without explicit operator confirmation

## Context

Maincloud (`maincloud.spacetimedb.com`) is the live, hosted database. `CLAUDE.md`
currently names maincloud as the default publish location and the database dashboard's
home. During active development we build and republish the module constantly — and the
publish tasks make maincloud a single flag away from local (`task server:publish-mc`,
`task server:publish-clear-mc`, `task server:delete-mc`). An accidental maincloud publish
would push unreviewed schema/logic to live players; a maincloud `--clear-database` /
`--delete-data` would destroy live state irrecoverably.

The local server is already the `spacetime` default (`127.0.0.1:3000`, marked `***` in
`spacetime server list`), so routine work has no reason to touch maincloud.

## Decision

All development publishing targets **local only**. No agent, script, or automation runs
any maincloud command — `spacetime publish --server maincloud` (`publish-mc`,
`publish-clear-mc`), `delete-mc`, or any maincloud data-clearing/delete — unless the
operator **explicitly asks for that maincloud action in the moment and confirms it**.

- Publishing to maincloud is never a default, a fallback, or a side effect of another task.
- **Clearing or deleting maincloud data requires a second, explicit confirmation** on top of
  the request — it is the highest-stakes action in this repo.
- This scopes the "maincloud is the default publish location" guidance in `CLAUDE.md` to
  *production release* only; during development the default is local, full stop.

## Why this shape

- **Rely on the `***` local default alone** — rejected. It only decides the *default*
  server; `--server maincloud` overrides it trivially, so the default is a convenience, not
  a guard.
- **A settings `deny`/`ask` permission rule or a pre-publish hook** — viable and stronger
  (hard enforcement the agent can't bypass). Not done here because the operator asked for a
  documented policy + agent note; a deny-rule can be layered on later if hard enforcement is
  wanted.
- **This ADR + an agent memory** (chosen) — documents the rule for humans and binds the
  agent's behavior across sessions, at zero tooling cost. Ceiling: it's a convention, not an
  interlock; escalate to the deny-rule above if that ceiling is ever hit.

## Consequences

- The agent asks before any maincloud publish and asks again before any maincloud clear/delete.
- `docs/RELEASING.md` and `CLAUDE.md` maincloud references are understood as
  *release-time, operator-initiated* steps, not routine ones.
- If stronger-than-convention enforcement is desired, add a `spacetime publish.*maincloud`
  `deny`/`ask` rule to `.claude/settings.json` (see the `update-config` skill).
