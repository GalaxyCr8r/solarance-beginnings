# Open Issues — Suggested Disposition

Per-issue triage for every open GitHub issue at the time of writing. Use this as the operational checklist when re-labelling and re-milestoning in GitHub.

See [`proposed-roadmap.md`](proposed-roadmap.md) for the new milestone definitions referenced below.

**Action vocabulary**

- **Move → Mn** — Re-milestone to the new milestone.
- **Defer → `future-vision`** — Keep the issue open, remove its milestone, apply the `future-vision` label, and strip any `agent-*` triage labels. This is the **default** for out-of-MVP-scope ideas that are still valid — preserved as backlog, but not closed and not in the agent queue.
- **Close** — Out of scope *and* not worth preserving (duplicate, replaced, or actively obsolete). Close with `wontfix`/`duplicate` and a comment pointing here. Use sparingly — prefer `future-vision` unless the issue is genuinely dead.
- **Keep no-milestone** — Tech-debt / infra / triage work that runs alongside milestones but isn't gate-blocking. Acceptable to do opportunistically.
- **Replace** — Issue is superseded by the new milestone planning itself; close it.

---

## Issues currently in Milestone 4 👥

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #11 | Add 3 major NPC factions with "elevator" stations | `milestone-req`, `factions` | **Defer → `future-vision`** | MVP is two factions, no NPCs. v1.0 territory. |
| #16 | Players can create/join factions | `milestone-req`, `factions` | **Split. #16a → M3 (rescope), #16b → `future-vision`** | #16a: rename to *"Players can join one of two factions on first login"*. Existing client UI already prompts for faction selection — temporarily disable everything except Lrak and Rediar, and color the egui buttons by faction color. #16b: rename to *"Players can create organizations"*; orgs are sub-groups within factions (Future Vision v1.0), not player-created factions. |
| #18 | NPCs auto-mine, auto-trade, or guard nearby assets | `milestone-req` | **Defer → `future-vision`** | No NPCs in MVP. v1.0 (Persistent NPC economy). |
| #19 | Add faction chat | `milestone-req`, `factions` | **Move → M3 (rescope to verification)** | Server-side already exists. Rescope to *"Verify faction chat works end-to-end with multiple players"* — multi-client smoke test before MVP launch. Update body accordingly. |
| #29 | Add proper configuration to store OIDC info and other things | `client-side`, `milestone-req` | **Keep no-milestone** | Infra. Not gate-blocking but useful before launch (M6). |
| #42 | Evaluate and create issues for the next three milestones | `documentation`, `milestone-req` | **Replace + close** | Replaced by this folder; close with a link to `docs/milestones/`. |
| #49 | Add emotes | `client-side` | **Defer → `future-vision`** | Designer feels strongly about this for cozy non-verbal communication between nearby players — but explicitly not for MVP. Implementation note for future: transient `VisualEffect`-style **Event table** (SpacetimeDB's Event tables broadcast on insert without persisting rows). |
| #61 | Update subscriptions for station modules only to return in-sector modules | `client-side` | **Close — superseded by M7 (Views)** | Filed before SpacetimeDB Views were available. The mechanism #61 proposed (SQL-subset client subscriptions) is replaced by Views, which let the server restrict per-client visibility without requiring `station_module` to be public at all. Close with a cross-link to #84/#75. |
| #68 | begin npcs — simple behavior tree | `server-side`, `npcs-impl` | **Defer → `future-vision`** | NPC spike cancelled by MVP design doc. |
| #80 | Refactor Server Reducers/Timers | `server-side` | **Keep no-milestone** | Tech debt. Opportunistic. |

---

