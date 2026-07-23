# Software Design Principles

> Read this first. Everything else in this file is mechanics. This is philosophy.

---

## The Mental Model

The server is built around one invariant: **state lives in tables, transitions live in reducers.**

Every bug in production manifests as "the wrong thing is in a table" or "the wrong reducer ran." That's it. If you keep that model sharp, you can diagnose anything from `spacetime logs` alone.

There are three layers and they must stay separate:

```
tables/     — What exists. Pure schema. No logic.
logic/      — What happens. Reducers + helper functions.
definitions/ — What starts as true. Seed data for init.
```

Don't let logic leak into table definitions. Don't let schema decisions be made inside reducers. When you find yourself writing a method on a table struct that calls back into the DSL — stop and ask whether that belongs in `logic/` instead.

---

## Deep Modules

This codebase is organised around **deep modules** (Ousterhout, *A Philosophy of Software Design*): each module should have a **simple interface and a complex implementation**. The interface is the public contract. The implementation is the mess you're paid to hide.

A shallow module has an interface almost as complex as its body — it adds a layer without adding abstraction. A deep module genuinely simplifies the caller's world.

**Measure depth by this ratio: how much does the caller need to know vs. how much is hidden?**

The `spacetimedsl` crate is a perfect example of a deep module. A caller writes `dsl.get_player_by_id(&id)?` and has no idea that SpacetimeDB index access, row deserialization, and error normalisation just happened. That complexity is paid once, hidden forever.

Write your logic functions the same way:

```
// Shallow — caller must orchestrate the whole operation
pub fn get_ship_type(dsl, ship_id) -> ShipTypeDefinition
pub fn get_ship_status(dsl, ship_id) -> ShipStatus
pub fn compute_new_velocity(status, type, controller) -> Vec2
pub fn save_velocity(dsl, sobj_id, velocity)

// Deep — caller states intent, not steps
pub fn apply_movement_tick(dsl, controller) -> Result<(), String>
```

If a caller has to call three of your functions in the right order to do one logical thing, your module is shallow. Pull that sequencing downward.

---

## The Debugging Contract

In production you have exactly one tool: `spacetime logs`. Design everything to make those logs useful.

**Every reducer is an audit trail entry.** Each state change has exactly one reducer that caused it. When something goes wrong in the live game, you will grep the logs for a reducer name and a row ID. That grep must find the answer.

This means:

1. **Reducers must be narrow.** One reducer = one conceptual operation. If a reducer does three unrelated things, a log line doesn't tell you which thing failed.

2. **Errors must carry context.** `"Ship not found"` is useless. `"Ship not found (likely docked), removing update timer for player ID: {id}"` is a diagnosis. Every `Err(...)` string should answer: *what* failed, *where*, and *why the code took that branch*.

3. **Timer reducers are the heartbeat.** `timer_update_all_ship_movement_controllers`, `station_production_schedule_reducer`, and their kin fire constantly. When something drifts, it drifted on a tick. Log the station ID, the sector, the ship — not just "tick completed".

4. **Helper functions propagate errors, they don't swallow them.** Use `?` and let the reducer surface the failure. A helper that returns `Ok(())` after silently catching an error breaks the audit trail.

---

## Information Hiding

Every decision you make inside a module that a caller doesn't need to know about is good engineering. Every detail that leaks across a module boundary is future maintenance debt.

**Concretely:**

- The `stations/production.rs` dispatcher (`match blueprint.get_category() { ... }`) is internal to the production tick. Callers don't know it exists. They call `process_station_production_tick`. Good.
- The `CreateShipMovementController` struct is an implementation detail of ship creation. Callers in `logic/ships/creation.rs` construct it, but nothing outside `logic/ships/` should need to know its fields. If it does, you've leaked a detail.
- `get_player_ship_and_sobj` in `players.rs` hides a two-step lookup (controller → sobj → ship). Any code that re-implements those two steps inline is a missed opportunity for this function.

