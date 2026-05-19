---
slug: 003-rotational-velocity
date: 2026-04-02
tag: DEVLOG · 003
title: Rotational velocity, finally arrested.
tags: [netcode, movement]
---

The client/server prediction bug that made ships spin gently forever is fixed. Took two weekends.

It's narrow on purpose — the rest of the movement system is "done enough for MVP" and I'm not touching it until pilots tell me otherwise. The two-player shared-building spike is no longer blocked.

If you've ever tried to write client-side prediction for angular velocity in a deterministic netcode setup, you know exactly the kind of headache this was. The fix in the end was small. Finding it was not.
