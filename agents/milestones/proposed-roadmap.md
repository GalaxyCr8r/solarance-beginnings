# Proposed Roadmap — MVP-Aligned Milestones

A new milestone sequence anchored to the [MVP design doc](../Solarance_Beginnings_MVP_Design_Doc.md). Read [`existing-milestones-triage.md`](existing-milestones-triage.md) first for context on why the previous milestones are being retired or rescoped.

The sequencing follows the design doc's own ordering: **shared-building spike → persistence → full MVP loop → polish/launch.** Each milestone has an explicit *exit gate* — the question that must be answered "yes" before moving to the next one.

---

## Guiding Constraints

These shape every milestone below.

- **One player to please: David.** Every deliverable answers *does this serve him?* (David is analagous too, but not exactly, the primary developer of this project)
- **Expected audience: 5–10 concurrent players.** Audience-size expectation given current marketing reach, not an engineering constraint. SpacetimeDB handles concurrency transparently; earlier versions of this repo have already been accessed by 3+ people simultaneously without bespoke work. Two-player or twenty-player, the server doesn't care.
- **Solo developer with limited time.** If a milestone needs more than 4–6 weeks of evening work, split it.
- **The core loop is sacred.** *Find → Extract → Haul → Contribute → Watch it grow.* Polish around it; do not extend it sideways until it is proven fun.
- **Devlog cadence starts at M1, not M6.** Public commitment is *the* multiplier; the rule is "willing to be boring publicly for 18 months."

---

## M0 — Movement Critical-Path Fix — ✅ COMPLETE

**Why it existed:** The MVP doc named exactly one technical blocker for the spike:

> *"The current client/server prediction model does not correctly handle arresting all rotational velocity. That specific issue (or an acceptable workaround) must be resolved before the two-player shared-building spike."*

**Status:** Resolved via the implementation of `docs/tmp/movement_system_plan.md`. The rotational-velocity arresting bug is no longer reproducible. No further movement work is on the MVP critical path.

**Deferred to tech-debt (revisit only if public MVP testing reveals sluggishness):**