Before adding a parameter to a function, ask: *is there a way to hide this inside the function instead?* Before making a type public, ask: *who outside this module actually needs to name this type?*

---

## The Three Rules for New Code

**1. Make the interface smaller than the implementation.**
If your public function count grows at the same rate as your line count, you're writing shallow modules. Group related operations and hide the sequencing.

**2. Errors are documentation.**
Every `Err(string)` will eventually be the only clue you have when the game is live and a player files a bug report at 2am. Write it for that person.

**3. The DSL is the database. Don't re-abstract it.**
Don't create helper functions that are thin wrappers around `dsl.get_x_by_id`. If the DSL already does it, call it directly. New abstractions earn their place by hiding *multi-step* operations or *domain logic*, not by renaming a single DSL call.

---

# SpacetimeDB Rules (All Languages)

> **Last updated:** 2026-05-07

## Language-Specific Rules

| Language | Rule File |
|----------|-----------|
| **TypeScript/React** | `spacetimedb-typescript.mdc` (MANDATORY) |
| **Rust** | `spacetimedb-rust.mdc` (MANDATORY) |
| **C#** | `spacetimedb-csharp.mdc` (MANDATORY) |

---

## Core Concepts

1. **Reducers are transactional** — they do not return data to callers
2. **Reducers must be deterministic** — no filesystem, network, timers, or random
3. **Read data via tables/subscriptions** — not reducer return values
4. **Auto-increment IDs are not sequential** — gaps are normal, don't use for ordering
5. **`ctx.sender()` is the authenticated principal** — never trust identity args

---

## Feature Implementation Checklist

When implementing a feature that spans backend and client:

1. **Backend:** Define table(s) to store the data
2. **Backend:** Define reducer(s) to mutate the data
3. **Client:** Subscribe to the table(s)
4. **Client:** Call the reducer(s) from UI — **don't forget this step!**
5. **Client:** Render the data from the table(s)

**Common mistake:** Building backend tables/reducers but forgetting to wire up the client to call them.

---

## Commands

```bash
# Login to allow remote database deployment e.g. to maincloud
spacetime login

# Start local SpacetimeDB
spacetime start

# Publish module
spacetime publish <db-name> --project-path <module-path>

# Clear and republish
spacetime publish <db-name> --clear-database -y --project-path <module-path>

# Generate client bindings
spacetime generate --lang <lang> --out-dir <out> --project-path <module-path>

# View logs
spacetime logs <db-name>
```

---

## Deployment

- Maincloud is the spacetimedb hosted cloud and the default location for module publishing
- The default server marked by *** in `spacetime server list` should be used when publishing
- If the default server is maincloud you should publish to maincloud
- When publishing to maincloud the database dashboard will be at the url: https://spacetimedb.com/@<username>/<database-name>
- The database owner can view utilization and performance metrics on the dashboard

---

## Debugging Checklist

1. Is SpacetimeDB server running? (`spacetime start`)
2. Is the module published? (`spacetime publish`)
3. Are client bindings generated? (`spacetime generate`)
4. Check server logs for errors (`spacetime logs <db-name>`)
5. **Is the reducer actually being called from the client?**

---

## Editing Behavior

- Make the smallest change necessary
- Do NOT touch unrelated files, configs, or dependencies
- Do NOT invent new SpacetimeDB APIs — use only what exists in docs or this repo


# SpacetimeDB Rust SDK

> **Tested with:** SpacetimeDB runtime 1.11.x, `spacetimedb` crate 1.1.x
> **Last updated:** 2026-01-14

---

## HALLUCINATED APIs — DO NOT USE

**These APIs DO NOT EXIST. LLMs frequently hallucinate them.**

