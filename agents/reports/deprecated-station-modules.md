# Deprecated Station Modules

> **Source:** `server/src/tables/stations/modules_deprecated/`
> **Status:** Deleted 2026-05-06. Preserved here for reimplementation reference.

These modules were table definitions (and in some cases factory/timer logic) for station sub-module types. Each module's primary key is a FK to a parent `StationModule` row (`StationModuleId`). They were removed because they were unused and not yet wired into the production tick system.

---

## `AntiCapitalTurret` (`anti_capital_turret.rs`)

A defensive weapon module mounted on a station. Holds a reference to a `ShipModuleBlueprint` that defines weapon stats (damage, range, fire rate). Tracks an optional current target ship and supports launching fighters (with optional fighter capacity). Ammo and fuel were intended to live in `StationModuleInventoryItem`.

**Table:** `anti_capital_turret_module`

| Field | Type | Notes |
|---|---|---|
| `weapon_core_blueprint_id` | `ItemDefinitionId` | Blueprint defining weapon stats |
| `current_target_ship_id` | `Option<ShipId>` | Active combat target |
| `can_launch_fighters` | `bool` | Whether fighters can be deployed |
| `fighter_capacity` | `Option<u8>` | Max fighters stored |

---

## `CapitalDock` (`capital_dock.rs`)

A docking bay for capital-class ships. Tracks how many capital ships can be docked simultaneously. A companion table `DockedCapitalShipAt` links individual capital ship instances to a specific dock module, recording when they docked.

**Tables:** `capital_dock_module`, `docked_capital_ship_at_module`

| Field | Type | Notes |
|---|---|---|
| `max_capital_ship_capacity` | `u8` | Max concurrent capital ships |
| *(DockedCapitalShipAt)* `capital_dock_module_id` | `StationModuleId` | FK to the dock |
| *(DockedCapitalShipAt)* `docked_at_timestamp` | `Timestamp` | When docking occurred |

---

## `Embassy` (`embassy.rs`)

A diplomatic presence module. The module itself is a marker table (no extra fields beyond the FK). A companion table `EmbassyPresence` tracks which factions have an active diplomatic presence at this embassy, with a manually-enforced composite key (`embassy_module_id:representing_faction_id`), an establishment timestamp, and optional status notes.

**Tables:** `embassy_module`, `embassy_presence`

| Field | Type | Notes |
|---|---|---|
| *(EmbassyPresence)* `representing_faction_id` | `FactionId` | Faction with presence |
| *(EmbassyPresence)* `established_at_timestamp` | `Timestamp` | When presence began |
| *(EmbassyPresence)* `diplomatic_status_notes` | `Option<String>` | e.g. "Ambassadorial level" |

---

## `Farm` (`farm.rs`, `farm/definitions.rs`, `farm/timers.rs`)

A food production module. Converts one or two input resources (primary: compost, secondary: water) into a food output resource at a configurable efficiency. Output quality maps to four tiers: Lower, Average, Upper, Luxury.

**Table:** `farm_module`

| Field | Type | Notes |
|---|---|---|
| `output_resource_id` | `u32` | The food item produced |
| `output_quality` | `FarmOutputQuality` | Lower / Average / Upper / Luxury |
| `primary_input_resource_id` | `ItemDefinitionId` | e.g. Biomatter Compost |
| `primary_input_conversion_rate` | `f32` | Units of primary input per unit of output |
| `secondary_input_resource_id` | `Option<ItemDefinitionId>` | e.g. Water |
| `secondary_input_conversion_rate` | `Option<f32>` | Units of secondary input per unit of output |
| `base_production_units_per_hour` | `f32` | Base rate at 1.0 efficiency |
| `current_efficiency_modifier` | `f32` | Multiplier; affected by station health and operational status |

**Blueprint ID constants** (from `definitions.rs`):

| Constant | ID |
|---|---|
| `MODULE_FARM_BASIC` | 4000 |
| `MODULE_FARM_STANDARD` | 4001 |
| `MODULE_FARM_ADVANCED` | 4002 |
| `MODULE_FARM_LUXURY` | 4003 |

**Production logic** (`timers.rs`): `calculate_farm_production` checks available inventory for both inputs, derives actual producible whole units, and returns a `FarmProductionResult`. `apply_farm_production` debits inputs and credits the output inventory slot. Efficiency is reduced by station health (scaled by `health / 100`) and zeroed if the module is not operational. Max efficiency cap is 2.0.

---

## `Hospital` (`hospital.rs`)

A medical facility module. Heals players and NPCs up to a concurrent capacity, with a configurable effectiveness modifier. Optionally provides a morale boost to the broader sector.

**Table:** `hospital_module`

| Field | Type | Notes |
|---|---|---|
| `medical_bay_capacity` | `u16` | Max simultaneous patients |
| `healing_effectiveness_modifier` | `f32` | Multiplier, base 1.0 |
| `sector_morale_boost_value` | `Option<i16>` | Sector-wide morale bonus |

---

## `Laboratory` (`laboratory.rs`, `laboratory/definitions.rs`, `laboratory/timers.rs`)

A research production module. Consumes research materials (Viveium Crystals, Research Devices, exotic fragments) at a per-hour rate and produces research fragments. Three tiers: Basic, Advanced, Exotic.

**Table:** `laboratory_module`

