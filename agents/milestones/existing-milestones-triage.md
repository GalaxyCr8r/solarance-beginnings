# Existing Milestones — Triage

Disposition for each currently-open GitHub milestone, measured against the [MVP design doc](../Solarance_Beginnings_MVP_Design_Doc.md).

**Legend**

- **CLOSE** — Out of MVP scope. Retire the milestone; the issues underneath get the `future-vision` label (preserved as backlog) or close as `wontfix`/`duplicate` (rejected outright).
- **RESCOPE** — The headline goal is still aligned, but the deliverables list mixes MVP and post-MVP work. Rewrite the milestone's scope and prune issues.
- **MERGE** — The intent is partly MVP-relevant but belongs combined with another milestone in the new roadmap (see [`proposed-roadmap.md`](proposed-roadmap.md)).
- **DEFER** — Keep on the books with the `future-vision` label, not active work.

Issues tagged `future-vision` are *not* in the agent queue and should not carry `agent-eligible` or other agent-triage labels.

---

## Milestone 4 👥 — *Factions, Players, and NPC Fleets!*

**Original goal:** Players form factions, faction NPCs spawn proportionally, NPCs auto-mine/auto-trade/guard.

**Verdict: RESCOPE → split into two new milestones.**

The "two factions exist, players join one of them" half is MVP (it's the second pillar of the loop — *which* colour station you contribute to). The "NPC fleets" half is explicitly out of MVP:

> *"Civilian NPC AI." — Future Vision v1.0. The previously planned civilian NPC AI spike is cancelled.*
> — MVP design doc

**Action:**
- Keep faction-joining work, hard-cap to **Lrak + Rediar only** — see new milestone **M3 — Two-Faction MVP Setup**.
- All NPC behavior, NPC fleet spawning, and NPC chat work → label `future-vision`, remove milestone.
- Close the milestone itself once issues are redistributed.

---

## Milestone 5 🏗️ — *Base-Building!*

**Original goal:** Deploy stations, mine→build→refine→protect chain. "Can be attacked (simple damage system)."

**Verdict: RESCOPE — but this is the core MVP pillar.**

The Expansion pillar *is* MVP. What's wrong with M5 as written is two things: (1) it bakes combat in ("can be attacked, simple damage system") — combat is absent in MVP; (2) it implies sprawling player-deployed stations, but the MVP commits to **one station per sector**, upgraded in place, not freely placed.

> *"Stations are warehouses with modules… One station per sector, maximum."*
> — MVP design doc

**Action:**
- Drop the damage/combat deliverable entirely.
- Reframe from "players deploy stations" → "players contribute resources to a pre-placed station's construction pool, watching it grow."
- Replaced by new milestones **M1 — Shared-Building Spike** and **M2 — Single-Player Persistence + Welcome-Back**.
- Galaxy Creator tool (#34) and loading screen (#28) stay, but move to the world-buildout milestone (**M4** in the new plan).

---

## Milestone 6 ⚔️ — *Initial PvP & Warfare!*

**Original goal:** Weapon systems, ship/station destruction, territory claims, fog-of-war.

**Verdict: CLOSE.**

This milestone is the single largest disagreement between old plan and new plan.

> *"Exterminate (combat) — Absent. Zero combat gameplay in MVP."*
> — MVP design doc

Combat is Future Vision v2.0 — not v1.0, not even v1.1. The "care bear" core is explicitly inviolable.

**Action:**
- Close the milestone outright.
- Issues like #22 ("destroyed ships create asteroids") → label `future-vision`, remove milestone.
- Genuinely useful but mis-labelled issues here (#41 — client-side caching/prediction; #51 — HUD for nearby objects) get re-homed in **M5 — Mining Loop Polish** under the new plan. The prediction issue overlaps with #85 already.

---

## Milestone 7 🌌 — *Dynamic Universe & Emergent Gameplay!*

**Original goal:** NPC factions scale, simulated trade convoys, contracts, admin tools.

**Verdict: CLOSE.**

Every deliverable here is explicitly Future Vision:

- NPC simulation → v1.0 ("Persistent NPC economy")
- Contracts → not in the MVP critical path's emotional beat
- Admin tools → useful but not MVP-blocking

> *"No passive income in MVP. When David logs off, resource generation pauses for his personal assets."*
> — MVP design doc

**Action:**
- Close the milestone.
- Multi-system work (#53, #55) → label `future-vision`, remove milestone (the MVP is **one** solar system). #54 (dynamic backgrounds) is salvageable for one system — re-home in **M5**.
- Devlog / public-facing presence work (#23) gets re-homed in **M6 — MVP Launch & Devlog**.

---

## Milestone 8 🌠 — *Multi-System Travel & Interstellar Infrastructure!*

**Verdict: CLOSE.** Zero open issues. The MVP is one solar system. This is Future Vision v1.0 ("A second solar system, connected by a jumpgate to the first").

**Action:** Close the milestone. The headline lives in Future Vision already. No issues to relabel.

---

## Milestone 9 🪐 — *Interstellar Trade, Logistics, and Diplomacy!*

**Verdict: CLOSE.** Zero open issues. Diplomacy, treaties, cross-system pricing — every deliverable is explicitly NOT in MVP per the "What is NOT in MVP" list.

**Action:** Close the milestone.

---

## Milestone 10 🚨 — *Interstellar Conflict and Empire-Scale Gameplay!*

**Verdict: CLOSE.** Zero open issues. Combat at empire scale — Future Vision v2.0 at the earliest.

**Action:** Close the milestone.

---

## Milestone Alpha — *Major polish/redesign milestone*

**Original goal:** First major polish/redesign. Re-engage market research, learn from EVE/Vendetta/X4. *"Every milestone before this is just to establish an engine."*

**Verdict: RESCOPE → rename "Post-MVP Review".**

The instinct behind Milestone Alpha — *stop and reassess once a foundation exists* — survives the rewrite. But it was framed around a different foundation (NPC fleets, combat, multi-system). With the MVP redefined, Alpha should be repurposed as the gate *after* the MVP ships and the devlog has run for some months: do players want to come back? What did the gap analysis miss?

**Action:**
- Rename "Milestone Alpha" → "Post-MVP Review".
- Reset its description to point at the MVP success criteria (specifically criterion 5: *the player wants to log in again tomorrow*) as the gate.
- It stays empty until MVP ships.

---

## Summary

| GH Milestone | Disposition | New home for live issues |
|---|---|---|
| M4 👥 — Factions + NPC Fleets | RESCOPE (split) | M3 (factions) + Future Vision (NPCs) |
| M5 🏗️ — Base-Building | RESCOPE (cut combat) | M1, M2, M4 (new plan) |
| M6 ⚔️ — PvP & Warfare | CLOSE | Future Vision v2.0 |
| M7 🌌 — Dynamic Universe | CLOSE | Future Vision v1.0–v1.1 |
| M8 🌠 — Multi-System | CLOSE | Future Vision v1.0 |
| M9 🪐 — Interstellar Trade | CLOSE | Future Vision v1.0 |
| M10 🚨 — Interstellar Conflict | CLOSE | Future Vision v2.0 |
| Alpha — Polish/Redesign | RESCOPE → "Post-MVP Review" | gate after MVP ships |

See [`proposed-roadmap.md`](proposed-roadmap.md) for the new milestone sequence, and [`issue-disposition.md`](issue-disposition.md) for per-issue re-homing.