```rust
// WRONG — these macros/attributes don't exist
#[spacetimedb::table]           // Use #[table] after importing
#[spacetimedb::reducer]         // Use #[reducer] after importing
#[derive(Table)]                // Tables use #[table] attribute, not derive
#[derive(Reducer)]              // Reducers use #[reducer] attribute

// WRONG — SpacetimeType on tables
#[derive(SpacetimeType)]        // DO NOT use on #[table] structs!
#[table(name = my_table)]
pub struct MyTable { ... }

// WRONG — mutable context
pub fn my_reducer(ctx: &mut ReducerContext, ...) { }  // Should be &ReducerContext

// WRONG — table access without parentheses
ctx.db.player                   // Should be ctx.db.player()
ctx.db.player.find(id)          // Should be ctx.db.player().id().find(&id)
```

### CORRECT PATTERNS:

```rust
// CORRECT IMPORTS
use spacetimedb::{table, reducer, Table, ReducerContext, Identity, Timestamp};
use spacetimedb::SpacetimeType;  // Only for custom types, NOT tables

// CORRECT TABLE — no SpacetimeType derive!
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub id: u64,
    pub name: String,
}

// CORRECT REDUCER — immutable context reference
#[reducer]
pub fn create_player(ctx: &ReducerContext, name: String) {
    ctx.db.player().insert(Player { id: 0, name });
}

// CORRECT TABLE ACCESS — methods with parentheses
let player = ctx.db.player().id().find(&player_id);
```

### DO NOT:
- **Derive `SpacetimeType` on `#[table]` structs** — the macro handles this
- **Use mutable context** — `&ReducerContext`, not `&mut ReducerContext`
- **Forget `Table` trait import** — required for table operations
- **Use field access for tables** — `ctx.db.player()` not `ctx.db.player`

---

## 1) Common Mistakes Table

### Server-side errors

| Wrong | Right | Error |
|-------|-------|-------|
| `#[derive(SpacetimeType)]` on `#[table]` | Remove it — macro handles this | Conflicting derive macros |
| `ctx.db.player` (field access) | `ctx.db.player()` (method) | "no field `player` on type" |
| `ctx.db.player().find(id)` | `ctx.db.player().id().find(&id)` | Must access via index |
| `&mut ReducerContext` | `&ReducerContext` | Wrong context type |
| Missing `use spacetimedb::Table;` | Add import | "no method named `insert`" |
| `#[table(name = "my_table")]` | `#[table(name = my_table)]` | String literals not allowed |
| Missing `public` on table | Add `public` flag | Clients can't subscribe |
| `#[spacetimedb::reducer]` | `#[reducer]` after import | Wrong attribute path |
| Network/filesystem in reducer | Use procedures instead | Sandbox violation |
| Panic for expected errors | Return `Result<(), String>` | WASM instance destroyed |

### Client-side errors

| Wrong | Right | Error |
|-------|-------|-------|
| Wrong crate name | `spacetimedb-sdk` | Dependency not found |
| Manual event loop | Use `tokio` runtime | Async issues |

---

## 2) Table Definition (CRITICAL)

**Tables use the `#[table]` attribute macro, NOT `#[derive(SpacetimeType)]`**

```rust
use spacetimedb::{table, Table, Identity, Timestamp};

// WRONG — DO NOT derive SpacetimeType on tables!
#[derive(SpacetimeType)]  // REMOVE THIS!
#[table(name = task)]
pub struct Task { ... }

// RIGHT — just the #[table] attribute
#[table(name = task, public)]
pub struct Task {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    pub owner_id: Identity,
    pub title: String,
    pub created_at: Timestamp,
}

// With indexes
#[table(name = task, public, index(name = by_owner, btree(columns = [owner_id])))]
pub struct Task {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    pub owner_id: Identity,
    pub title: String,
}
```

### Field attributes

```rust
#[primary_key]              // Exactly one per table (required)
#[auto_inc]                 // Auto-increment (integer primary keys only)
#[unique]                   // Unique constraint (can have multiple)
#[index(btree)]             // Single-column BTree index
```