## Issues currently in Milestone 5 🏗️

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #17 | NPCs visible in-world, loosely autonomous | `milestone-req` | **Defer → `future-vision`** | No NPCs in MVP. |
| #28 | Create loading screen/use threads to load images | `good first issue`, `client-side`, `milestone-req` | **Defer → `future-vision`** | Already attempted; stutters even when image loading runs on a separate thread. Not a blocker for MVP. Different concern: brief "loading" screen on sector-jump (once Views shape subscriptions) is tracked by #84/#89, not #28. |
| #34 | Create "Galaxy Creator" privileged client | `server-side`, `milestone-req` | **Move → M4** | Directly enables the seven-new-sector buildout. |
| #40 | Create privileged game client for controlling faction NPCs | `client-side`, `server-side`, `milestone-req` | **Defer → `future-vision`** | NPC-driven; out of MVP. |
| #50 | Actually implement client-side subscriptions if RLS still hasn't been fixed | `client-side` | **Keep no-milestone** | Related to #84 (views vs RLS). Track jointly. |
| #62 | Time how long biggest timers take to complete | `server-side` | **Keep no-milestone** | Perf observation; useful but not gating. |

---

## Issues currently in Milestone 6 ⚔️

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #22 | Destroyed ships create "asteroids" | `server-side`, `polish`, `milestone-req` | **Defer → `future-vision`** | Requires combat; combat is v2.0. |
| #24 | Create basic forums for GitHub page | `documentation`, `client-side`, `polish`, `milestone-req` | **Defer → `future-vision`** | Discord covers MVP-scale community. Forums are post-launch. |
| #41 | Client-side caching and basic prediction | `client-side`, `milestone-req` | **Close as duplicate of #85** | #85 is the active, scoped version of this work post-movement-refactor. Cross-link and close. |
| #51 | Implement better HUD for nearby sector objects | `good first issue`, `client-side` | **Close + file replacement** | The screenshot in the body is from a combat prototype (hull/shield/energy bars inside a radar circle). The radar circle is already implemented. Close #51 and file a focused replacement: *"Improve radar object distance readability"* — driven by the designer's observation that the current build does a poor job of conveying object distance. Keep no-milestone (opportunistic polish, refine based on user feedback). |

---

## Issues currently in Milestone 7 🌌

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #23 | Add React-Based GitHub Page | `documentation`, `polish`, `milestone-req` | **Defer → `future-vision`** | Title is misleading — issue is actually a TypeScript React app for SpacetimeDB login + account management + billing operations. Not a devlog. Account creation in MVP is handled by Auth0 OIDC; billing is a post-launch concern (no charging until there's a fun game). Update issue title/body to reflect actual scope before deferring. **Devlog work is split off into a new issue (see below).** |
| #53 | Add solar systems | `server-side` | **Defer → `future-vision`** | MVP is one system. Multi-system is v1.0+. |
| #54 | Add dynamic sector backgrounds based on the solar system | `client-side` | **Close as already-implemented + file replacement** | The basic 2.5D background system is already live: jumping into "Homeworld" shows a habitable planet centered; "Beta" shows a gas giant in the background with the star to the south and the habitable planet to the right. Close #54. File a Future Vision replacement: *"Asteroid and nebula fields as 2.5D background elements based on sector proximity to those stellar objects."* |
| #55 | Add system chat | `client-side`, `server-side` | **Defer → `future-vision`** | Cross-system chat presupposes multiple systems. |

---

## New issues to file (work uncovered by this planning pass)

