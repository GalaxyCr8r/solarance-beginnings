# Overview

Here be dragons.

## Core Loops

- `Station` Module Loop
- `StellarObject` Transform Loop
-

```mermaid
---
title: Flow of Control
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