### Column types

```rust
u8, u16, u32, u64, u128     // Unsigned integers
i8, i16, i32, i64, i128     // Signed integers
f32, f64                     // Floats
bool                         // Boolean
String                       // Text
Identity                     // User identity
Timestamp                    // Timestamp
ScheduleAt                   // For scheduled tables
Option<T>                    // Nullable
Vec<T>                       // Arrays
```

### Insert returns the row

```rust
// Insert and get the auto-generated ID
let row = ctx.db.task().insert(Task {
    id: 0,  // Placeholder for auto_inc
    owner_id: ctx.sender(),
    title: "New task".to_string(),
    created_at: ctx.timestamp,
});
let new_id = row.id;  // Get the actual ID
```

---

## 3) Index Access

### Naming convention
- **Tables**: snake_case methods on `ctx.db`
  - `#[table(name = my_table)]` → `ctx.db.my_table()`
- **Indexes**: exact declared name
  - `index(name = by_owner, ...)` → `ctx.db.my_table().by_owner()`

### Primary key operations

```rust
// Find by primary key — returns Option<Row>
if let Some(task) = ctx.db.task().id().find(&task_id) {
    // Use task
}

// Update by primary key
ctx.db.task().id().update(Task { id: task_id, ...updated_fields });

// Delete by primary key
ctx.db.task().id().delete(&task_id);
```

### Index filter

```rust
// Filter by indexed column — returns iterator
for task in ctx.db.task().by_owner().filter(&owner_id) {
    // Process each task
}
```

### Unique column lookup

```rust
// Find by unique column — returns Option<Row>
if let Some(player) = ctx.db.player().username().find(&"alice".to_string()) {
    // Found player
}
```

### Iterate all rows

```rust
// Full table scan
for task in ctx.db.task().iter() {
    // Process each task
}
```

---

## 4) Reducers

### Definition syntax

```rust
use spacetimedb::{reducer, ReducerContext, Table};

#[reducer]
pub fn create_task(ctx: &ReducerContext, title: String) {
    // Validate
    if title.is_empty() {
        panic!("Title cannot be empty");  // Rolls back transaction
    }

    // Insert
    ctx.db.task().insert(Task {
        id: 0,
        owner_id: ctx.sender(),
        title,
        created_at: ctx.timestamp,
    });
}

// With Result return type (preferred for recoverable errors)
#[reducer]
pub fn update_task(ctx: &ReducerContext, task_id: u64, title: String) -> Result<(), String> {
    let task = ctx.db.task().id().find(&task_id)
        .ok_or("Task not found")?;

    if task.owner_id != ctx.sender() {
        return Err("Not authorized".to_string());
    }

    ctx.db.task().id().update(Task { title, ..task });
    Ok(())
}
```

### Lifecycle reducers

```rust
#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    // Called when module is first published
}

#[reducer(client_connected)]
pub fn on_connect(ctx: &ReducerContext) {
    // ctx.sender() is the connecting client
    log::info!("Client connected: {:?}", ctx.sender());
}

#[reducer(client_disconnected)]
pub fn on_disconnect(ctx: &ReducerContext) {
    // Clean up client state
}
```

### ReducerContext fields

```rust
ctx.sender()          // Identity of the caller
ctx.timestamp       // Current timestamp
ctx.db              // Database access
ctx.rng             // Deterministic RNG (use instead of rand)
```

### Error handling

```rust
// Option 1: Panic (simple, destroys WASM instance)
if condition_failed {
    panic!("Error message");
}

// Option 2: Result (preferred, graceful error handling)
#[reducer]
pub fn my_reducer(ctx: &ReducerContext) -> Result<(), String> {
    do_something().map_err(|e| e.to_string())?;
    Ok(())
}
```

---

## 5) Custom Types

**Use `#[derive(SpacetimeType)]` ONLY for custom structs/enums used as fields or parameters.**