- Full client-side input prediction + reconciliation (#85).
- Clock-skew estimator (#86).

Solarance is a cozy game, not a twitch shooter — sub-frame input feel is not on the critical path.

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
- **Parallel-track:** add a `/devlog/` section to the existing `solarance-beginnings.com` GitHub Pages site and write post #1 (about the spike itself). The page already exists; this is structural work + first post, not platform-building. Cadence from this point forward: one post per month, cross-posted from Discord/Bluesky/Itch.io to this canonical location. The Discord Devlog channel keeps running; it now links to the canonical posts rather than being the canonical posts.

**Out of scope**

- Mining (M5 onwards). *(Note: previous text said "M3 onwards" — corrected to M5, where the mining loop polish lives.)*
- Welcome-back screen (M2).
- Persistence across server restart for the test (we can wipe between spikes).
- Faction filtering (M3).

**Exit gate:** *The designer reports that the moment two players watched the bar move together was emotionally satisfying. Optionally validated with one or two Discord contacts in a voice-call play session, but the designer's honest reaction is sufficient — Solarance is being made for the designer first, and "find a human" is a marketing-funnel problem, not a validation problem.*

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

**Out of scope**

- Per-sector view-based subscriptions (#84) and station view refactor (#75) — moved to **M7** (first post-MVP work). Welcome-back is a one-shot `ServerMessage`, not an ongoing subscription, so views are not actually load-bearing here. RLS is sufficient through MVP.
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
- Construction sites are faction-flagged (red = Lrak, blue = Rediar). The flag is a **label and a default UI affordance**, not an exclusion gate — any player may contribute to any site. Your own faction's sites are highlighted / recommended on the welcome-back screen; the other faction's are visible and contributable but not promoted. This preserves the design doc's "which colour do I help" beat while keeping friends in different factions able to play together.
- Faction chat wired up end-to-end (#19) — chat tables already exist server-side.
- Each faction has one Capital-class station per the design doc.

**Out of scope**

- Sub-organisations / orgs within factions — explicitly excluded.
- Faction reputation system — Future Vision v1.0. *(Same-faction contribution will eventually grant rep + compounding bonuses; cross-faction contribution will pay out at flat market value with no rep. Captured for v1.0 below.)*
- Faction research trees — Future Vision v1.0.
- Cross-faction politics, treaties, etc. — Future Vision.

**Exit gate:** *A new player picks Lrak or Rediar, spawns in their Capital station, flies a Column corvette, sees their own faction's construction sites highlighted, and can contribute to any site they fly to.*

---

## M4 — Multi-Sector World Buildout

**Why it exists:** MVP world scope is ten sectors. Three are seeded. The remaining seven need design and seeding before the loop is interesting.

**Sector purpose is functionality-first, not theme-first.** A "refinery sector" is not just a visual reskin — it must *do* something different from the asteroid sector. Theme follows function, not the reverse. Functional differentiators viable in MVP (i.e. without re-implementing deprecated module logic) are to be decided as part of this milestone (see open design question below); station-module-as-flavor stands for MVP, but the *sector itself* needs functional reason to exist.

**Scope**

- Design + seed seven additional sectors. Each has a functional differentiator that distinguishes it from every other sector (open question: which differentiators).
- One station per sector — pre-placed in `init`. Different progress states across sectors so a returning player has variety. Two of the ten sectors are pre-claimed for the Lrak and Rediar Capital stations (per design doc).
- Galaxy Creator privileged client (#34) — used by the designer to lay out sectors and adjust station placement without rebuilding the module.
- Jumpgate network connecting all ten sectors. Cross-sector flicker fix (#89).
- Decorative-only nebulae (X/Y + sprite, no mechanic) in selected sectors — reuse existing art assets. Permitted by the MVP doc's clarified anomaly carve-out.

**Out of scope**

- Per-sector subscription filtering (#61) — closed; superseded by M7's Views work. Sector visibility scoping happens via Views, not client-side subscription queries.
- Functional station module logic (refineries refining, etc.) — Future Vision. Modules in MVP are flavor + flag.
- Asset preload / loading screen (#28) — already attempted; stutter persists even with threaded loading. Deferred to Future Vision; not MVP-blocking.
- Asteroid/nebula 2.5D background fields (replaces #54's deferral) — Future Vision. The basic stellar-object 2.5D background system is already implemented.
- Procedural sector generation — explicitly forbidden by MVP.
- Multiple star systems — Future Vision.
- Anomaly-as-mechanic (sensor pulses, discovery, exploration gameplay) — Future Vision.

**Sector differentiators for MVP (resolved):**

Core (every sector picks values for these):

1. **Asteroid yield type** — iron-rich, silicate-rich, ice/volatiles, etc. Per-sector resource composition.
2. **Station size at the construction site** — Capital / Large / Medium / Small. Already supported by the `Station` size enum.
3. **Construction pool resource demand profile** — *what* and *how much* each station needs. This is what turns "haul ore" into "haul the right ore to the right place" — the proto-economy that makes 5–10 players coordinate. Highest-leverage differentiator.
4. **Faction ownership** — Lrak / Rediar / unaligned. Drives the soft-default UI affordance from M3.

Variety amplifiers:

5. **Asteroid density / size distribution** — few large vs. many small.
6. **Gate connectivity** — hub (4+ connections) vs. spoke (1–2).
7. **Background visuals + ambient audio** — theme follows function. Per-sector asset references.
8. **Decorative anomalies (nebulae)** — pure render hint, X/Y + sprite, no game mechanic. In-sector nebulae reuse the existing background-nebula art at a different scale. The MVP doc was updated 2026-05-13 to explicitly permit decorative-only anomalies while keeping the anomaly *mechanic* in Future Vision.

**Exit gate:** *A player can jumpgate between all ten sectors, each sector functionally differs from the others (not just visually), and each has a partially-constructed station with persistent state.*

---

## M5 — Mining Loop + Polish

**Why it exists:** Up to this point, the spike used inventory-spawned cargo. Now the *Find → Extract* half of the core loop has to be real. Plus the cozy aesthetic the design doc calls out has to actually land.

**Scope**

- Mining works end-to-end: target an asteroid, mine for ~15 minutes, get ore, haul to construction site. Most of this exists server-side already (per gap analysis).
- Mining visual effects (#81, #87).
- Reuse 2010 Solarance art assets where they fit (per MVP design doc constraints).
- General "feel good" polish pass: completion animations, contribution feedback, sound cues.

**Out of scope**

- HUD/radar tweaks (#51 replacement) — handled opportunistically based on user feedback; not gating MVP.
- Dynamic sector backgrounds (#54) — basic 2.5D background system already implemented; asteroid/nebula extension is Future Vision.
- Emotes (#49) — designer wants them but explicitly not for MVP.
- Mining minigame / heatmap — Future Vision.
- Combat visual effects, weapon firing — combat is not in MVP.
- Pirate / civilian NPC AI — Future Vision.

**Exit gate:** *David sits down for a 20-minute session, mines, hauls, contributes, watches the bar move, and reports it felt good.*

---

## M6 — MVP Launch & Devlog

**Why it exists:** The design doc is explicit about the marketing minimum:

> *"A public devlog, updated on a consistent cadence (e.g., monthly), on a single platform… The bar is 'am I willing to be boring publicly for 18 months.'"*

By M6 the devlog has been running since M1 (see the parallel-track item in M1's scope). M6 is *not* where the devlog *starts* — it's where it gets the launch-amplification post and the smoke test happens.

**Scope**

- Continue the monthly devlog cadence established in M1.
- Existing Discord kept alive with cross-links to the canonical devlog posts.
- Landing page is already live at `solarance-beginnings.com` — minor polish if needed, but no platform work.
- Smoke test with whoever shows up — likely a handful of Discord contacts. The point is *someone other than the designer* using the welcome-back screen, not hitting any particular player-count threshold. Concurrency is a SpacetimeDB problem, not ours.
- Final round of bug-fixing in the core loop.

**Out of scope**

- Steam page, trailer, influencer outreach — explicitly post-MVP per design doc.
- Pre-launch hype cycle — designer rule is "no hype, no promises about Future Vision".
- Forums (#24) — defer; Discord serves the same purpose at this scale.
- React-based account/billing app (#23) — that's the *real* scope of #23, deferred to Future Vision. No billing until there's a fun game. Account creation in MVP is handled by Auth0 OIDC.

**Exit gate:** This is the MVP success criteria gate, verbatim from the design doc:

1. A player logs in and immediately understands what they're supposed to do.
2. They complete the find → extract → haul → contribute loop in a short session.
3. They see other players' contributions reflected in the shared station.
4. They log out and return days later to find both their progress and the station's progress preserved.
5. **They report that they *want to log in again tomorrow*.**

Criterion 5 is the only one that matters. The others are prerequisites.

---

## M7 — Anti-Cheat Hardening via Views (first post-MVP work)

**Why it exists:** Views are the designer's chosen mechanism for protecting client-visible state from cheaters — they allow per-client filtering of what ships, modules, and other entities are even sent to the client, rather than relying on RLS at the row level. The designer has flagged this as the first post-MVP priority, ahead of any Future Vision feature work.

This milestone is sequenced *before* Post-MVP Review because it's a known, scoped piece of work — not a "what should we build next" decision. It does not need to wait for criterion 5 to be argued through.

**Scope**

- Per-sector visibility scoping via SpacetimeDB views (#84).
- Refactor Stations to use Views instead of RLS (#75).
- Reference implementation in `docs/tmp/views/` (from the `Solarance-Movement-Prototype` repo).

**Exit gate:** *A client subscribed to the world only receives entities visible from its current sector. Tooling-level inspection of the wire confirms cross-sector data is not leaked.*

---

## Post-MVP Review (formerly "Milestone Alpha")

Empty until MVP ships *and* M7 (Views hardening) completes. Gate is criterion 5 above. Once it passes:

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
