# Solarance: Beginnings — Design Document

*A cozy persistent space MMO for adults with jobs. Contribute to something bigger than yourself in the time you have. Your progress will be waiting for you.*

---

## Purpose of this document

This is the single source of truth for what Solarance: Beginnings is, and — just as importantly — what it is not. When scope creep, shiny-object syndrome, or months away from the project threaten to unmoor the plan, this document is the anchor. Re-read it before starting a new branch of work. If something you want to build isn't in the MVP section, it belongs in Future Vision. That's the rule.

---

## The Player

Solarance is designed for one specific player. Every feature must pass the test: *does this serve him?*

**David, 38.** IT Systems Analyst / mid-level manager. Married, two young kids (ages 4 and 7). Mentally drained after work. Wants games that calm him down, not wind him up.

His library: No Man's Sky (relaxed mode), Factorio and Dyson Sphere Program (peaceful), Melvor Idle, Old School RuneScape, and — eight years ago — EVE Online, which he loved for its economy and quit after a PvP gank destroyed a month of work. He has never gone back.

**Solarance is for the player EVE traumatized and lost.**

David's play pattern is intermittent: sometimes 20 minutes between household demands, sometimes a Saturday afternoon. Most sessions are short. Some weeks he can't play at all. The game must respect that — his progress must wait for him, and when he returns he must not feel behind.

---

## Genre and tone

Solarance is a **cozy, persistent, co-op space sandbox**. It is not a shooter. Combat exists only as environmental weather — never as a required player activity. The core emotional beat is *social convergence on shared construction*: players arriving, contributing, and watching something grow together.

**Competitive reference points:**
- **Melvor Idle** — progress-while-away as core retention mechanic
- **No Man's Sky (peaceful)** — a living, wondrous environment that doesn't threaten
- **Old School RuneScape** — respects intermittent time investment; dated visuals forgiven because the game keeps its promises
- **EVE Online (cooperative null-sec sov-building, minus the PvP)** — the specific feeling of a group slowly building up a presence in a quiet corner of space

Solarance is none of these games. It is the game each of them hints at for David.

---

## The MVP

### Core loop

**Find → Extract → Haul → Contribute → Watch it grow.**

1. **Find** a resource source (hand-placed in the MVP; no exploration mechanics)
2. **Extract** it (simple mining interaction, no heatmap yet)
3. **Haul** it back to a partially-built station (this is what "trade" means in the MVP — delivery, not markets)
4. **Contribute** it to the station's construction pool
5. **Watch** the station grow as you and other players contribute

The climactic emotional beat is step 5: *my pile got bigger, other people's piles got bigger, and together we made a thing.*

### MVP pillar: Expansion (building) is primary

Every other pillar collapses into a service role for Expansion:

| Pillar | Role in MVP |
|---|---|
| **Expansion (build)** | **Primary.** The reason to play. |
| Exfiltrate (mine) | Service role. Produces the resources that feed Expansion. |
| Exchange (trade) | Service role. Collapses into "haul ore to construction site." No markets. |
| Exploration | Stubbed. Hand-placed points of interest. No procedural generation, no sensor upgrades, no wormholes, no anomalies. |
| Exterminate (combat) | **Absent.** Zero combat gameplay in MVP. |

### World scope

- **One solar system.** Named, hand-designed.
- **Ten sectors.** Hand-placed. Each has a clear purpose (asteroid sector, station sector, jumpgate hub, etc.).
- **Sectors are connected by jumpgates.** Clear point-to-point teleportation. No FTL travel mechanics. No procedural sector generation.

### Stations

- **One station per sector, maximum.** This forces social gravity — everyone in this sector is helping build *the* station here. It also simplifies technology and makes each sector memorable ("the refinery sector," "the research sector"). Players upgrade the size of the existing station rather than sprawling.
- **Stations are warehouses with modules.** Pick a size, fill it with specialized modules: Storage (solid/liquid/gas), Refinery, Production, Assembly, Research, Repair, Defense. Each module has its own storage for its own needs; storage modules can serve other modules.
- **All stations accept docking from all ship sizes.**
- **Each faction has one Capital-class station** in the solar system. Capital stations are where new players spawn and have room for many modules.
- **Stations persist.** Contributions persist. What you build is still there tomorrow, next week, next month.

### Ships

- **One ship per player in the MVP.** A corvette / light-freighter jack-of-all-trades. It can mine slowly, haul, and travel. Specialization (dedicated miners, explorers, builders, freighters) is post-MVP.
- Prototype art already exists for this ship.

### Factions

- **Two factions only: Lrak Combine (red) and Rediar Federation (blue).**
- **No faction lore, politics, goals, research, or cross-faction mechanics in the MVP.**
- A faction in the MVP is: a color, a name, and the set of stations you're helping build. Lrak players contribute to red stations. Rediar players contribute to blue stations. That's it.
- Lore exists in the designer's head and in the Future Vision section. It does not belong in the MVP.