```rust
use spacetimedb::SpacetimeType;

// Custom struct for table fields
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Custom enum
#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum PlayerStatus {
    Idle,
    Walking(Position),
    Fighting(Identity),
}

// Use in table
#[table(name = player, public)]
pub struct Player {
    #[primary_key]
    pub id: Identity,
    pub position: Position,
    pub status: PlayerStatus,
}
```

---

## 6) Scheduled Tables

```rust
use spacetimedb::{table, reducer, ReducerContext, Table, ScheduleAt};

#[table(name = reminder, scheduled(send_reminder))]
pub struct Reminder {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub message: String,
    pub scheduled_at: ScheduleAt,
}

// Scheduled reducer receives the full row
#[reducer]
fn send_reminder(ctx: &ReducerContext, reminder: Reminder) {
    log::info!("Reminder: {}", reminder.message);
    // Row is automatically deleted after reducer completes
}

// Schedule a reminder
#[reducer]
pub fn create_reminder(ctx: &ReducerContext, message: String, delay_secs: u64) {
    let future_time = ctx.timestamp + std::time::Duration::from_secs(delay_secs);
    ctx.db.reminder().insert(Reminder {
        id: 0,
        message,
        scheduled_at: ScheduleAt::Time(future_time),
    });
}

// Cancel by deleting the row
#[reducer]
pub fn cancel_reminder(ctx: &ReducerContext, reminder_id: u64) {
    ctx.db.reminder().id().delete(&reminder_id);
}
```

---

## 7) Timestamps

```rust
use spacetimedb::Timestamp;

// Current time from context
let now = ctx.timestamp;

// Create future timestamp
let future = ctx.timestamp + std::time::Duration::from_secs(60);

// Compare timestamps
if row.created_at < ctx.timestamp {
    // Row was created before now
}
```

---

## 8) Data Visibility

**`public` flag exposes ALL rows to ALL clients.**

| Scenario | Pattern |
|----------|---------|
| Everyone sees all rows | `#[table(name = x, public)]` |
| Users see only their data | Private table + row-level security |

### Private table (default)

```rust
// No public flag — only server can read
#[table(name = secret_data)]
pub struct SecretData { ... }
```

### Row-level security

```rust
// Use row-level security for per-user visibility
#[table(name = player_data, public)]
#[rls(filter = |ctx, row| row.owner_id == ctx.sender())]
pub struct PlayerData {
    #[primary_key]
    pub id: u64,
    pub owner_id: Identity,
    pub data: String,
}
```

---

## 9) Procedures (Beta)

**Procedures are for side effects (HTTP, filesystem) that reducers can't do.**

Procedures are currently unstable. Enable with:

```toml
# Cargo.toml
[dependencies]
spacetimedb = { version = "1.*", features = ["unstable"] }
```

```rust
use spacetimedb::{procedure, ProcedureContext};

// Simple procedure
#[procedure]
fn add_numbers(_ctx: &mut ProcedureContext, a: u32, b: u32) -> u64 {
    a as u64 + b as u64
}

// Procedure with database access
#[procedure]
fn save_external_data(ctx: &mut ProcedureContext, url: String) -> Result<(), String> {
    // HTTP request (allowed in procedures, not reducers)
    let data = fetch_from_url(&url)?;

    // Database access requires explicit transaction
    ctx.try_with_tx(|tx| {
        tx.db.external_data().insert(ExternalData {
            id: 0,
            content: data,
        });
        Ok(())
    })?;

    Ok(())
}
```

### Key differences from reducers

| Reducers | Procedures |
|----------|------------|
| `&ReducerContext` (immutable) | `&mut ProcedureContext` (mutable) |
| Direct `ctx.db` access | Must use `ctx.with_tx()` |
| No HTTP/network | HTTP allowed |
| No return values | Can return data |

---

## 10) Logging

