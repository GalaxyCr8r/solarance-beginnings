---
inclusion: always
---

# SpacetimeDB Solarance Project Guidelines

## Project Overview

Solarance: Beginnings is a 2D top-down space MMO built with Rust, using SpacetimeDB for the backend and Macroquad for the client. The project follows a client-server architecture with real-time multiplayer synchronization.

## Architecture Patterns

### SpacetimeDB Module Structure

- **Server**: Located in `server/src/`, implements SpacetimeDB reducers and tables
- **Client**: Located in `client/src/`, uses `spacetimedb-sdk` for database connectivity
- **Module Organization**: Each domain (ships, stations, sectors, etc.) has its own module with:
  - `types.rs` - Core data structures with `#[spacetimedb::table]` attributes
  - `reducers.rs` - Functions marked with `#[spacetimedb::reducer]`
  - `timers.rs` - Scheduled functions using `#[spacetimedb::table(scheduled(...))]`
  - `rls.rs` - Row-level security rules (currently commented out)
  - `impls.rs` - Implementation blocks for structs

### Code Style Conventions

#### SpacetimeDB Patterns

- Use `#[spacetimedb::table]` for persistent data structures
- Mark reducer functions with `#[spacetimedb::reducer]`
- Use `ReducerContext` parameter for all reducers: `ctx: &ReducerContext`
- Implement scheduled functions with `#[spacetimedb::table(scheduled(function_name))]`
- Use `spacetimedb::ScheduleAt::Interval()` for recurring timers
- Access database through `spacetimedsl::dsl(ctx)` wrapper

#### Error Handling

- Reducers return `Result<(), String>` for error handling
- Use `?` operator for error propagation
- Log important events with `spacetimedb::log::info!()`

#### Data Access Patterns

- Use DSL wrapper functions for database operations: `dsl.create_*()`, `dsl.get_*()`, `dsl.update_*()`
- Implement `#[primary_key]` and `#[unique]` attributes appropriately
- Use `#[index(btree)]` for frequently queried fields

### Client-Side Patterns

#### Game Architecture

- Main game loop in `client/src/main.rs` with async/await pattern
- Asset loading using Macroquad's texture system
- UI built with egui-macroquad integration
- Game state managed through Macroquad's storage system

#### SpacetimeDB Client Integration

- Connect using `spacetimedb_sdk::DbContext`
- Handle authentication via Auth0 integration
- Use module bindings for type-safe database access

## Development Guidelines

### File Organization

- Keep domain logic separated into distinct modules
- Use consistent naming: `snake_case` for files, `PascalCase` for types
- Place utility functions in dedicated `utility.rs` files
- Separate initialization logic into `init()` functions

### Performance Considerations

- Use scheduled reducers for background processing
- Implement proper indexing on frequently queried fields
- Batch database operations when possible
- Use timers for delayed operations rather than polling

### Security & Access Control

- Implement server-side validation in reducers
- Use `try_server_only(ctx)?` for server-only operations
- Plan for row-level security implementation (currently disabled)

### Asset Management

- Store assets in `client/assets/` directory
- Handle cross-platform asset loading (especially macOS app bundles)
- Use `.env` files for configuration management

## Common Patterns to Follow

### Creating New Entities

```rust
#[spacetimedb::reducer]
pub fn create_entity(ctx: &ReducerContext, params: Type) -> Result<(), String> {
    let dsl = dsl(ctx);
    let entity = dsl.create_entity(params)?;
    spacetimedb::log::info!("Created entity #{}", entity.id);
    Ok(())
}
```

### Timer Implementation

```rust
#[spacetimedb::table(name = entity_timer, scheduled(timer_function))]
pub struct EntityTimer {
    #[primary_key]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
    // additional fields
}
```

### Client Connection Handling

- Always check for valid identity before operations
- Handle connection timeouts gracefully
- Implement proper error messaging for connection failures

## Testing & Debugging

- Use `spacetimedb::log::info!()` for server-side logging
- Implement debug UI panels for development
- Test with multiple clients for multiplayer scenarios
- Use Taskfile.yml for build automation