| Proposed title | Rationale | Disposition |
|---|---|---|
| Add `/devlog/` section to `solarance-beginnings.com` GitHub Pages site | Devlog cadence needs a canonical durable home that isn't Discord chat. Page already exists; this just adds a posts directory and structure. First post can be written the same day it's set up. Cross-posts from Discord/Bluesky/Itch.io link here. | **M1** (parallel-track, small) |
| #82a — Station Contribution Mechanic (split from #82) | The M1 half of #82: `contribute_to_station` reducer, real-time progress, completion broadcast. | **M1** |
| #82b — Welcome-Back ServerMessage (split from #82) | The M2 half of #82: `client_connected` composes a welcome-back message with station progress, contributions since last login, current cargo. | **M2** |
| #16a — Players can join one of two factions on first login (split from #16) | Faction-selection UI hard-caps to Lrak + Rediar, egui buttons coloured by faction. | **M3** |
| #16b — Players can create organizations (split from #16) | Orgs are sub-groups within factions, not player-created factions. Future Vision v1.0. | **Defer → `future-vision`** |
| Enforce one Column corvette per player; reject Phalanx/Javelin in `create_player_controlled_ship` | One-ship-per-player enforcement; ship-type lockdown. | **M3** |
| #75a — Sector-scoped station view via Views (split from #75) | The "sector of a ship you control" + "sector of a station your faction controls" filters. | **M7** |
| #75b — Persistent fog-of-war for visited stations (split from #75) | The "visited before" filter; fog-of-war is exploration mechanics. | **Defer → `future-vision`** |
| Improve radar object distance readability (replaces #51) | Radar circle exists; current build does a poor job conveying object distance. Tweak based on user feedback. | **Keep no-milestone** (opportunistic polish) |
| Asteroid and nebula fields as 2.5D background elements based on sector proximity (replaces #54's deferral) | Extends the existing 2.5D background system from stellar objects to asteroid/nebula fields. | **Defer → `future-vision`** |

---

## Issues currently with no milestone (recent, post-movement-refactor)

These are the most MVP-relevant issues in the tracker — most should move into M0–M2.

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #71 | Unify Ship tables | `server-side` | **Keep no-milestone** | Tech debt. Opportunistic. |
| #75 | Refactor Stations to use Views instead of RLS | `client-side`, `server-side` | **Move → M7** | Pairs with #84. Not actually load-bearing for welcome-back (that's a one-shot ServerMessage), so it doesn't gate MVP. First post-MVP work for anti-cheat hardening. |
| #76 | Refactor Player_Controller into Reducers instead of Timers | `client-side`, `server-side` | **Keep no-milestone** | Tech debt from movement refactor; not gate-blocking. |
| #80 | Refactor Server Reducers/Timers | `server-side` | **Keep no-milestone** | (Already listed above; same disposition.) |
| #81 | Create/remove mining effect when mining | `client-side`, `server-side`, `polish` | **Move → M5** | Mining loop polish. |
| #82 | Shared-Building Spike: Station Contribution Mechanic + Welcome-Back Message | *(none)* | **Move → M1** (split if needed) | **This is M1.** Welcome-back half may move to M2 if it grows. |
| #84 | Per-sector visibility scoping via SpacetimeDB views | `enhancement`, `server-side`, `agent-triaged`, `agent-human-needed` | **Move → M7** | Designer's chosen anti-cheat mechanism — fundamental for protecting client-visible state. Can be deferred till after MVP, but wanted soon thereafter. Reference impl in `docs/tmp/views/`. |
| #85 | Client-side input prediction + reconciliation for the player's own ship | `enhancement`, `client-side`, `server-side`, `agent-triaged`, `agent-human-needed` | **Keep no-milestone** | Solarance is not a quick-action game — full prediction/reconciliation is not MVP-blocking. Revisit only if the game feels sluggish during public MVP testing (M6). |
| #86 | Minimum-latency server_offset clock-skew estimator (client) | `enhancement`, `client-side`, `agent-triaged`, `agent-eligible` | **Keep no-milestone** | Same as #85 — only needed if full client-side prediction becomes necessary. |
| #87 | Generalize visual_effects for mining-laser broadcast | `server-side`, `polish`, `agent-triaged`, `agent-eligible` | **Move → M5** | Mining polish. |
| #88 | Evaluate removing the StellarObject table | `enhancement`, `server-side`, `agent-triaged`, `agent-eligible` | **Keep no-milestone** | Tech debt; resolve opportunistically. |
| #89 | Cross-sector render flicker on jumpgate transit | `client-side`, `polish`, `agent-triaged`, `agent-eligible` | **Move → M4** | Polish for the ten-sector world. Players will be jumping a lot. |

---

## Summary by new milestone

| New milestone | Issues |
|---|---|
| **M0** Movement Critical-Path | ✅ Already complete (resolved by `docs/tmp/movement_system_plan.md` implementation) |
| **M1** Shared-Building Spike | #82a (split from #82) + new `/devlog/` section issue |
| **M2** Persistence + Welcome-Back | #82b (split from #82) |
| **M3** Two-Faction MVP Setup | #16a (split + rescoped), #19 (rescoped to verification) |
| **M4** Multi-Sector World Buildout | #34, #89 |
| **M5** Mining Loop + Polish | #81, #87 |
| **M6** MVP Launch & Devlog | #29 *(by M6 the devlog has months of posts already)* |
| **M7** Anti-Cheat Hardening via Views | #75a (split from #75), #84 |
| **Keep no-milestone (tech debt + opportunistic polish)** | #50, #62, #71, #76, #80, #85, #86, #88, plus new "Improve radar object distance readability" issue |
| **Defer → `future-vision`** | #11, #16b (split from #16), #17, #18, #22, #23, #24, #28, #40, #49, #53, #55, #68, #75b (split from #75), plus new "asteroid/nebula 2.5D backgrounds" issue |
| **Close** | #41 (dup of #85), #42 (replaced by this folder), #51 (replaced by radar-readability issue), #54 (already implemented), #61 (superseded by M7 Views work), #82 (replaced by #82a + #82b), #75 (replaced by #75a + #75b), #16 (replaced by #16a + #16b) |

---

## Activation checklist

Execute in order. Each numbered step is independent; sub-bullets are inside one step.

### 1. Create new GitHub milestones

- **M1** Shared-Building Spike
- **M2** Single-Player Persistence + Welcome-Back
- **M3** Two-Faction MVP Setup
- **M4** Multi-Sector World Buildout
- **M5** Mining Loop + Polish
- **M6** MVP Launch & Devlog
- **M7** Anti-Cheat Hardening via Views

M0 is intentionally skipped — it's already complete (resolved by the movement-system rewrite in `docs/tmp/movement_system_plan.md`).

### 2. Close obsolete milestones

Close with a closing comment that links to `docs/milestones/`:

- Milestone 4 👥 (Factions + NPC Fleets)
- Milestone 5 🏗️ (Base-Building)
- Milestone 6 ⚔️ (PvP & Warfare)
- Milestone 7 🌌 (Dynamic Universe)
- Milestone 8 🌠 (Multi-System)
- Milestone 9 🪐 (Interstellar Trade)
- Milestone 10 🚨 (Interstellar Conflict)

### 3. Rename Milestone Alpha → "Post-MVP Review"

Reset its description to point at MVP success criterion 5 (*the player wants to log in again tomorrow*) as the gate. Empty until MVP ships *and* M7 (Views) completes.

### 4. Split existing issues into new issues

For each split below: create the two new issues, cross-link them in both directions, then close the parent with a comment pointing at both children.

| Parent | Children |
|---|---|
| **#82** Shared-Building Spike (huge bundle) | **#82a** Station Contribution Mechanic → M1, body = the contribute-reducer + real-time progress + completion broadcast sections of #82. **#82b** Welcome-Back ServerMessage → M2, body = the `client_connected` + welcome-back sections of #82. |
| **#16** Players can create/join factions | **#16a** Players can join one of two factions on first login → M3, body = hard-cap to Lrak + Rediar, egui buttons coloured by faction. **#16b** Players can create organizations → `future-vision`, body = orgs are sub-groups within factions, v1.0 work. |
| **#75** Refactor Stations to use Views instead of RLS | **#75a** Sector-scoped station view → M7, body = "ship-you-control sector" + "faction-station sector" filters. **#75b** Persistent fog-of-war for visited stations → `future-vision`, body = the "visited before" filter (exploration mechanics). |

### 5. File new issues that don't replace existing ones

Grouped by milestone. Each issue should be created with a body that names *what the work delivers* and *what its acceptance criterion is*, not just a title.

**M1 — Shared-Building Spike (client-side + test scaffolding)**

| Title | Notes |
|---|---|
| Add `/devlog/` section to `solarance-beginnings.com` GitHub Pages site | Parallel-track. Small. Page already exists; this adds the posts directory + structure + first post. |
| Construction-site UI: progress bar, contributions panel, deposit-from-cargo button | The client-side half of the spike. #82a is server-only per its parent's framing — without this, the spike has no observable progress bar. |
| Admin/seed reducer: spawn test cargo in player inventory for spike testing | M1 explicitly uses inventory-spawned cargo (no mining yet). Probably 30 minutes of work, but file it so it doesn't get forgotten. |

**M2 — Persistence + Welcome-Back (client-side + missing server pieces)**

| Title | Notes |
|---|---|
| Welcome-back panel: client-side rendering of the welcome-back `ServerMessage` | The render half. #82b is server-side composition only. |
| `ServerMessage` notification scopes (personal/faction/system) + priorities | Named explicitly in the MVP design doc but not in #82's user stories. Either lands in M2, or the design doc trims the requirement for MVP — file the issue and resolve in the body. |
| Persistence smoke test: stations + contributions survive server restart | M2's exit gate needs a concrete pass/fail checklist. Per gap analysis most of this already works; verify it explicitly. |

**M3 — Two-Faction Setup (UI affordance + spawn behaviour)**

| Title | Notes |
|---|---|
| Enforce one Column corvette per player; reject Phalanx/Javelin in `create_player_controlled_ship` | Single-ship rule. |
| Faction-flag UI affordance: highlight own-faction construction sites, mute others | Operationalizes the M3 "soft default" decision. Without this issue, the design point is only words — there's no UI evidence of which sites are "yours." |
| Verify each faction has one Capital-class station and that new players spawn there | The MVP doc says new players spawn at the Capital station. Per gap analysis the station rows exist, but spawn-on-Capital is a separate behavior that needs verification. |

**M4 — Multi-Sector World Buildout (the bulk of M4 is currently un-issued)**

| Title | Notes |
|---|---|
| Design and seed seven additional MVP sectors | The largest piece of M4 work and currently has no issue. Body should enumerate each sector's functional differentiators (yield, station size, demand profile, faction, density, gate topology) per the M4 design table in `proposed-roadmap.md`. May warrant sub-issues per sector once seeded — file as a tracking issue. |
| Render decorative in-sector nebulae from X/Y data | The MVP-doc carve-out from Q8. Reuse background-nebula art at smaller in-sector scale. No game mechanic — pure render. |
| Verify jumpgate connectivity across all ten sectors | Subtask of sector seeding; deserves its own checkpoint so the gate "player can jumpgate between all ten" isn't fuzzy. |

**M5 — Mining Loop + Polish**

| Title | Notes |
|---|---|
| Mining loop end-to-end verification | Gap analysis says most server-side mining exists. Concrete verification: target asteroid → mine → cargo populated → haul to site → contribute → progress bar moves. Without this, "mostly done" hides bugs that surface during M6 smoke test. |

(Polish-level issues — completion animations, sound cues, contribution feedback — are intentionally *not* pre-filed. They emerge from playtesting; pre-filing risks scope creep.)

**M6 — MVP Launch & Devlog**

| Title | Notes |
|---|---|
| MVP smoke test session with welcome-back verification | M6's exit gate is the design doc's 5-criteria checklist, but no issue currently captures the smoke-test *session itself* — date, recruitment from Discord, observation protocol, what counts as pass/fail. The point is *someone other than the designer* hitting the loop end-to-end; concurrency is handled by SpacetimeDB so headcount is not the bar. |

**No milestone (opportunistic polish)**

| Title | Notes |
|---|---|
| Improve radar object distance readability | Replaces #51. Tweak based on user feedback. |

**Future Vision**

| Title | Notes |
|---|---|
| Asteroid and nebula fields as 2.5D background elements based on sector proximity | Replaces #54's deferral. Extends the existing stellar-object 2.5D system to asteroid/nebula sources. |

### 6. Edit existing issue titles/bodies before re-milestoning

| # | Action |
|---|---|
| **#19** Add faction chat | Rewrite body: *"Server-side faction chat tables already exist. Scope is multi-client smoke-test verification before MVP launch. **File separate issues for any bugs found; do not fix them in this issue.**"* (Last sentence prevents scope creep if verification surfaces real bugs.) |
| **#23** Add React-Based GitHub Page | Edit title to *"React-based account / billing client for SpacetimeDB"*. Body should clarify this is the account management + billing app, not a devlog page. Then defer to `future-vision`. |
| **#28** Loading screen | Add body explaining the threading attempt didn't resolve stutter; defer to `future-vision`. |
| **#34** Galaxy Creator | **Constrain body before M4 starts.** Pin the scope to *"a privileged client that supports exactly these seed operations for M4: sector creation, station placement, jumpgate connection. Anything else is out of scope."* Without this, "investigate and implement" can balloon from a CLI to a full React app. |
| **#49** Add emotes | Add implementation note: transient `VisualEffect`-style Event table. Defer to `future-vision`. |
| **#54** Dynamic sector backgrounds | Close as already-implemented (cross-link to the new asteroid/nebula 2.5D backgrounds Future Vision issue). |
| **#84** Per-sector visibility scoping | **Decide before M7 starts: split per-table or stay bundled.** Body currently lists 5 tables to convert (`ship`, `asteroid`, `cargo_crate`, `visual_effect`, `stellar_object`). Each is substantial. Either split into 5 sub-issues for tracking, or leave bundled and add a table-by-table checklist to the body. Don't enter M7 with this unresolved. |
| **#87** Generalize visual_effects for mining-laser broadcast | No changes needed; move to M5. |
| **#89** Cross-sector render flicker on jumpgate transit | No changes needed; move to M4. |

### 6a. Scope-creep fixes inside the splits (apply when creating the children)

These are revisions that *must* happen at split time, not later. They prevent the parent's stale framing from being inherited verbatim by the children.

| When splitting | Required fix |
|---|---|
| **#82 → #82a** (Station Contribution Mechanic) | **Drop user story #7** from the parent body, which says *"contribute to a station that belongs to my faction only."* That directly contradicts the M3 soft-default decision (any player may contribute to any site). Also drop the matching out-of-scope item that talks about "non-MVP factions" — the reducer no longer enforces faction restriction at all. |
| **#82 → #82b** (Welcome-Back ServerMessage) | **Resolve the welcome-back-counters question.** #82's user stories 14–17 cover *construction progress, contributions, and cargo* — but the MVP design doc's welcome-back spec also names *trades since last login, credits earned, ships visited*. Either #82b explicitly adds those counters (likely zero in MVP since markets/trades don't exist yet) or the MVP doc's welcome-back content list is trimmed. Decide before activating M2. |

### 7. Close issues with cross-links

- **#41** Client-side caching and basic prediction — close as duplicate of #85; cross-link.
- **#42** Evaluate and create issues for next three milestones — close as replaced by this folder; link to `docs/milestones/`.
- **#51** Implement better HUD for nearby sector objects — close as replaced by the new "Improve radar object distance readability" issue.
- **#61** Update subscriptions for station modules only — close as superseded by M7 (Views); cross-link to #84.

### 8. Re-milestone all remaining issues per the summary table above

For each issue with a new milestone assignment, set its milestone in GitHub.

### 9. Re-label deferred and tech-debt issues

For every issue moved to `future-vision`:

- Apply the `future-vision` label.
- Strip `agent-eligible` / `agent-triaged` / `agent-blocked` / `agent-human-needed` — they're no longer in the agent queue (see `docs/issues/CONTEXT.md` for the agent triage state machine).
- Remove the milestone (if any).

For every issue kept as no-milestone tech debt:

- Remove the milestone (if any).
- Keep agent labels if appropriate.

### 10. Audit `milestone-req` label usage

The `milestone-req` label was applied broadly in the old plan. Strip it from any issue that is *not* gating its new milestone. Re-apply only where the issue genuinely blocks the milestone exit gate.

### 11. Final audit queries

After all activation steps run, these queries should match expected counts:

- `is:open label:future-vision` → Future Vision backlog (~15 from old issues + 4 from splits/replacements = ~19)
- `is:open -label:future-vision no:milestone` → no-milestone tech-debt + opportunistic polish drawer (~10)
- `is:open milestone:M1` → #82a, devlog `/devlog/` section issue, Construction-site UI issue, admin-seed-cargo issue
- `is:open milestone:M2` → #82b, welcome-back panel client-side issue, notification scopes/priorities issue, persistence smoke test issue
- `is:open milestone:M3` → #16a, #19 (rescoped), one-ship-enforcement issue, faction-flag UI affordance issue, Capital-station-spawn verification issue
- `is:open milestone:M4` → #34, #89, seven-sector seed-data design issue, decorative nebulae rendering issue, jumpgate connectivity verification issue
- `is:open milestone:M5` → #81, #87, mining loop end-to-end verification issue
- `is:open milestone:M6` → #29, MVP smoke test session issue
- `is:open milestone:M7` → #75a, #84