### Persistence

- **Stations, contributions, and player inventory persist between sessions.**
- **Welcome-back screen** (text-only, no LLM): list of current assets, current station progress, simple counters (trades since last login, credits earned, ships visited), and a scoped notification list. Notifications have scopes (personal / faction / system) and priorities.
- **No passive income in MVP.** When David logs off, resource generation pauses for his personal assets. The wider world (NPC traffic, station visits) still ticks.

### What David's MVP session looks like

David logs in. He sees: "Welcome back. Outpost Echo (Rediar construction site) is 34% complete. 3 players contributed since your last login. You have 240 units of iron ore in storage."

He flies his corvette to the asteroid sector, mines for 15 minutes, hauls the ore back to the construction site, and deposits it. The station's progress bar ticks up. He sees another player's ship dock at the same site while he's there. They don't talk. They both know what they're doing. He logs off.

This is the game. If this loop is not satisfying, nothing else matters. If this loop *is* satisfying, everything else is amplification.

### What is NOT in MVP

These features do not exist in the MVP and must not be built until the core loop is proven fun:

Player organizations (factions only, no sub-groups). Royalty systems of any kind. Faction goals, faction research, faction politics. Passive / offline income generation. Mining minigame and heatmap mechanics. Drones. Multiple ships per player. Ship-switching / specialized hulls. Wormholes, anomalies as gameplay mechanic (sensor pulses, discovery rewards, exploration as a discipline), non-jumpgate FTL. Procedural sector or system generation. Combat of any kind. Vancellan Swarm. Free Trade Union, Independent Worlds Alliance. Civilian NPC AI. Pirate NPC AI. AI-generated welcome-back summaries.

*Permitted as MVP flavor (no gameplay mechanic attached):* purely decorative anomalies — nebula sprites rendered in-sector at fixed coordinates. These are larger, slower-moving cousins of the background-system nebulae and reuse existing art assets. They distinguish sectors visually; they do not gate or alter any player action. The anomaly *mechanic* (sensors, discovery, exploration gameplay) remains Future Vision.

---

## Technical scope for MVP

- **Expected concurrent audience: 5–10 players.** This is an *audience-size expectation* given current marketing presence (an inactive Discord, occasional posts in SpacetimeDB and indie-game-dev communities) — not an engineering target. SpacetimeDB handles concurrency transparently; earlier versions of this repo have already been accessed by 3+ people simultaneously with no special work. Two-player, ten-player, twenty-player — the server doesn't care, and neither does the designer.
- **No orchestrator concerns.** SpacetimeDB hosts the module on Maincloud; sector handoff, worker management, and split/merge are not the designer's problem at MVP scale (or any plausible post-MVP scale for the foreseeable future). Don't pre-engineer for an orchestrator that may never exist.
- **Movement system rework is on the critical path — but narrowly scoped.** The current client/server prediction model does not correctly handle arresting all rotational velocity. That specific issue (or an acceptable workaround) must be resolved before the two-player shared-building spike. Everything else about the movement system is "done enough for MVP." NPC movement is not a concern — there are no NPCs in the MVP.
- **Art assets: reuse the 2010 Solarance assets** where they fit the cozy aesthetic. The corvette prototype art is already done.

---

## Success criteria for the MVP

The MVP is done when David (or a realistic proxy for David) can:

1. Log in and immediately understand what he's supposed to do.
2. Complete the find → extract → haul → contribute loop in a short session.
3. See other players' contributions reflected in the shared station.
4. Log out and return days later to find his progress and the station's progress preserved.
5. Report that he *wants to log in again tomorrow*.

Criterion 5 is the only one that matters. The others are prerequisites.

---

## The next spike

**Build the minimum shared-building loop between two players.**

Two players. One partially-built station in a sector. Each player can haul resources (spawned in inventory for testing — no mining yet) to it. The station visibly updates for both players in real time. Completion triggers a shared moment.

No mining. No combat. No exploration. No persistence across sessions yet. No factions. No NPCs.

The question this spike must answer: *does the core emotional beat — social convergence on shared construction — actually feel good when stripped to its simplest form?*

If yes: add persistence next (the welcome-back screen for a single player returning to a saved station).

If no: stop. Do not build more systems on top of a core loop that doesn't work. Figure out why it doesn't work, or reconsider the project.

The previously planned civilian NPC AI spike is **cancelled**. Civilian NPCs are Future Vision flavor, not MVP foundation.

---

## Marketing and community

MVP phase marketing commitment: **a public devlog, updated on a consistent cadence (e.g., monthly), on a single platform.**

