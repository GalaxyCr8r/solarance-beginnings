# Proposed Roadmap — MVP-Aligned Milestones

A new milestone sequence anchored to the [MVP design doc](../Solarance_Beginnings_MVP_Design_Doc.md). Read [`existing-milestones-triage.md`](existing-milestones-triage.md) first for context on why the previous milestones are being retired or rescoped.

The sequencing follows the design doc's own ordering: **shared-building spike → persistence → full MVP loop → polish/launch.** Each milestone has an explicit *exit gate* — the question that must be answered "yes" before moving to the next one.

---

## Guiding Constraints

These shape every milestone below.

- **One player to please: David.** Every deliverable answers *does this serve him?*
- **Target: 5–10 concurrent players.** No orchestrator. Single worker process.
- **Solo developer with limited time.** If a milestone needs more than 4–6 weeks of evening work, split it.
- **The core loop is sacred.** *Find → Extract → Haul → Contribute → Watch it grow.* Polish around it; do not extend it sideways until it is proven fun.
- **Devlog cadence starts at M1, not M6.** Public commitment is *the* multiplier; the rule is "willing to be boring publicly for 18 months."

---

## M0 — Movement Critical-Path Fix

**Why it exists:** The MVP doc names exactly one technical blocker for the spike:

> *"The current client/server prediction model does not correctly handle arresting all rotational velocity. That specific issue (or an acceptable workaround) must be resolved before the two-player shared-building spike."*

Nothing else from the movement system overhaul is gating MVP — only the rotational-velocity bug.

**Scope**

- Resolve the rotational-velocity arresting bug (workaround acceptable).
- Whatever client-side prediction / clock-skew work is the minimum to unblock #82 — likely **#85** and **#86**.

**Out of scope**

