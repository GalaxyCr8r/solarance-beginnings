---
slug: 005-the-bar-moved
date: 2026-05-20
tag: DEVLOG · 005
title: The bar moved.
tags: [m1, spike, contribution]
---

The shared-building spike works. One station, two pilots, one progress bar that ticks up when either of us drops cargo into it.

It's not pretty. Nothing flashes when it completes. The construction site is a single seeded outpost in Alpha called "Alpha Outpost" that needs 100 iron and 50 silicon. You can fly to it, press B, watch the bar, deposit, watch the bar move. That's the whole loop.

## What's actually in

- A `contribute_to_station` reducer with a 300px range gate. Get close, deposit, the bar moves.
- A construction window on the client — progress bar, per-resource breakdown, deposit-from-cargo buttons. Greys out when you drift too far.
- An admin reducer to spawn test cargo, since mining doesn't exist yet.
- A docking gate, because docking with a half-built station looked wrong.
- "(Under Construction)" derived from state, not baked into the station name. Once the bar hits 100% the suffix disappears everywhere.

## What's next

Persistence. The welcome-back screen. Log out, come back tomorrow, see what changed while you were gone — both your own progress and the other pilot's. That's M2.

No mining yet. No second sector with its own construction site. No completion fanfare. Those are later. The core question this spike was supposed to answer — *does watching the bar move together actually feel like anything* — is the part I can't write down. That's for the people who flew up to the outpost with me.