The bar is not "how much do I have to show." The bar is "am I willing to be boring publicly for 18 months." A devlog with 40 followers after a year is not failure; it is the job. Indie multiplayer games without a pre-launch community launch to empty servers regardless of code quality.

Minimum viable public presence for MVP phase:
- One devlog post per month, even if it's short.
- The existing Discord, kept alive with those same updates.
- Honest scope — no hype, no promises about features that are in Future Vision.

Bigger marketing (trailer, Steam page, influencer outreach) is post-MVP and outside the scope of this document.

---

## Development constraints

This project is built by one person with a full-time job, young children, and ADHD-pattern focus challenges. The design above is shaped by those constraints, not in spite of them.

**Rules of engagement:**
- If a feature is not on this document's critical path, it is a distraction.
- Technically interesting problems (orchestrators, sector handoff protocols, AI-generated summaries, procedural generation) are traps when the core loop is unproven. They are *delicious* — which is exactly why they must wait.
- When in doubt, re-read the section "What David's MVP session looks like." If a feature doesn't show up in that paragraph, it is not MVP.
- When adding an idea to Future Vision, write it down the same day. Do not try to hold it in your head. Do not expand the MVP to accommodate it.

---

## Future Vision

*This section is a structured scratchpad. When ideas occur, write them here so they stop occupying mental space. Nothing in this section is committed. Nothing in this section is allowed to leak into the MVP section.*

### v1.0 — Systems that make the world feel alive

- **Persistent NPC economy.** Free Trade Union ships as the NPC trader layer. NPC ships visit player stations while offline. "Your station facilitated 42 trades" becomes a real number, not a placeholder.
- **Faction rep and faction-weighted economy.** Selling the right ore at the right time to the right faction grants rep and bonus credits. **Same-faction contribution** to construction sites grants rep points and (over time) compounding contribution bonuses. **Cross-faction contribution** still works — the goods are accepted — but pays out at flat market value with no rep accrual. Some faction pairings are antagonistic enough that cross-contribution may carry a *negative* rep modifier with your own faction. This preserves the "friends in different factions can still play together" affordance from the MVP while giving same-faction play a long-term progression hook.
- **Simplified royalty system.** Players who mark a point of interest earn a flat credit reward from nowhere (not player-funded) when another player uses it. No market distribution model needed.
- **Civilian NPC AI.** Background ships flying around to make sectors feel populated.
- **Pirate NPC encounters** as avoidable environmental threats. No player-side combat skill required; defenders/autoturrets handle it.
- **Mining as a minigame with a heatmap.** Players must shift position and beam intensity to maintain yield. Anti-botting by design.
- **Specialized ship hulls.** Dedicated miners, freighters, builders, explorers. Players own and switch between multiple ships.
- **Player organizations (orgs).** Sub-groups within factions. Shared storage, shared beacons, shared construction goals.
- **The two additional factions:** Independent Worlds Alliance, Free Trade Union (now playable as well as economic layer).
- **Exploration mechanics.** Sensor pulses, upgradeable sensor range, hand-rewarded discovery. Still no procedural generation.
- **Faction research trees.** Collective progress unlocks new ship modules, new mining types, etc.
- **"Welcome back" summaries enriched** with detailed event logs (still no LLM).
- **A second solar system,** connected by a jumpgate to the first.

### v1.1 — The living cosmos

- **Vancellan-touched sectors** as environmental pressure. Not enemies — a condition. Mining yields increase in Vancellan-affected sectors because of biological residues; NPC patrol frequency also increases. Players choose the risk/reward tradeoff.
- **LLM-generated welcome-back narrative summaries** as an optional polish layer (gated by cost analysis).
- **More solar systems.** Gas giants with dozens of sectors around lagrange points and moons, making a single system feel like a small galaxy.
- **Additional FTL travel types** (warp drives, wormholes). Unlocking FTL is a game-shaking event.
- **Anomalies and rare points of interest.** Exploration becomes its own discipline.
- **Drones.** Customizable mining and utility drones that accompany the player.
- **Fleet commander role** — preparation for v2.0 combat.

### v2.0 — Combat as an opt-in pillar

- **Playable combat** for players who want it. Fleet command over small squadrons of ships.
- **Vancellan as a genuine invasion faction** in combat-enabled sectors.
- **PvP in designated empty / lawless sectors only.** The "care bear" core is inviolable. Core sectors remain safe.
- **Large-scale faction warfare** as opt-in emergent content.

### Parking lot (uncategorized)

*Use this space to drop raw ideas that haven't been triaged. Don't worry about which version they belong to yet.*

- *(add ideas here as they occur; triage later)*

---

## Changelog of this document

- **Initial version** — established MVP scope, cut shooter framing, committed to Expansion as the primary pillar, committed to the shared-building spike as the next development task.
