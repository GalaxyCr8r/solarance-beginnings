---
status: accepted
date: 2026-06-09
issue: #101
---

# Split messaging into Channel Messages + Direct Server Messages

## Context

Issue #101 asked us to add `Scope` (Personal / Faction / System) and `Priority` columns to the existing `ServerMessage` + `ServerMessageRecipient` pair. The pre-existing system had two parallel, poorly-aligned mechanisms:

- **Chat tables** (`GlobalChatMessage` / `SectorChatMessage` / `FactionChatMessage`) — separate tables per audience, sender is always a `Player`, no read state.
- **Server messages** (`ServerMessage` + `ServerMessageRecipient`) — fan-out via a recipient join table, no sender identity (just a `message_type` enum + free-text `sender_context`), per-recipient `read_at` / `delivered_at`.

The two systems answered the same question — "who should see this message?" — in incompatible ways. The proposed scope/priority columns would have layered a *third* taxonomy on top.

## Decision

Replace both systems with **two coherent ones**, six tables total:

- **Channel Messages** — 1-to-many, fire-and-forget, no read state.
  - `ServerChannelMessage` (public, no view) — official MOTD; sender is implicitly the Server.
  - `GalaxyChannelMessage` (private, view-gated) — galaxy-wide player chat.
  - `StarSystemChannelMessage` (private, view) — scoped to one StarSystem.
  - `SectorChannelMessage` (private, view) — scoped to one Sector.
  - `FactionChannelMessage` (private, view) — scoped to one Faction.
- **Direct Server Message** — `DirectServerMessage` (private, view), 1-to-1, server→player, async inbox.

Cross-cutting rules:

- Scope is "which table you posted to" — no `Scope` enum column.
- Read state is login-relative: a message is unread if `created_at > player.last_login`. Cleared when the player next sends a chat. **No** `read_at` columns server-side.
- `MessageSender { Player(Identity), System }` lives only on channel tables; `DirectServerMessage` has no sender field (server is the implicit sender).
- `MessageSeverity { Info, Warning, Critical }` is for `DirectServerMessage` only. `Error` is deliberately omitted — synchronous validation errors stay as reducer `Err(String)` returns. The inbox is for **durable, async** events the player needs to learn about after returning.
- All private tables are exposed exclusively through STDB Views (`#[spacetimedb::view]`).

## Why this shape

1. **The original "fan-out with per-recipient read state" justified its complexity by tracking exactly-once delivery to each player.** Once we accepted login-relative read state instead (cheap, client-derived), the `ServerMessageRecipient` join table dissolved — both server-to-many cases now route to channels (no read state needed) and server-to-one cases get a flat `to:` column.
2. **Server vs Galaxy is two tables, not one with a discriminator.** Server is truly public (banned / not-logged-in players still see MOTD; can live on a website). Galaxy is gated to logged-in players (banned shouldn't read game chat). Splitting also future-proofs multi-system: once StarSystem #2 ships, `StarSystem` chat scopes correctly without any Galaxy-chat refactor.
3. **Severity, not category.** `Admin` and `System` from the old `ServerMessageType` enum were sender-axis terms confused with severity-axis. With sender implicit on DMs, the remaining axis is purely severity — and three tiers (Info / Warning / Critical) forces real distinction at write time.

## The Galaxy view's constant-`galaxy_id` column

STDB Views may not perform full table scans — every View must filter on an **indexed** column. StarSystem / Sector / Faction / DM each have a natural indexed FK; **Galaxy does not** (its audience is "all logged-in players"). To keep the view legal without compromising the design, `GalaxyChannelMessage` carries an indexed `galaxy_id: u32` column whose value is always `0` in MVP. The view filters `galaxy_id == 0` (legal indexed filter), then branches in-body on `is logged-in?` (banned / not-registered → empty result).

This is a deliberate exception we accepted only after rejecting two alternatives:

- **Windowing the Galaxy view** (`created_at` range filter) — gave a more honest bounded read set, but the user ruled out server-side windowing across all channels for consistency.
- **Treating Galaxy as a public table like Server** — rejected: banned and lapsed-supporter identities would gain access to all galaxy chat.

The constant-column read set is the whole table at MVP scale (<10 players, low chat volume), which is fine. The column also stops being purely artificial post-MVP if multiple galaxies are ever modeled.

## Considered and rejected

- **Add `Scope` + `Priority` columns to the existing `ServerMessage` row.** What the issue literally asked for; rejected because it overlapped with the existing `ServerMessageType::System` variant and didn't address the fan-out / read-state mismatch underneath.
- **Allow player→player DMs in MVP.** Schema would have supported it via `MessageSender::Player`, but no moderation surface exists (no mute / block), so the structural enum stays in the channel tables only; `DirectServerMessage` is named/shaped for *server*-only senders. A future `DirectMessage` (player↔player) would be a separate table.
- **Single `Message` table with a `Scope` column.** STDB's view-legality rule made the natural unified shape (nullable `sector_id` / `system_id` / `faction_id` per row) structurally illegal — a View can't filter on `Option<>`. The split-into-tables design fell out of this constraint.

## Consequences

- **Breaking schema change** — first deploy needs `task server:publish-clear`.
- **No durable inbox for synchronous reducer errors.** "Username taken" no longer leaves an artifact; it is now exclusively a reducer `Err` surfaced via `_then`. This is intentional but means clients must use `_then` (or the equivalent) for action feedback.
- **Per-caller views are O(subscribers).** Five of the six tables (every one with a view) are `ViewContext`, so each subscribed client gets a separately-tracked materialization. Acceptable at MVP scale; a real scaling cliff if player counts grow beyond a few dozen — at which point per-sector / per-faction views may need to become `AnonymousViewContext` keyed on stable scope IDs.
- **First use of STDB Views + private tables in this codebase.** Every previous table is plain `public`. The messaging refactor is also the pilot for the View+RLS pattern; expect future sensitive tables (player credits, inventory) to migrate similarly.