| Field | Type | Notes |
|---|---|---|
| `base_research_points_per_hour` | `u32` | Raw research rate |
| `primary_input_resource_id` | `ItemDefinitionId` | e.g. Viveium Crystal |
| `secondary_input_resource_id` | `Option<ItemDefinitionId>` | e.g. Research Device |
| `primary_input_consumption_rate` | `f32` | Units consumed per hour |
| `secondary_input_consumption_rate` | `Option<f32>` | Units consumed per hour |
| `current_efficiency_modifier` | `f32` | Multiplier |

**Blueprint ID constants** (from `definitions.rs`):

| Constant | ID |
|---|---|
| `MODULE_LABORATORY_BASIC` | 5000 |
| `MODULE_LABORATORY_ADVANCED` | 5001 |
| `MODULE_LABORATORY_EXOTIC` | 5002 |

**Output mapping:**
- Basic → `ITEM_RESEARCH_FRAGMENT`
- Advanced → `ITEM_RESEARCH_FRAGMENT_RARE`
- Exotic → `ITEM_RESEARCH_FRAGMENT_EXOTIC`

**Production logic** (`timers.rs`): `calculate_laboratory_production` computes research points as a fraction of full production limited by available inputs (10 points = 1 fragment). `apply_laboratory_production` debits inputs and credits fragments. Output fragment type is determined by which exotic inputs are present. Efficiency follows the same station-health and operational-status pattern as Farm.

---

## `Manufacturing` (`manufacturing/definitions.rs`, `manufacturing/timers.rs`)

Module files exist but contain only bare `use` imports — no struct definition or logic was written yet. Manufacturing was intended as a production module but is entirely skeletal.

**Table:** *(none defined)*

---

## `Observatory` (`observatory.rs`)

A passive data-gathering module that generates research points by observing the sector environment (e.g., nebulae, anomalies). Consumes a sensor input resource at an optional per-hour rate and produces a raw astronomical data fragment.

**Table:** `observatory_module`

| Field | Type | Notes |
|---|---|---|
| `base_research_points_per_hour` | `u32` | Base research rate |
| `current_efficiency_modifier` | `f32` | Sector-type and upgrade multiplier |
| `primary_input_resource_id` | `ItemDefinitionId` | e.g. Advanced Sensor Crystal |
| `primary_input_consumption_rate` | `Option<f32>` | Units per hour (optional) |
| `output_data_fragment_resource_id` | `u32` | e.g. Raw Astronomical Data |

---

## `Residential` (`residential.rs`)

A population and crew housing module. Tracks occupancy, morale, and crew replenishment pool. Supports three tiers in one struct via optional fields: standard (base fields only), spacious (adds amenity upgrade level and morale boost), and luxury (adds luxury NPC slots and upkeep resource requirements).

**Table:** `residential_module`

| Field | Type | Notes |
|---|---|---|
| `base_max_occupancy` | `u32` | Blueprint-defined capacity |
| `current_occupancy` | `u32` | Current population/crew count |
| `current_internal_morale_percentage` | `f32` | 0.0–100.0 |
| `crew_replenishment_pool` | `u32` | Crew available for players to hire |
| `crew_generation_rate_per_hour` | `f32` | Rate of crew pool growth |
| `amenity_upgrade_level` | `Option<u8>` | Spacious/Luxury tier upgrade level |
| `amenity_morale_boost` | `Option<i16>` | Morale bonus from amenities |
| `max_luxury_npc_slots` | `Option<u8>` | Luxury tier NPC capacity |
| `current_luxury_npc_count` | `Option<u8>` | Current luxury NPCs |
| `luxury_upkeep_requirements` | `Option<Vec<ResourceAmount>>` | Luxury food/drink costs |

---

## `StorageDepot` (`storage_depot.rs`)

A marker table indicating that a station module is a storage depot. No extra fields — storage capacity configuration lives in `StationModuleBlueprint` (`max_internal_storage_slots` / `max_internal_storage_volume_per_slot_m3`).

**Table:** `storage_depot_module`

---

## `Synthesizer` (`synthesizer.rs`)

A jump-fuel production module. Combines exotic matter and a gas resource to produce jump fuel at a configurable conversion rate and per-hour output.

**Table:** `synthesizer_module`

| Field | Type | Notes |
|---|---|---|
| `input_exotic_matter_resource_id` | `ItemDefinitionId` | Exotic matter input |
| `input_gas_resource_id` | `ItemDefinitionId` | Gas input |
| `output_jump_fuel_resource_id` | `ItemDefinitionId` | Jump fuel output |
| `exotic_matter_per_fuel_unit` | `f32` | Consumption ratio |
| `gas_per_fuel_unit` | `f32` | Consumption ratio |
| `base_fuel_units_produced_per_hour` | `f32` | Base production rate |

---

## Reimplementation Notes

- All modules share the same pattern: a sub-module table whose `id` is a FK to `StationModule`, plus optional companion tables.
- Production modules (Farm, Laboratory) had working calculate/apply logic in their `timers.rs` files that is worth reusing.
- Blueprint ID constants for Farm (4000–4003) and Laboratory (5000–5002) need to exist in the item definitions before the modules can be instantiated.
- Manufacturing was entirely unimplemented — design from scratch.
- The `#[dsl(...)]` and `#[use_wrapper(...)]` attributes are project-specific macros; ensure the DSL layer supports corresponding create/update/get methods before wiring production ticks.