```rust
use spacetimedb::log;

log::trace!("Detailed trace");
log::debug!("Debug info");
log::info!("Information");
log::warn!("Warning");
log::error!("Error occurred");
```

---

## 11) Commands

```bash
# Start local server
spacetime start

# Publish module
spacetime publish <module-name> --project-path <backend-dir>

# Clear database and republish
spacetime publish <module-name> --clear-database -y --project-path <backend-dir>

# Generate bindings
spacetime generate --lang rust --out-dir <client>/src/server/bindings --project-path <backend-dir>

# View logs
spacetime logs <module-name>
```

---

## 12) Hard Requirements

1. **DO NOT derive `SpacetimeType` on `#[table]` structs** — the macro handles this
2. **Import `Table` trait** — required for all table operations
3. **Use `&ReducerContext`** — not `&mut ReducerContext`
4. **Tables are methods** — `ctx.db.table()` not `ctx.db.table`
5. **Reducers must be deterministic** — no filesystem, network, timers, or external RNG
6. **Use `ctx.rng`** — not `rand` crate for random numbers
7. **Add `public` flag** — if clients need to subscribe to a table

---

# SpacetimeDSL — Project-Specific Layer

> This project uses a **custom `spacetimedsl` crate** built on top of SpacetimeDB.
> All server-side code (`server/src/`) must use the DSL patterns below.
> **Last updated:** 2026-05-07

---

## Overview

`spacetimedsl` is a code-generation layer. It wraps standard SpacetimeDB table/reducer macros and generates:
- Strongly-typed ID wrapper structs (e.g., `PlayerId`, `SectorId`)
- `Create<Table>` structs for insertion
- Getter/setter methods on table rows
- CRUD methods on a `DSL<T>` context object

---

## Imports

```rust
use crate::spacetimedsl::prelude::*;        // Always — pulls in DSL, dsl(), WriteContext, etc.
use spacetimedb::*;                         // For ReducerContext, ScheduleAt, etc.
```

---

## Table Definition

Tables use `accessor =` (not `name =`) inside `#[table]`, paired with `#[dsl(...)]`:

```rust
#[dsl(plural_name = players, method(update = true))]
#[table(accessor = player, public)]
pub struct Player {
    #[primary_key]
    #[create_wrapper]           // Marks this field as the key in CreatePlayer wrapper
    id: Identity,

    pub username: String,
    pub credits: u64,
}
```

### `#[dsl(...)]` options

| Attribute | Purpose |
|-----------|---------|
| `plural_name = <ident>` | Name for `get_all_<plural_name>()` iterator method |
| `method(update = true)` | Generate `dsl.update_<table>_by_id()` method |
| `method(update = false)` | Skip update method (e.g. insert-only timer tables) |

### Extra field attributes

| Attribute | Purpose |
|-----------|---------|
| `#[create_wrapper]` | Marks the field included in `Create<Table>` struct |
| `#[referenced_by(path = crate::tables::ships, table = ship)]` | Documents foreign key relationships (informational) |

---

## Reducers

Always use the full-path form `#[spacetimedb::reducer]` (project convention):

```rust
#[spacetimedb::reducer]
pub fn update_ship_movement_controller(
    ctx: &ReducerContext,
    forward: bool,
) -> Result<(), String> {
    let dsl = dsl(ctx);   // Get the DSL handle
    // ...
    Ok(())
}
```

---

## Using the DSL

Get a DSL handle inside a reducer with `dsl(ctx)`:

```rust
let dsl = dsl(ctx);
```

For helper functions called from reducers, type the `dsl` parameter as `&DSL<'_, ReducerContext>`:

```rust
pub fn my_helper(dsl: &DSL<'_, ReducerContext>) -> Result<(), String> {
    // ...
}
```

