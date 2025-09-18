# Requirements Document

## Introduction

This feature implements basic NPC behaviors for the Solarance space MMO, adding three distinct types of autonomous NPCs that will enhance gameplay through dynamic economic activity, security presence, and combat encounters. The NPCs will operate independently using SpacetimeDB's scheduled reducer system to create a living, breathing universe where players interact with AI-driven entities that have their own goals and behaviors.

## Requirements

### Requirement 1

**User Story:** As a player, I want to encounter Trader NPCs that buy and sell items at stations, so that I can participate in a dynamic economy by both AI-driven and player-driven market activity.

#### Acceptance Criteria

1. WHEN a Trader NPC is spawned THEN the system SHALL assign it a home station, a list of one or more target stations to trade with, and items to sell at the first station
2. WHEN a Trader NPC has cargo to sell THEN it SHALL sell items at stations where they are needed
3. WHEN a Trader NPC reaches a station THEN it SHALL attempt to buy items that are in demand at its next destination station
4. WHEN a Trader NPC completes its trading route THEN it SHALL return to its home station
5. WHEN a Trader NPC's cargo hold is full THEN it SHALL prioritize selling over buying
6. WHEN a Trader NPC encounters hostile ships THEN it SHALL attempt to flee to the nearest friendly station or home if that's closer.

### Requirement 2

**User Story:** As a player, I want to see Patrol NPCs moving between sectors, so that I experience a sense of faction presence and security in controlled space.

#### Acceptance Criteria

1. WHEN a Patrol NPC is spawned THEN the system SHALL assign it a home station and a patrol route of 2-3 sectors
2. WHEN a Patrol NPC completes its patrol route THEN it SHALL return to its home station
3. WHEN a Patrol NPC encounters enemy faction ships in its patrol area THEN it SHALL engage in combat
4. WHEN a Patrol NPC's health drops below 25% THEN it SHALL retreat to its home station
5. WHEN a Patrol NPC reaches its home station while damaged THEN it is despawned and a replacement SHALL be spawned after a cooldown period
6. WHEN a Patrol NPC is destroyed THEN a replacement SHALL be spawned after a cooldown period

### Requirement 3

**User Story:** As a player, I want to encounter Raider NPCs that attack enemy faction ships, so that I experience dynamic PvE combat and faction warfare. This look like LC sending ships to fight back against the VCN, or the FTU sending raiders into IWA space.

#### Acceptance Criteria

1. WHEN a Raider NPC is spawned THEN the system SHALL assign it a home station and identify nearby enemy-controlled sectors
2. WHEN a Raider NPC enters an enemy sector THEN it SHALL seek out and attack ships belonging to hostile factions
3. WHEN a Raider NPC's health drops below 30% THEN it SHALL retreat to its home station
4. WHEN a Raider NPC destroys an enemy ship THEN it SHALL attempt to loot valuable cargo
5. WHEN a Raider NPC cannot find targets in a sector THEN it SHALL move to another enemy sector or return home
6. WHEN a Raider NPC reaches its home station while damaged THEN it is despawned and a replacement SHALL be spawned after a cooldown period
7. WHEN a Raider NPC is destroyed THEN a replacement SHALL be spawned after a cooldown period

### Requirement 4

**User Story:** As a game administrator, I want to configure NPC spawn rates and behaviors, so that I can balance gameplay and server performance. There is currently a `GlobalConfig` singleton-struct, there should be a corresponding `GlobalNpcConfig` singleton-struct that allows for real-time tweaking of NPCs.

#### Acceptance Criteria

1. WHEN configuring NPCs THEN the system SHALL allow setting maximum NPC counts per faction
2. WHEN an NPC is destroyed THEN the system SHALL respect configured respawn cooldown periods in `GlobalNpcConfig`
3. WHEN NPCs are active THEN the system SHALL reference a `GlobalNpcConfig` field to limit their actions to prevent server overload
4. WHEN NPCs perform actions THEN the system SHALL log important events for monitoring into a dedicated table for it
5. WHEN NPC populations exceed limits THEN the system SHALL prevent spawning additional NPCs based on `GlobalNpcConfig` settings

### Requirement 5

**User Story:** As a player, I want NPCs to interact realistically with the game world, so that they feel like genuine participants rather than scripted entities. Just like players have a `PlayerController` struct, NPCs should have a `NpcShipController` struct to manage their ships' movement at a slower pace than actual players to conserve system resources

#### Acceptance Criteria

1. WHEN NPCs move between sectors THEN they SHALL use the same movement mechanics as players
2. WHEN NPCs engage in combat THEN they SHALL use appropriate weapons and tactics for their ship type (once combat is implemented)
3. WHEN NPCs dock at stations THEN they SHALL follow the same docking procedures as players
4. WHEN NPCs carry cargo THEN it SHALL be visible and lootable if the NPC is destroyed
