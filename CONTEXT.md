
This document defines the initial Domain-Specific Language (DSL) for **Solarance: Beginnings**. Every developer, agent, and domain expert must use these exact terms when discussing the game, writing documentation, or naming variables/classes in the codebase. 

If a concept cannot be described using these terms, it is likely out of scope for the MVP.

## 1. The Core Loop (Verbs)
These are the primary actions a player can take. Do not use synonyms (e.g., use `Contribute`, not `Donate` or `Give`).

*   **Find:** The act of navigating to a hand-placed resource node. *(Code: `Location`, `ResourceSource`)*
*   **Extract:** The interaction of pulling raw resources from a source into the ship's inventory. Do not use "mine" as a system noun to avoid confusion with future minigame mechanics. *(Code: `ExtractAction`, `ExtractionYield`)*
*   **Haul:** The state of transporting extracted resources in a ship's inventory across space. *(Code: `HaulingState`, `ShipInventory`)*
*   **Contribute:** The critical transfer of hauled resources from a Player's Ship to a Station's `ContributionPool`. This is the primary driver of progression. *(Code: `ContributeResource()`, `ContributionEvent`)*
*   **Grow / Update:** The automatic server-side reaction when a `ContributionPool` meets its threshold, updating the station visually and mechanically for all players.

## 2. World Architecture (Nouns)
Strict hierarchical containers for the game world.

*   **Solar System:** The highest-level map container. There is only one in the MVP.
*   **Sector:** A discrete, hand-placed area within the Solar System. It has a singular, defined purpose (e.g., "Asteroid Sector"). There is no coordinate space outside of Sectors.
*   **Jumpgate:** The only method of moving between Sectors. A point-to-point teleportation entity.
*   **Station:** A persistent, player-upgraded structure. **Rule:** Exactly one per Sector, maximum. 
*   **Module:** A specialized component (Storage, Refinery, Production, Assembly, Research, Repair, Defense) added to a Station. Stations are simply "warehouses with Modules."
*   **Capital Station:** The single, massive root station for a specific Faction where new players spawn.
*   **Resource Source:** A generic, hand-placed point of interest in a sector where players perform the `Extract` action (e.g., an asteroid). 

## 3. Player & Entities (Nouns)
*   **Player / Client:** The human interacting with the game. Designed around "David" (intermittent play, requires pause-able progression).
*   **Ship:** The player's avatar. In MVP, this is strictly a jack-of-all-trades **Corvette**. 
*   **Faction:** A team identifier. In MVP, strictly limited to a string name and a color (`Lrak Combine` / Red, `Rediar Federation` / Blue). Determines which Stations a player can `Contribute` to.
*   **Contribution Pool:** The required list of resources a Station or Module needs to reach the next growth stage. 
*   **Welcome-Back Summary:** A text-only data payload delivered to the client upon login, detailing offline ticks, station progress, and personal asset state. *(Code: `WelcomeBackPayload`, `OfflineTickCalculator`)*

## 4. Systems & Architecture
*   **Social Convergence:** The design philosophy and network event of multiple players observing a shared `ContributionPool` update simultaneously. 
*   **Offline Pause:** The rule that personal asset generation ceases when a player disconnects.
*   **Worker Process:** The single server instance managing the MVP. (Do not use "Orchestrator" or "Server Mesh" for MVP tasks).

## 5. Anti-Concepts (Banned Terminology)
To prevent scope creep, these terms are explicitly banned from MVP code, PRs, and design discussions. If you see them, flag them for the `Future Vision` backlog:

*   🚫 **Combat / Attack / Health (for ships) / Weapons** -> (Exterminate pillar is absent).
*   🚫 **Markets / Economy / Buy / Sell / Trade (between players)** -> (Exchange collapses into "Haul" and "Contribute").
*   🚫 **Explore / Procedural / Heatmap / Scan / Wormhole** -> (Exploration is stubbed; Sectors are hand-placed).
*   🚫 **Orgs / Guilds / Squads** -> (Only Factions exist).
*   🚫 **AI / NPCs / Pirates / Civilians** -> (No AI actors exist in MVP).
*   🚫 **Offline Income / Passive Generation** -> (Violates Offline Pause).
*   🚫 **Orchestrator / Handoff** -> (Over-engineering for <10 concurrent players).

---

### How to use this document
- **When writing Code:** Name your classes strictly after these terms. E.g., `class ContributionPool`, `struct WelcomeBackSummary`, `fn extract_resource()`.
- **When writing Tickets/Commits:** Use the verbs defined here. E.g., *"Fix bug where Haul state drops inventory on Jumpgate use."*
- **When talking to AI / Agents:** Reference these terms directly. E.g., *"Write a function that calculates the Welcome-Back Summary based on the Offline Pause rules."*