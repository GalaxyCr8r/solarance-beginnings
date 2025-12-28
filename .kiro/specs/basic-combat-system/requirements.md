# Requirements Document

## Introduction

The basic combat system enables players to engage in real-time combat using their ships' weapons. This system implements hitscan weapons that instantly apply damage to targeted stellar objects, visual effects for combat feedback, and energy management for weapon usage. The system is designed to be computationally efficient and prepares the foundation for future NPC combat integration.

## Requirements

### Requirement 1

**User Story:** As a player, I want to fire my ship's weapons at targeted objects, so that I can engage in combat and destroy enemy targets.

#### Acceptance Criteria

1. WHEN a player sets `PlayerShipController.fire_weapons` to true AND `targetted_sobj_id` is a valid `Ship` or `Station` class THEN the system SHALL trigger an instant hitscan attack and reset `fire_weapons` to `false`
2. WHEN a hitscan attack is triggered THEN the system SHALL calculate damage based on the ship's weapon configuration
3. WHEN damage is calculated THEN the system SHALL apply damage to the target's shields first, then hull health
4. WHEN weapons are fired THEN the system SHALL consume energy from the ship's power systems
5. IF the ship has insufficient energy THEN the system SHALL prevent weapon firing
6. WHEN a target's hull health reaches zero THEN the system SHALL destroy the target object

### Requirement 2

**User Story:** As a player, I want to see visual feedback when weapons are fired, so that I can understand the combat is happening and track my shots.

#### Acceptance Criteria

1. WHEN weapons are fired THEN the system SHALL create a `VisualEffect` entry in the database
2. WHEN a `VisualEffect` is created THEN the system SHALL schedule its automatic deletion after 10 milliseconds
3. WHEN the scheduled deletion timer triggers THEN the system SHALL remove the `VisualEffect` from the database
4. WHEN a `VisualEffect` is created THEN the client SHALL begin a firing effect for a configurable duration

### Requirement 3

**User Story:** As a developer, I want the combat system to be computationally efficient, so that the server can handle many simultaneous combat actions without performance degradation.

#### Acceptance Criteria

1. WHEN implementing hitscan weapons THEN the system SHALL NOT create stellar objects for regular weapon projectiles
2. WHEN processing weapon fire THEN the system SHALL use direct damage calculation without physics simulation
3. WHEN multiple players fire simultaneously THEN the system SHALL handle all actions within a single server tick
4. WHEN visual effects are created THEN the system SHALL use minimal data structures to reduce network overhead

### Requirement 4

**User Story:** As a developer, I want to prepare for NPC combat integration, so that NPCs can use the same combat mechanics as players in future updates.

#### Acceptance Criteria

1. WHEN designing combat reducers THEN the system SHALL accept both player and NPC ship controllers as input
2. WHEN implementing damage calculation THEN the system SHALL use generic ship data rather than player-specific data
3. WHEN creating the combat system THEN the system SHALL include placeholder structures for `NpcShipController`
4. WHEN NPC controllers are implemented THEN they SHALL use the same weapon firing mechanisms as player controllers

### Requirement 5

**User Story:** As a player, I want missile weapons to be distinguished from regular weapons, so that I understand different weapon types have different behaviors.

#### Acceptance Criteria

1. WHEN `PlayerShipController.fire_missiles` is set THEN the system SHALL recognize this as a different weapon type and reset `fire_missiles` to `false`
2. WHEN missile firing is triggered THEN the system SHALL prepare for future stellar object creation (not implemented in this phase)
3. WHEN regular weapons are fired THEN the system SHALL NOT create stellar objects
4. WHEN the combat system is designed THEN it SHALL accommodate, hitscan, projectile, and area-of-effect weapon types

### Requirement 6

**User Story:** As a player, I want my ship's energy to be consumed when firing weapons, so that there are tactical considerations around energy management during combat.

#### Acceptance Criteria

1. WHEN weapons are fired THEN the system SHALL calculate energy cost based on weapon configuration
2. WHEN energy cost is calculated THEN the system SHALL deduct energy from the ship's current energy reserves
3. IF insufficient energy is available THEN the system SHALL prevent weapon firing and provide feedback
4. WHEN energy is consumed THEN the updated energy levels SHALL be synchronized to all clients
5. WHEN energy reaches zero THEN the system SHALL disable all energy-dependent systems including weapons

### Requirement 7

**User Story:** As a player, I want to switch between combat and utility modes. You shouldn't be able to fire when attempting to dock or opening your cargo bay.

#### Acceptance Criteria

1. WHEN the player presses "Q" THEN the UI state SHALL flip between combat/utility modes
2. WHEN in combat mode THEN pressing spacebar SHALL set `PlayerShipController.fire_weapons` to `true`
3. WHEN in combat mode THEN pressing Left Control SHALL set `PlayerShipController.fire_missiles` to `true`
4. WHEN in utility mode THEN the client UI SHALL allow the docking/mining/cargo bay should become operable
