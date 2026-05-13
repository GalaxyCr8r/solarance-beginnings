# Milestone Planning

This folder exists because the GitHub milestones in `GalaxyCr8r/solarance-beginnings` were written for a different game than the one the MVP design doc commits to.

Milestones 1–3 are done. M4–M10 and "Milestone Alpha" are open and most of them encode the *old* scope: NPC fleets, PvP combat, multi-system travel, interstellar diplomacy, empire warfare. The [MVP design doc](../Solarance_Beginnings_MVP_Design_Doc.md) cuts all of that. The job of these documents is to make the gap explicit and propose a path forward.

## Files

| File | Purpose |
|---|---|
| [`existing-milestones-triage.md`](existing-milestones-triage.md) | Disposition (keep / rescope / defer / close) for each currently-open GitHub milestone. |
| [`proposed-roadmap.md`](proposed-roadmap.md) | A new sequence of MVP-aligned milestones, with success criteria and exit gates. The starting point for replacing the old plan. |
| [`issue-disposition.md`](issue-disposition.md) | Every open issue mapped to a suggested action: which new milestone, defer to Future Vision, or close as out-of-scope. |

## How to use this

1. Read `existing-milestones-triage.md` first. It's the "what to do with what's already in GitHub" decision sheet.
2. Read `proposed-roadmap.md` next. That's the shape of work going forward.
3. Use `issue-disposition.md` as the operational checklist when you actually go and re-label / re-milestone the GitHub issues.

## Source documents

- [`../Solarance_Beginnings_MVP_Design_Doc.md`](../Solarance_Beginnings_MVP_Design_Doc.md) — the authoritative scope.
- [`../reports/mvp-gap-analysis.md`](../reports/mvp-gap-analysis.md) — what already exists in the server vs. what the MVP needs. Cross-referenced throughout.
- [`../reports/deprecated-station-modules.md`](../reports/deprecated-station-modules.md) — station modules removed during the recent cleanup; useful when scoping the construction system.

## Rule

If a feature isn't in the [MVP design doc](../Solarance_Beginnings_MVP_Design_Doc.md), it belongs in Future Vision. These docs apply that rule to the issue tracker.

## The `future-vision` label

Issues that are out-of-MVP-scope but still valid (i.e. the idea is good, just not now) get the `future-vision` label rather than being closed as `wontfix`. The distinction matters:

- **`wontfix`** — the issue itself is rejected: wrong, obsolete, or actively undesirable.
- **`future-vision`** — the issue is preserved as a backlog item for v1.0+, gated by the MVP shipping first.

The agent triage labels (`agent-eligible`, `agent-blocked`, etc.) should **not** be applied to `future-vision` issues — they're not in the active queue. Filter the agent queue with `-label:future-vision`.
