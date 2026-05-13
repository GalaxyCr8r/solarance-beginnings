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
| #16 | Players can create/join factions | `milestone-req`, `factions` | **Move → M3** (rescope) | Keep, but cap to Lrak + Rediar; pick-one-on-spawn. |
| #18 | NPCs auto-mine, auto-trade, or guard nearby assets | `milestone-req` | **Defer → `future-vision`** | No NPCs in MVP. v1.0 (Persistent NPC economy). |
| #19 | Add faction chat | `milestone-req`, `factions` | **Move → M3** | Chat tables exist (per gap analysis); wire up end-to-end. |
| #29 | Add proper configuration to store OIDC info and other things | `client-side`, `milestone-req` | **Keep no-milestone** | Infra. Not gate-blocking but useful before launch (M6). |
| #42 | Evaluate and create issues for the next three milestones | `documentation`, `milestone-req` | **Replace + close** | Replaced by this folder; close with a link to `docs/milestones/`. |
| #49 | Add emotes | `client-side` | **Move → M5** | Cozy aesthetic — fits the no-voice-chat "two players who don't talk but wave" beat. |
| #61 | Update subscriptions for station modules only to return in-sector modules | `client-side` | **Move → M4** | Load-bearing for ten-sector world. |
| #68 | begin npcs — simple behavior tree | `server-side`, `npcs-impl` | **Defer → `future-vision`** | NPC spike cancelled by MVP design doc. |
| #80 | Refactor Server Reducers/Timers | `server-side` | **Keep no-milestone** | Tech debt. Opportunistic. |

---

## Issues currently in Milestone 5 🏗️

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #17 | NPCs visible in-world, loosely autonomous | `milestone-req` | **Defer → `future-vision`** | No NPCs in MVP. |
| #28 | Create loading screen/use threads to load images | `good first issue`, `client-side`, `milestone-req` | **Move → M4** | Needed once ten sectors of art are loading. |
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
| #51 | Implement better HUD for nearby sector objects | `good first issue`, `client-side` | **Move → M5** | Polish; fits the cozy-feel pass. |

---

## Issues currently in Milestone 7 🌌

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #23 | Add React-Based GitHub Page | `documentation`, `polish`, `milestone-req` | **Move → M6** | The devlog page. Single-platform public presence. |
| #53 | Add solar systems | `server-side` | **Defer → `future-vision`** | MVP is one system. Multi-system is v1.0+. |
| #54 | Add dynamic sector backgrounds based on the solar system | `client-side` | **Move → M5** | Polish; one-system version is still useful. |
| #55 | Add system chat | `client-side`, `server-side` | **Defer → `future-vision`** | Cross-system chat presupposes multiple systems. |

---

## Issues currently with no milestone (recent, post-movement-refactor)

These are the most MVP-relevant issues in the tracker — most should move into M0–M2.

| # | Title | Current labels | Suggested action | Rationale |
|---|---|---|---|---|
| #71 | Unify Ship tables | `server-side` | **Keep no-milestone** | Tech debt. Opportunistic. |
| #75 | Refactor Stations to use Views instead of RLS | `client-side`, `server-side` | **Move → M2** | Pairs with #84; load-bearing for welcome-back subscription scoping. |
| #76 | Refactor Player_Controller into Reducers instead of Timers | `client-side`, `server-side` | **Keep no-milestone** | Tech debt from movement refactor; not gate-blocking. |
| #80 | Refactor Server Reducers/Timers | `server-side` | **Keep no-milestone** | (Already listed above; same disposition.) |
| #81 | Create/remove mining effect when mining | `client-side`, `server-side`, `polish` | **Move → M5** | Mining loop polish. |
| #82 | Shared-Building Spike: Station Contribution Mechanic + Welcome-Back Message | *(none)* | **Move → M1** (split if needed) | **This is M1.** Welcome-back half may move to M2 if it grows. |
| #84 | Per-sector visibility scoping via SpacetimeDB views | `enhancement`, `server-side`, `agent-triaged`, `agent-human-needed` | **Move → M2** | Foundation for welcome-back, load-bearing for M4 too. |
| #85 | Client-side input prediction + reconciliation for the player's own ship | `enhancement`, `client-side`, `server-side`, `agent-triaged`, `agent-human-needed` | **Move → M0** | This *is* the movement critical-path the MVP doc calls out. |
| #86 | Minimum-latency server_offset clock-skew estimator (client) | `enhancement`, `client-side`, `agent-triaged`, `agent-eligible` | **Move → M0** | Prerequisite for #85. |
| #87 | Generalize visual_effects for mining-laser broadcast | `server-side`, `polish`, `agent-triaged`, `agent-eligible` | **Move → M5** | Mining polish. |
| #88 | Evaluate removing the StellarObject table | `enhancement`, `server-side`, `agent-triaged`, `agent-eligible` | **Keep no-milestone** | Tech debt; resolve opportunistically. |
| #89 | Cross-sector render flicker on jumpgate transit | `client-side`, `polish`, `agent-triaged`, `agent-eligible` | **Move → M4** | Polish for the ten-sector world. Players will be jumping a lot. |

---

## Summary by new milestone

| New milestone | Issues |
|---|---|
| **M0** Movement Critical-Path | #85, #86 |
| **M1** Shared-Building Spike | #82 (primary) |
| **M2** Persistence + Welcome-Back | #75, #84, (#82 welcome-back half if split) |
| **M3** Two-Faction MVP Setup | #16, #19 |
| **M4** Multi-Sector World Buildout | #28, #34, #61, #89 |
| **M5** Mining Loop + Polish | #49, #51, #54, #81, #87 |
| **M6** MVP Launch & Devlog | #23, #29 |
| **Keep no-milestone (tech debt)** | #50, #62, #71, #76, #80, #88 |
| **Defer → `future-vision`** | #11, #17, #18, #22, #24, #40, #53, #55, #68 |
| **Close** | #41 (dup of #85), #42 (replaced by this folder) |

---

## Re-milestoning checklist

When ready to execute against GitHub:

1. Create new milestones in GitHub matching M0–M6 in the new roadmap.
2. For each issue above, set its new milestone (or clear it for no-milestone / `future-vision`).
3. Apply the `future-vision` label to every deferred issue. Strip `agent-eligible` / `agent-triaged` / `agent-blocked` / `agent-human-needed` from those same issues — they're no longer in the agent queue (see `docs/issues/CONTEXT.md` for the agent triage state machine).
4. Re-label `milestone-req` only on issues that are blocking the milestone they now sit in. The old `milestone-req` label is stale on most of these.
5. Close old milestones (M6, M7, M8, M9, M10) with a closing comment that links here.
6. Rename Milestone Alpha → "Post-MVP Review" and update its description.
7. Close #42 and #41 with cross-links.

To audit later: `is:open label:future-vision` is the Future Vision backlog. `is:open -label:future-vision no:milestone` is the no-milestone tech-debt drawer.
