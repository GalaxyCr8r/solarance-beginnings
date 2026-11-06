# Issue Triage Report — 2026-11-06

Automated triage pass over **new / untriaged** open issues in `galaxycr8r/solarance-beginnings`.

**Scope of this pass.** The tracker has 67 open issues. All are unassigned except **#80** (GalaxyCr8r).
Issues ≤ #89 are already dispositioned in [`issue-disposition.md`](issue-disposition.md); the faction-epic
cluster (#205, #222–#234) and #85/#86/#88/#109/#125 already carry the `agent-triaged` marker. This report
covers the **22 open issues created after that baseline that have no triage treatment** (no `agent-triaged`
label, no milestone — every open issue currently shows no milestone).

Vocabulary follows `issue-disposition.md`: **Move → Mn**, **Defer → `future-vision`**, **Keep no-milestone**,
**Close**, **Replace**. No labels or milestones were changed — this is a review only.

---

## Summary table

| # | Title (short) | Area | Priority | Category / disposition | Duplicate / relates to |
|---|---|---|---|---|---|
| **#251** | BUY fails silently — no DM, no UI feedback | server + client | **High** | Bug. Fix on MVP trade path; gates #110 smoke test | — |
| **#242** | Asteroids broadcast exact resource amount/type | server | **High** | Anti-cheat → **M7**; split bonus goals to `future-vision` | **Subset of #177**, pairs with #84 |
| **#152** | Web build: compile client to WASM | client | **High** | Keep no-milestone (M6 enabler — removes the #1 smoke-test barrier) | — |
| #177 | Private entity tables via two-tier views | server + client | Medium | Anti-cheat foundation → **M7** | Generalizes #84; superset of #242 |
| #180 | Player-built station modules on completed stations | server + client | Medium | Shared-building extension (post-M1/M2) | Builds on contribution loop (#82a) |
| #186 | Player roles: admin + moderator | server | Medium | Infra, keep no-milestone | Blocks #185, #183 |
| #158 | Station modules: configurable input/output | server + admin | Medium | Galaxy-Creator / M4 support | #34, #159 |
| #159 | Galaxy Creator: top-level map tabs | client-admin | Medium | M4 tooling | #34, #155/#156/#157 |
| #157 | Galaxy Creator: Sector View via render_star_system() | client-admin | Medium | M4 tooling (blocked on #155) | #155, #159 |
| #155 | Extract render_star_system() into reusable crate | client | Medium | Tech debt, keep no-milestone (unblocks #157) | #157 |
| #156 | Galaxy Creator: per-sector Select button | client-admin | Low | M4 tooling | #34, #157 |
| #188 | Player map: "Sector Map" tab | client | Medium | Polish, keep no-milestone | #160, #190 |
| #190 | Minimap window: tabbed Map/Targets | client | Medium | Polish, keep no-milestone | #188, #160 |
| #160 | Player map: populate "Galaxy Map" tab | client | Low | **Defer → `future-vision`** (needs multi-system) | #188, #190 |
| #191 | Targeting HUD: name/type/distance | client | Medium | Polish, keep no-milestone | #192 |
| #192 | Press T to target object ahead; cycle in 5° cone | client | Medium | Polish, keep no-milestone | #191 |
| #243 | Top bar hotkeys require Ctrl held | client | Medium | Polish, keep no-milestone | hotkey cluster (#218) |
| #246 | Persist hotkey bindings between sessions | client | Medium | Polish, keep no-milestone | follow-up #218; lands before #247 |
| #247 | Hotkey manager tab in options window | client | Medium | Polish, keep no-milestone | follow-up #218; depends #246 |
| #185 | Mod/admin post into any chat channel | server + client | Low | Keep no-milestone (depends #186) | #186, #183 |
| #183 | Admin client: monitor every chat channel | server + admin | Low | Keep no-milestone | #185, #186 |
| #169 | Jumpgate transit warp effect | client | Low | Cosmetic polish → with #89 (M4) or no-milestone | split from #89 |

**Clusters worth handling together**

- **Anti-cheat views:** #84 (triaged) · #177 · #242 — one body of work, see below.
- **Galaxy Creator (M4 tooling):** #34 → #155 → #156 / #157 / #159 / #158.
- **Player maps / targeting:** #188 · #190 · #160 · #191 · #192.
- **Hotkeys (follow-ups to the closed #218):** #243 · #246 · #247.
- **Admin / moderation:** #186 → #185 / #183.

---

## Detailed notes — High priority

### #251 — Buying without enough credits fails silently
**Priority: High.** Not a cosmetic nit — it exposes a **systemic** defect on an MVP-critical path (buy/sell is
part of the economy loop the M6 smoke test #110 must exercise). Credits are correctly *not* deducted, so there is
no data corruption, which is why this is High rather than Critical.

Root cause is architectural and worth a deliberate decision: `buy_item_from_station_module` calls
`send_direct_server_warning` (a table insert) and then `return Err(...)` in the same reducer. **Reducers are
transactional** (per CLAUDE.md core concept #1), so the `Err` rolls the warning row back before it ever reaches
the client. The issue author counts **15 send-then-`Err` sites across 4 files** with the same bug:

| Count | File |
|---|---|
| 7 | `server/src/logic/stations/buy_and_sell.rs` |
| 6 | `server/src/logic/stations/contribution.rs` |
| 1 | `server/src/logic/ships/creation.rs` |
| 1 | `server/src/logic/ships/station_interactions.rs` |

Two more layers compound it: the client treats the reducer call returning `Ok` as "purchase succeeded" (the
generated binding only errors if the *send* fails, not the reducer), and the buy-total UI never turns red when it
exceeds the player's credits. The convention to settle — **a reducer commits a DM *or* returns `Err`, not both** —
should be documented so the next reducer doesn't reintroduce the pattern.

**Suggested area:** server-side (the rollback fix + convention) and client-side (success feedback + red total).
The red-total fix is independent and shippable on its own.

### #242 — Asteroids broadcast exact resource amount/type
**Priority: High.** The author flags this explicitly as a credibility blocker ("needs to be changed before anyone
takes this game seriously") — a hacked client can read every asteroid's exact ore type and amount because the
`Asteroid` table is `public` with all fields. This is the **same anti-cheat problem as #84 and #177**.

**Duplicate flag — consolidate, don't implement in isolation.** #177 ("Private entity tables via two-tier views")
already proposes exactly this pattern generalized across all entity tables (asteroid included): a coarse
*sector-public* view plus a detailed *targeted/scanned* view. #242's `Asteroid` (coarse `resourcesLeft` 0.0–1.0) +
private `AsteroidDetails` + per-player `AsteroidScanDetails` is the asteroid-specific instance of #177's model, and
#84 is the triaged umbrella ticket. **Recommendation:** treat #242 as the asteroid slice of #177/#84 (**Move →
M7**), and cross-link all three so the view mechanism is designed once.

The issue's "bonus goals" (multi-resource asteroids, mining heat → explosion, a six-tier scanner progression,
sector-wide scan boosters) are a **gameplay-expansion wishlist**, not anti-cheat — they should be split into
separate `future-vision` tickets per the author's own "create followup tickets" note, so the security fix isn't
blocked on designing a scanner economy.

### #152 — Web build: compile the client to WASM for browser play
**Priority: High (keep no-milestone, M6 enabler).** Called out because it directly serves M6's single success
criterion — *"someone other than the designer wants to log in tomorrow."* The author's feasibility pass finds the
macroquad / egui-macroquad / GLSL-ES stack already essentially web-ready, so this converts "download an unsigned
binary and bypass OS warnings" into "click a link" — the biggest barrier to recruiting smoke-testers. Desktop/laptop
browsers only; mobile explicitly out of scope. Not milestone-gating, but the highest-leverage non-bug item in this
batch. **Area:** client-side.

---

## Notes

- **No action taken on GitHub.** Labels, milestones, and assignments are unchanged. The dispositions above are
  recommendations; applying them is step 8–10 of `issue-disposition.md`'s activation checklist.
- **Milestones:** every open issue currently shows *no milestone*, including issues `issue-disposition.md` expected
  to be milestoned (M1–M7). Either the activation checklist hasn't been run or the milestones were reset — worth a
  human glance before relying on milestone queries.
- Only GitHub is connected as a tracker for this session's scope (`galaxycr8r/solarance-beginnings`).