**Never pass `ctx` alongside `dsl`.** `dsl` is a superset of `ctx` — reach the underlying `ReducerContext` via `dsl.ctx()` when you need `rng()`, `sender()`, or `timestamp`. A signature like `fn foo(ctx: &ReducerContext, dsl: &DSL<T>, ...)` is a sign that `dsl` was typed too generically; collapse to the concrete form above.

Generic `&DSL<T>` (with a `T: spacetimedsl::WriteContext` bound) is reserved for helpers that must work across reducer *and* procedure contexts. This project has no procedures today, so prefer the concrete form for new helpers.

---

## Generated CRUD Methods

Given `#[dsl(plural_name = players, method(update = true))]` on `Player`:

```rust
// Read — by primary key (returns Result<Row, String>)
let player = dsl.get_player_by_id(&PlayerId::new(identity))?;

// Read — by indexed/unique column
let players = dsl.get_players_by_faction_id(&faction_id);  // returns iterator

// Read — full scan
for player in dsl.get_all_players() { ... }

// Create — uses CreatePlayer wrapper
dsl.create_player(CreatePlayer { id: identity, username: "Alice".into(), credits: 0 })?;

// Update (only if method(update = true))
dsl.update_player_by_id(modified_player)?;

// Delete
dsl.delete_player_by_id(&player_id)?;
```

---

## Generated Getters / Setters

Private fields on table structs get generated accessors:

```rust
// Getter — returns reference or copy
let id = player.get_id();               // returns &PlayerId or PlayerId
let speed = ship_type.get_base_speed(); // returns &f32

// Setter — mutates field in place
velocity.set_rotation_radians(0.5f32);
velocity.set_x(new_x);
```

---

## ID Wrapper Types

Each table gets a typed ID wrapper (e.g. `PlayerId`, `SectorId`):

```rust
// Construct from raw value
let player_id = PlayerId::new(ctx.sender());

// Get raw value
let identity: Identity = player_id.value();
```

---

## `Create<Table>` Structs

Tables with `#[create_wrapper]` on a field get a `Create<Table>` struct:

```rust
dsl.create_ship_movement_controller(CreateShipMovementController {
    id: player.clone(),
    stellar_object_id: sobj.get_id(),
    forward: false,
    backward: false,
    left: false,
    right: false,
})?;
```

---

## Scheduled Timer Tables

Timer tables use `#[dsl]` + `scheduled(reducer_name)` and typically set `method(update = false)`:

```rust
#[dsl(plural_name = create_update_ship_movement_controllers_timers, method(update = false))]
#[table(
    accessor = create_update_ship_movement_controllers_timer,
    scheduled(timer_update_all_ship_movement_controllers)
)]
pub struct UpdateShipMovementControllers {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,
    scheduled_at: spacetimedb::ScheduleAt,
}

#[spacetimedb::reducer]
pub fn timer_update_all_ship_movement_controllers(
    ctx: &ReducerContext,
    _timer: UpdateShipMovementControllers,
) -> Result<(), String> {
    let dsl = dsl(ctx);
    for item in dsl.get_all_create_update_ship_movement_controllers_timers() {
        // process...
    }
    Ok(())
}
```

---

## Hard Requirements (DSL)

1. **Use `#[table(accessor = ...)]`** — not `name =` — in this codebase
2. **Pair `#[dsl(...)]` with every `#[table]`** — defines plural name and update method
3. **Use `#[spacetimedb::reducer]`** (full path) — project convention
4. **Use `dsl(ctx)` inside reducers** — not `ctx.db` directly
5. **Type helper `dsl` params as `&DSL<'_, ReducerContext>`** — never pass `ctx` alongside `dsl`; reach the context via `dsl.ctx()`. Generic `&DSL<T>` is reserved for cross-reducer/procedure reuse.
6. **Use generated CRUD methods** — don't call `ctx.db.*` for tables covered by DSL
7. **Use ID wrappers** — `PlayerId::new(identity)`, not raw `Identity` where typed ID exists
