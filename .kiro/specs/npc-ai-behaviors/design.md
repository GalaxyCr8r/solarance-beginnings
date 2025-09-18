# NPC AI Behaviors Design Document

## Overview

This design implements three distinct NPC AI behaviors (Traders, Patrol, and Raiders) for the Solarance space MMO using SpacetimeDB's scheduled reducer system. The design leverages the existing ship, station, sector, and faction infrastructure while adding new AI state management and behavior scheduling systems.

The implementation follows SpacetimeDB patterns with persistent state storage, scheduled reducers for autonomous behavior, and integration with existing game systems like combat, trading, and movement.

## Architecture

### Core Components

1. **NPC Behavior State Machine**: Manages current AI state and transitions
2. **Behavior Schedulers**: SpacetimeDB scheduled reducers that execute NPC actions
3. **Pathfinding System**: Calculates routes between sectors and stations
4. **Decision Engine**: Evaluates conditions and chooses appropriate actions
5. **Integration Layer**: Interfaces with existing ship, station, and combat systems

### Data Flow

```
NPC Spawn → Behavior Assignment → State Initialization → Scheduled Execution
     ↓                                                           ↑
Behavior Timer → Decision Engine → Action Execution → State Update
```

## Components and Interfaces

### NPC Behavior State Management

#### NpcBehaviorType Enum

```rust
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum NpcBehaviorType {
    Trader {
        home_station_id: u64,
        trade_stations: Vec<u64>,
        current_route_index: usize,
    },
    Patrol {
        home_station_id: u64,
        patrol_sectors: Vec<u64>,
        current_patrol_index: usize,
    },
    Raider {
        home_station_id: u64,
        target_sectors: Vec<u64>,
        current_target_index: usize,
    },
}
```

#### NpcBehaviorState Enum

```rust
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq)]
pub enum NpcBehaviorState {
    // Common states
    Idle,
    MovingToSector(u64),
    DockingAtStation(u64),
    Docked(u64),

    // Trader-specific states
    Trading,
    BuyingCargo,
    SellingCargo,

    // Patrol-specific states
    Patrolling,
    Investigating,

    // Raider-specific states
    Hunting,
    Engaging(u64), // target ship_id
    Looting,
    Retreating,
}
```

### Database Tables

#### NpcInstance Table

```rust
#[dsl(plural_name = npc_instances)]
#[table(name = npc_instance, public)]
pub struct NpcInstance {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(path = ShipId)]
    #[foreign_key(path = crate::types::ships, table = ship, column = id, on_delete = Cascade)]
    pub ship_id: u64,

    #[index(btree)]
    #[use_wrapper(path = FactionId)]
    #[foreign_key(path = crate::types::factions, table = faction, column = id, on_delete = Error)]
    pub faction_id: u32,

    pub behavior_type: NpcBehaviorType,
    pub current_state: NpcBehaviorState,
    pub health_threshold: f32, // Retreat when health drops below this
    pub last_action_time: Timestamp,
    pub respawn_cooldown: Option<Timestamp>,
}
```

#### Enhanced NpcBehaviorSchedule Table

```rust
#[dsl(plural_name = npc_behavior_schedules)]
#[table(name = npc_behavior_schedule, scheduled(process_npc_behavior_tick))]
pub struct NpcBehaviorSchedule {
    #[primary_key]
    #[create_wrapper]
    #[use_wrapper(path = NpcInstanceId)]
    #[foreign_key(path = crate::types::npcs, table = npc_instance, column = id, on_delete = Cascade)]
    id: u64,

    pub scheduled_at: ScheduleAt,
    pub behavior_type: NpcBehaviorType, // Cached for performance
}
```

### Behavior Implementation

#### Trader Behavior Logic

- **Route Planning**: Calculate optimal trading routes based on station supply/demand
- **Cargo Management**: Buy low-demand items, sell high-demand items
- **Economic Integration**: Use existing station trading systems
- **Risk Assessment**: Flee from hostile encounters

#### Patrol Behavior Logic

- **Route Execution**: Follow predefined patrol routes between sectors
- **Threat Response**: Engage enemy faction ships in patrol area
- **Health Management**: Retreat and repair when damaged
- **Persistence**: Resume patrol after repairs

#### Raider Behavior Logic

