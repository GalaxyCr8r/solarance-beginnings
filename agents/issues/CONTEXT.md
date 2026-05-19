# GitHub Issues — Context

Conventions for filing issues in `GalaxyCr8r/solarance-beginnings`. The repo is a cozy 2D space MMO built on SpacetimeDB; issues are split between a Rust server module (`server/`) and a Rust executable client (`client/`).

Most issues should carry **one type label**, **one or more area labels**, and **any applicable scope/triage labels**. Labels are additive — a single feature touching both sides should get both `client-side` and `server-side` (see #85 for a canonical example).

---

## Type — what kind of work

Pick exactly one.

| Label | Use when |
|---|---|
| `bug` | Existing behavior is wrong, crashing, or diverging from intent. |
| `enhancement` | New feature, capability, or significant behavioral change. |
| `documentation` | Docs, ADRs, CONTEXT files, README updates. |
| `question` | Open question or design discussion with no decided action. |

---

## Area — where the change lives

Apply all that fit.

| Label | Use when |
|---|---|
| `server-side` | Touches `server/` — tables, reducers, DSL, scheduled timers. |
| `client-side` | Touches `client/` — React UI, rendering, input, subscriptions. |
| `factions` | Factions, organizations, reputation, faction-owned assets. |
| `npcs-impl` | NPC behavior, AI controllers, NPC ship spawning. |

If an issue spans both client and server, apply both — don't pick the "primary" one.

---

## Scope / quality

| Label | Use when |
|---|---|
| `polish` | Visual, UX, or feel improvement that gets the game closer to final vibes. Not load-bearing for milestones. |
| `milestone-req` | Required to complete the current milestone. Use sparingly — this is a commitment. |

---

## Agent triage workflow

These labels coordinate which issues are LLM-agent-ready. They form a small state machine, not a free-for-all.

| Label | Meaning |
|---|---|
| `agent-triaged` | An LLM has read the issue, confirmed scope, and added enough context to act on it. Apply once triage is complete. |
| `agent-eligible` | The issue is a candidate for an LLM agent to implement, partially or fully. Apply after triage if the work is well-scoped, low-blast-radius, and doesn't require live judgement. |
| `agent-blocked` | Triage stalled — the agent needs a human decision before continuing. The blocker should be stated in a comment. |
| `agent-human-needed` | Triage complete, but the work itself needs a human — agent skills aren't sufficient for full implementation or triage. Different from `agent-blocked`: nothing the human says will unblock an agent here, the human has to do it. |
| `agent-wont-fix` | Out of scope for agents — duplicate, too risky, too cross-cutting, or otherwise shouldn't be picked up by an agent. The issue itself may still be valid; this only excludes it from the agent queue. |

Typical flow: new issue → `agent-triaged` → one terminal state:
- `agent-eligible` — pick it up
- `agent-blocked` — waiting on a human answer (resumes after)
- `agent-human-needed` — handed off to a human implementer
- `agent-wont-fix` — removed from the agent queue entirely

---

## Meta / admin

Standard GitHub defaults. Use when applicable; not all issues need one.

- `good first issue` — small, contained, good for newcomers.
- `help wanted` — maintainers want outside input or implementation.
- `duplicate`, `invalid`, `wontfix` — closing reasons; pair with a closing comment that explains.

---

## What a well-labeled issue looks like

- **UI flicker on jumpgate transit** → `bug`, `client-side`, `polish`
- **Remove the StellarObject table** → `enhancement`, `server-side`
- **Client-side input prediction** → `enhancement`, `client-side`, `server-side`
- **Add new NPC freighter behavior** → `enhancement`, `server-side`, `npcs-impl`

---

## Writing the issue body

Keep the body diagnostic-first — the goal is that a future engineer (or agent) reading `spacetime logs` and the issue together can act without re-deriving context.

- **What's wrong / what's needed** — one paragraph.
- **Where in the code** — file paths and, when possible, reducer or component names. The mental model in `CLAUDE.md` applies: bugs live as "wrong row in a table" or "wrong reducer ran."
- **Repro or acceptance criteria** — concrete enough that "done" is unambiguous.
- **Related issues / ADRs** — link them; don't restate them.
