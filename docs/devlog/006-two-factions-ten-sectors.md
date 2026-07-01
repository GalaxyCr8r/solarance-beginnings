---
slug: 006-two-factions-ten-sectors
date: 2026-06-30
tag: DEVLOG · 006
title: Two factions, ten sectors, one real map.
tags: [m3, m4, factions, map, messaging]
---

June was the month M3 actually closed, and I didn't stop to celebrate before M4 was already half-seeded.

## What's actually in

**Messaging got untangled first.** The old chat/server-message setup was two systems answering the same question — who should see this? — in incompatible ways. Replaced both with six tables: five Channel Messages (Server, Galaxy, StarSystem, Sector, Faction — pick your audience) and one Direct Server Message inbox for things the server needs to tell one player later, like the welcome-back summary. Net -947 lines. Wrote the reasoning up as ADR-0001 if you want more than the diff.

**M3 closed.** New players now pick Lrak Combine or Rediar Federation — the only two factions with a Capital station, so the picker greys out anything that isn't real yet — and spawn at their faction's capital in a single Column corvette, one ship per player, enforced server-side. Construction sites tint by owning faction now too, so you can tell whose site you're looking at without reading a tooltip. Chasing a bug where the welcome-back panel vanished for docked players (#149) turned into ADR-0002: docking doesn't "pause" a ship, it structurally removes it from the sector simulation. Piloted Ship vs. Docked Ship is vocabulary now, not just behavior.

**M4 started before M3's paint dried.** Procyon is ten sectors now instead of four — two faction capitals bracketing a neutral middle, connected by a real hub-and-spoke jumpgate network, five sectors minable. Built a small admin-only client (`client-admin`) to seed and inspect all of that without hand-writing SQL, and it already does more than seeding — this week it grew a read-only galaxy overview: players, ships, and construction sites at a glance. The in-game map got jumpgate lines, always-on labels, and auto-fit so the whole network is legible instead of an arbitrary crop.

**The website map stopped being a mock.** It now queries SpacetimeDB's public HTTP /sql endpoint directly — no SDK, no build step — so what you see on the site is the actual galaxy, not a screenshot pretending to be one.

**Small stuff that mattered:** asteroid fields now have per-sector ore mixes instead of one global table (Karren's Reach finally has Carbon, which the old global table never spawned); killed a 100ms combat-cooldown timer that turned out to be the single biggest CPU cost on the live maincloud instance, for a system that isn't even in scope yet; fixed a jumpgate-transit flicker where ships from the wrong sector would ghost into view for a frame.

## What's next

M4's exit gate: jumpgate connectivity verified across all ten sectors, decorative nebulae, and the admin client covering the rest of the seed/inspect workflow. After that I want actual bodies in more than one sector at once — the M1 question, whether watching the bar move together feels like anything, still hasn't been answered with more than two people in the same place.