- Per-sector view scoping (#84) — important, but moves to **M2**.
- General reducer/timer refactor (#80) — tech-debt, not MVP-blocking.

**Exit gate:** *Two clients can move their ships, jump between sectors, and stop cleanly without rotational drift.*

---

## M1 — Shared-Building Spike

**Why it exists:** This *is* the next spike from the design doc:

> *"Build the minimum shared-building loop between two players. Two players. One partially-built station in a sector. Each player can haul resources (spawned in inventory for testing — no mining yet) to it. The station visibly updates for both players in real time. Completion triggers a shared moment."*

#82 already exists for this work.

**Scope**

- `contribute_to_station` reducer: moves cargo from `ShipCargoItem` into a station's construction pool, increments progress.
- Client subscribes to `StationUnderConstruction`; progress renders in real time.
- Construction-site UI for depositing.
- Cargo spawned in inventory for testing — **no mining yet**.
- Completion event broadcasts a shared visible moment (a chat line, a flash, anything — the medium is less important than the fact that both players see it together).

**Out of scope**

- Mining (M3 onwards).
- Welcome-back screen (M2).
- Persistence across server restart for the test (we can wipe between spikes).
- Faction filtering (M3).

**Exit gate:** *The designer (or a David proxy) reports that the moment two players watched the bar move together was emotionally satisfying.*

This is the only milestone where the exit gate is **subjective**, and deliberately so. The MVP doc is explicit:

> *"If yes: add persistence next. If no: stop. Do not build more systems on top of a core loop that doesn't work."*

A "no" at this gate is a project-level decision, not a milestone-level one.

---

## M2 — Single-Player Persistence + Welcome-Back

**Why it exists:** Per the design doc, the work *immediately after the spike succeeds* is persistence:

> *"If yes: add persistence next (the welcome-back screen for a single player returning to a saved station)."*

**Scope**

- Stations + contributions survive server restart (most of this is already true; verify and harden).
- `client_connected` lifecycle reducer composes a welcome-back `ServerMessage` with:
  - station progress %,
  - number of contributions since last login,
  - current cargo on hand,
  - simple counters (trades since last login, credits earned, ships visited).
- Notification scopes (personal / faction / system) + priorities — the design doc names these.
- Client renders the welcome-back panel on connect. **Text-only. No LLM.**
- Per-sector view-based subscriptions (#84) — load-bearing for the welcome-back screen and for keeping the client lean.

**Out of scope**

- LLM-generated welcome-back summaries — Future Vision v1.1, gated by cost analysis.
- Passive offline income — explicitly excluded by MVP.
- Mining minigame / heatmap — Future Vision.

**Exit gate:** *A player can log out, come back the next day, and the welcome-back screen accurately reflects what happened while they were away (including another player's contributions if any).*

---

## M3 — Two-Faction MVP Setup

**Why it exists:** The MVP commits to exactly two joinable factions and exactly one ship type per player. Right now the server has six faction definitions and three ship types — both need to be locked down before the loop reaches a real player.

**Scope**

- Hard-cap joinable factions to **Lrak Combine + Rediar Federation**. Hide IWA, FTU, Vancellan, Procyon Compact from faction selection (do not delete — they're future).
- Player can choose one of the two factions on first login (#16, rescoped).
- One ship per player enforced: only the Column (corvette). Phalanx and Javelin remain in tables for future use but `create_player_controlled_ship` rejects them.
- Construction sites are faction-flagged. Lrak players contribute to red sites; Rediar players contribute to blue. This is the **"which colour do I help"** mechanic from the design doc.
- Faction chat wired up end-to-end (#19) — chat tables already exist server-side.
- Each faction has one Capital-class station per the design doc.

**Out of scope**

- Sub-organisations / orgs within factions — explicitly excluded.
- Faction reputation system — Future Vision v1.0.
- Faction research trees — Future Vision v1.0.
- Cross-faction politics, treaties, etc. — Future Vision.

**Exit gate:** *A new player picks Lrak or Rediar, spawns in their Capital station, flies a Column corvette, and can only see and contribute to construction sites for their own faction.*

---

## M4 — Multi-Sector World Buildout

**Why it exists:** MVP world scope is ten sectors. Three are seeded. The remaining seven need design and seeding before the loop is interesting.

**Scope**

- Design + seed seven additional sectors. Each has a clear purpose ("the refinery sector", "the research sector", "the jumpgate hub", etc.).
- One station per sector — pre-placed in `init`. Different progress states across sectors so a returning player has variety.
- Galaxy Creator privileged client (#34) — used by the designer to lay out sectors and adjust station placement without rebuilding the module.
- Jumpgate network connecting all ten sectors. Cross-sector flicker fix (#89).
- Subscriptions for in-sector station modules only (#61).
- Loading screen / image preload (#28) — long sector lists need it.

**Out of scope**

- Procedural sector generation — explicitly forbidden by MVP.
- Multiple star systems — Future Vision.
- Specialty sectors that require non-MVP mechanics (anomalies, nebulas, etc.) — leave hooks, don't build.

**Exit gate:** *A player can jumpgate between all ten sectors, each sector visibly differs from the others, and each has a partially-constructed station with persistent state.*

---

## M5 — Mining Loop + Polish

**Why it exists:** Up to this point, the spike used inventory-spawned cargo. Now the *Find → Extract* half of the core loop has to be real. Plus the cozy aesthetic the design doc calls out has to actually land.

**Scope**

- Mining works end-to-end: target an asteroid, mine for ~15 minutes, get ore, haul to construction site. Most of this exists server-side already (per gap analysis).
- Mining visual effects (#81, #87).
- HUD for nearby sector objects (#51).
- Dynamic sector backgrounds (#54).
- Emotes (#49) — fits the "cozy" aesthetic; two players who don't talk can still wave.
- Reuse 2010 Solarance art assets where they fit (per MVP design doc constraints).
- General "feel good" polish pass: completion animations, contribution feedback, sound cues.

**Out of scope**

- Mining minigame / heatmap — Future Vision.
- Combat visual effects, weapon firing — combat is not in MVP.
- Pirate / civilian NPC AI — Future Vision.

**Exit gate:** *David sits down for a 20-minute session, mines, hauls, contributes, watches the bar move, and reports it felt good.*

---

## M6 — MVP Launch & Devlog

**Why it exists:** The design doc is explicit about the marketing minimum:

> *"A public devlog, updated on a consistent cadence (e.g., monthly), on a single platform… The bar is 'am I willing to be boring publicly for 18 months.'"*

**Scope**

- Monthly devlog post on a single platform (pick one — likely a static site or GitHub Pages, see #23).
- Existing Discord kept alive with the same updates.
- Public-facing landing page (#23) — minimal; the devlog *is* the page.
- 5–10 player smoke test. Real-world or arranged. Observe the welcome-back screen working for someone other than the designer.
- Final round of bug-fixing in the core loop.

**Out of scope**

- Steam page, trailer, influencer outreach — explicitly post-MVP per design doc.
- Pre-launch hype cycle — designer rule is "no hype, no promises about Future Vision".
- Forums (#24) — defer; Discord serves the same purpose at this scale.

**Exit gate:** This is the MVP success criteria gate, verbatim from the design doc:

1. A player logs in and immediately understands what they're supposed to do.
2. They complete the find → extract → haul → contribute loop in a short session.
3. They see other players' contributions reflected in the shared station.
4. They log out and return days later to find both their progress and the station's progress preserved.
5. **They report that they *want to log in again tomorrow*.**

Criterion 5 is the only one that matters. The others are prerequisites.

---

## Post-MVP Review (formerly "Milestone Alpha")

Empty until MVP ships. Gate is criterion 5 above. Once it passes:

- Read the [Future Vision section](../Solarance_Beginnings_MVP_Design_Doc.md#future-vision) of the design doc.
- Run a real retro: what did the gap analysis miss, what was over-built, what did David actually use vs. ignore.
- Pick the *single* Future Vision v1.0 item with the highest ratio of "amplifies the existing loop" to "build cost". Build only that.

---

## What's *not* in this roadmap (and where it went)

| Old plan | New home |
|---|---|
| NPC mining/trade/guard fleets | Future Vision v1.0 (Persistent NPC economy) |
| Civilian NPC AI | Future Vision v1.0 |
| Pirate NPCs | Future Vision v1.0 |
| Combat / weapons / shields | Future Vision v2.0 |
| Ship destruction → asteroid loot | Future Vision v2.0 |
| Multiple star systems | Future Vision v1.0 (second system) → v1.1 (many systems) |
| Diplomacy / treaties / faction politics | Future Vision v1.0+ |
| Empire-scale warfare | Future Vision v2.0 |
| Player organisations within factions | Future Vision v1.0 |
| Markets / dynamic pricing | Future Vision v1.0 (faction-weighted economy) |
| Research trees | Future Vision v1.0 |
| Multiple ship hulls per player | Future Vision v1.0 |
| Drones | Future Vision v1.1 |
| Procedural generation | Explicitly forbidden — not in any planned phase |

Anything that doesn't appear in the MVP loop paragraph ("What David's MVP session looks like") lives in Future Vision. The whole point of this rewrite is to enforce that line.