- **Target Acquisition**: Identify enemy faction ships in target sectors
- **Combat Engagement**: Attack using ship's equipped weapons
- **Loot Collection**: Gather cargo from destroyed ships
- **Tactical Retreat**: Return home when health is low

### Integration Points

#### Ship System Integration

- Leverage existing `Ship` and `ShipStatus` tables
- Use `ShipLocation` enum for docking/undocking
- Integrate with ship movement and combat systems
- Utilize ship cargo and equipment systems

#### Station System Integration

- Use existing station docking mechanisms
- Integrate with station trading and inventory systems
- Leverage station repair functionality
- Respect station faction ownership

#### Sector System Integration

- Use existing sector navigation
- Integrate with jumpgate travel system
- Respect sector faction control
- Handle sector-based visibility

#### Faction System Integration

- Use faction standings for friend/foe identification
- Respect faction-controlled territory
- Integrate with faction warfare mechanics

## Data Models

### NPC Configuration

```rust
#[derive(SpacetimeType, Debug, Clone)]
pub struct NpcSpawnConfig {
    pub max_npcs_per_faction: u32,
    pub trader_spawn_rate: f32,
    pub patrol_spawn_rate: f32,
    pub raider_spawn_rate: f32,
    pub respawn_cooldown_minutes: u32,
    pub behavior_tick_interval_seconds: u32,
}
```

### Pathfinding Data

```rust
#[derive(SpacetimeType, Debug, Clone)]
pub struct SectorRoute {
    pub from_sector: u64,
    pub to_sector: u64,
    pub jumpgate_ids: Vec<u64>,
    pub estimated_travel_time: u32,
}
```

## Error Handling

### Graceful Degradation

- **Missing Targets**: NPCs return home if target stations/sectors are unavailable
- **Combat Failures**: Damaged NPCs retreat and attempt repairs
- **Pathfinding Failures**: NPCs use fallback routes or wait for resolution
- **Resource Constraints**: Limit NPC actions to prevent server overload

### Error Recovery

- **State Corruption**: Reset NPC to safe state (Idle at home station)
- **Timer Failures**: Reschedule with exponential backoff
- **Database Inconsistencies**: Log errors and attempt cleanup
- **Performance Issues**: Reduce NPC activity dynamically

### Logging Strategy

- Log NPC spawns, deaths, and major state transitions
- Track performance metrics for behavior processing
- Monitor error rates and recovery attempts
- Provide debugging information for behavior tuning

## Testing Strategy

### Unit Testing

- **Behavior State Machines**: Test state transitions and edge cases
- **Decision Logic**: Verify correct action selection under various conditions
- **Integration Points**: Mock external systems for isolated testing
- **Error Handling**: Test recovery from various failure scenarios

### Integration Testing

- **Multi-NPC Scenarios**: Test interactions between different NPC types
- **Player Interaction**: Verify NPCs respond correctly to player actions
- **Performance Testing**: Ensure system scales with NPC population
- **Persistence Testing**: Verify state preservation across server restarts

### Behavior Testing

- **Trader Routes**: Verify economic behavior and route optimization
- **Patrol Coverage**: Ensure patrol routes provide adequate sector coverage
- **Raider Tactics**: Test combat engagement and retreat behaviors
- **Faction Interactions**: Verify correct friend/foe identification

### Performance Considerations

- **Batch Processing**: Group NPC updates to reduce database load
- **Intelligent Scheduling**: Vary timer intervals based on NPC activity
- **State Caching**: Cache frequently accessed data to improve performance
- **Population Limits**: Implement dynamic population control based on server load

## Implementation Phases

### Phase 1: Core Infrastructure

- Implement NPC data models and database tables
- Create basic behavior state machine
- Set up scheduled reducer framework
- Implement NPC spawning system

### Phase 2: Trader Implementation

- Implement trader behavior logic
- Integrate with station trading systems
- Add route planning and optimization
- Test economic impact and balance

### Phase 3: Patrol Implementation

- Implement patrol behavior logic
- Add sector navigation and route following
- Integrate with combat systems
- Test patrol coverage and effectiveness

### Phase 4: Raider Implementation

- Implement raider behavior logic
- Add target acquisition and combat engagement
- Implement loot collection mechanics
- Test combat balance and player interaction

### Phase 5: Polish and Optimization

- Performance optimization and tuning
- Advanced AI behaviors and decision making
- Comprehensive testing and bug fixes
- Documentation and monitoring tools
