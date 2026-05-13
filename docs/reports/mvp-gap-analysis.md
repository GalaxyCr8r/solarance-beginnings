# MVP Gap Analysis — Solarance: Beginnings

*Reviewed against `docs/Solarance_Beginnings_MVP_Design_Doc.md`. Server implementation reviewed in `server/src/`.*

*Date: 2026-04-30*

---

## Section 1 — Already Matching MVP

Features that are implemented and aligned with what the MVP calls for.

| Feature | Evidence in `server/src/` |
|---|---|
| **Mining / extraction loop** | `ship_mining_timer_reducer` — 3-second scheduled tick extracts ore from `Asteroid` table (resource type + quantity) |
| **Ship cargo holds** | `ShipCargoItem` table with volume-based stacking (compact / loose / large / massive sizes) |
| **Sector travel via jump gates** | `JumpGate` table, `create_jumpgate_in_sector` reducer, automatic orientation (N/S/E/W) |
| **Station persistence** | `Station` table with size enum (Capital → Satellite); data survives server restarts |
| **Station construction tracking** | `StationUnderConstruction` table exists with progress tracking |
| **Player persistence** | `Player` table — credits, faction affiliation, login state all persist between sessions |
| **Lrak Combine + Rediar Federation factions** | Both seeded in `init`; faction definitions include name, color, and capital station association |
| **Chat system** | `GlobalChatMessage`, `SectorChatMessage`, `FactionChatMessage` tables cover all three required scopes |
| **Server notifications** | `ServerMessage` + `ServerMessageRecipient` tables provide the infrastructure for the welcome-back screen |
| **One solar system** | `StarSystem` table seeded with Procyon (spectral G, main sequence) |
| **Named sectors with jump gates** | Alpha (asteroid field), Beta (dense asteroids / refinery), Gamma (homeworld / capital) seeded and gate-connected |

---

## Section 2 — Needs Tweaks

These features exist in some form but are misaligned with the MVP spec and need targeted work before the shared-building spike.

| Feature | Current State | Required Adjustment |
|---|---|---|
| **Station contribution mechanic** | `StationUnderConstruction` table exists but there is no confirmed player-callable reducer to deposit cargo and advance construction | Implement (or verify) a `contribute_to_station` reducer: transfer items from `ShipCargoItem` into a construction pool and increment the progress field. This is the **core action** of the MVP loop. |
| **World scope** | 3 sectors seeded (Alpha, Beta, Gamma) | MVP specifies 10 named, hand-designed sectors. The remaining 7 need to be designed (purpose, sector type, security level, asteroid density) and seeded in `init`. |
| **Faction scope** | 6 factions defined (Lrak Combine, Rediar Federation, IWA, FTU, Vancellan, Procyon Compact) with joinable flags | MVP scope is exactly **2 joinable factions**. Extra factions don't need to be deleted — but non-MVP factions should be marked non-joinable and hidden from player faction selection until post-MVP. |
| **Single ship per player enforcement** | Three ship type definitions exist: Phalanx (fighter), Column (shuttle/corvette), Javelin (fighter) | MVP allows **one ship per player** — the Column (the corvette/light-freighter). `create_player_controlled_ship` should enforce this: reject if the player already owns a ship, and only allow Column type. Phalanx and Javelin are post-MVP hulls. |
| **Welcome-back screen content** | `ServerMessage` infrastructure is in place | The `client_connected` lifecycle reducer should compose and send a welcome-back message with: station progress %, number of contributions since last login, and current cargo on hand. The design doc describes this as text-only — no LLM. |

---

## Section 3 — Back Burner

These features are implemented and may be technically sound, but the design doc explicitly excludes them from MVP scope. **Do not iterate on these until the core find → extract → haul → contribute loop is proven fun.**

| Feature | Why It's Post-MVP |
|---|---|
| **Full combat system** — `fire_weapons`, `process_weapon_combat_action`, `process_weapon_fire`, `VisualEffect` table, all weapon `ItemDefinition` entries, `ItemMetadata` weapon stats (damage, cooldown, lock-on angles) | Design doc: *"Exterminate (combat) — Absent. Zero combat gameplay in MVP."* |
| **Fighter ship types** — Phalanx (fighter, 3 weapon slots), Javelin (fighter, 2 weapon slots) | Only the corvette (Column) belongs in MVP. Specialized hulls are v1.0. |
| **`ShipEquipmentSlot` system** — weapon / shield / engine equipment slots | Entirely tied to combat loadouts. No combat = no need for this system at launch. |
| **Station production pipeline** — `process_station_production_tick`, refinery ore→ingot conversion, factory component production, farm food cycles, lab research cycles | MVP contribution mechanic is **deposit → progress bar moves**. Full resource transformation chains are a post-MVP economy feature. |
| **Dynamic market pricing** — supply/demand buy/sell with margin-based price adjustment | Design doc: *"Exchange collapses into 'haul ore to construction site.' No markets."* |
| **Faction standings system** — `FactionStanding` bilateral rep table, -75 (hostile) to +75 (allied) scale | MVP factions = a color + which stations you help build. No reputation mechanics until v1.0. |
| **Research system** — lab station modules, research fragment / research device items, exotic research chains | Not mentioned anywhere in the MVP section. Belongs in v1.0 faction research trees. |
| **NPC infrastructure** — `NPC` table and related scaffolding | Design doc: *"The previously planned civilian NPC AI spike is cancelled. No NPCs in MVP."* |
| **`StarSystemObject` detail** — planets, moons, nebulas, lagrange points | MVP world is hand-placed sectors connected by gates. Celestial body detail is Future Vision flavor. |

---

## Section 4 — Critical Path

The design doc's stated next task is the **shared-building spike**: two players, one partially-built station, each contributing resources, watching the station grow together in real time.

The three things that must be true before that spike is runnable:

1. **`contribute_to_station` reducer** — players need a way to move cargo from their ship to the construction pool and see progress increment (see Section 2).
2. **Client subscribed to `StationUnderConstruction`** — the progress state needs to be visible and update in real time on the client.
3. **Two players can see each other's actions** — confirm that the construction progress table is `public` and that client subscription updates render for all players in the sector.

Everything else in this report can wait until after that spike validates the core emotional beat.
