# Overview

Here be dragons.

## Core Loops

EVE Online learned 15~ years ago about the need for time dilation. It is important that
the systems of Solarance happen only after a measured amount of updates beneath it happen
so that if a prior reducer call takes longer than usual, it doesn't effect the GAME itself
but only the PERFORMANCE.

Below is an idealized view of what I see Solarance needing.

```mermaid
---
title: Flow of Control
---
flowchart TD
%% Relationships
    Input-->Movement
    Movement-->Interactions
    Interactions-->Combat
    Combat-->Economy
    Economy-->Sector
    Sector-->Faction
    Faction-->Waves

%% Descriptions
    Input(INPUT: Change Velocity. 30fps)
    Movement(MOVEMENT: Translate/Rotate. 20fps)
    Interactions(INTERACTIONS: Mine Asteroid, Use Jumpgate, etc. 5fps)
    Combat(COMBAT: Shield updates, Spawn missiles, etc. 2fps)
    Economy(ECONOMY: Update station modules, factories produce things, etc. 0.1fps)
    Sector(SECTOR: Respawn asteroids, enemies, etc. 0.001fps)
    Faction(FACTION: Send fleets. Build fleets. Begin new stations. etc. 0.00001fps)
    Waves(WAVES: Gameplay ebbs and flows, Vancellan/Raider invasions, etc. days)

```

```mermaid
---
title: Possible Flow of Control
---
flowchart LR
%% Relationships
%% >Sources
    players==>player_actions
    admins==>admin_actions
    spacetimedb==>timers
    spacetimedb==>init

%% >Internal
    init(initialization)
    init==>definitions

    definitions-.->tables
    definitions-->utilities

    reducers-->logic
    reducers-->utilities
    reducers-.->tables
    reducers-.->impls

    logic-->utilities
    logic-.->tables

    utilities-.->tables

    impls-.->tables

%% Topology
    subgraph agents
        players
        admins
        spacetimedb
    end

    subgraph reducers
        player_actions
        admin_actions
        timers
        init
    end

    subgraph game_specific
        definitions
        logic
        utilities
    end

```
