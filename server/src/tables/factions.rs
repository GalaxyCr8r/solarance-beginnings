use log::info;
use spacetimedb::{table, Identity, SpacetimeType, TimeDuration};
use spacetimedsl::*;

use crate::{
    logic::factions::CreateFactionShipReactionTimerRow,
    tables::{sectors::*, ships::*, stations::*},
};

#[derive(SpacetimeType, Clone, Debug, PartialEq)]
pub enum FactionTier {
    /// A meta-type of faction. Should always be unjoinable. Made up of other factions.
    Alliance,
    /// The largest type of faction - new players can spawn into it.
    Galactic,
    /// Multi-corporation faction, but still under a galactic faction's aegis
    Conglomerate,
    /// A large player-controlled faction.
    Guild,
    /// A small/medium player-controlled faction.
    Corporation,
    /// A small, usually temporary, group within a faction.
    Squad, // e.g., local corporation, pirate clan
}

#[dsl(plural_name = factions, method(update = true))]
#[table(name = faction, public)]
pub struct Faction {
    #[primary_key]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::star_system, table = star_system)]
    #[referenced_by(path = crate::tables::sectors, table = sector)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::stations, table = station)]
    #[referenced_by(path = crate::tables::chats, table = faction_chat_message)]
    #[referenced_by(path = crate::tables::factions, table = faction_standing)]
    //#[referenced_by(path = crate::tables::factions, table = player_faction_standing)]
    id: u32,

    /// The Faction ID of the parent faction, if any.
    pub parent_id: Option<FactionId>,

    pub name: String,
    pub short_name: String,
    pub description: String,

    /// The tier/size of this faction.
    pub tier: FactionTier,

    /// Whether players can join this faction.
    /// Some factions like Vancellan are antagonistic, or not large enough, and not joinable by players.
    pub joinable: bool,

    /// Faction's Capital Station, if it exists.
    /// FK to Station.
    pub capital_station_id: Option<u64>,
    // Other faction-specific data like relations, home sector, etc.
}

#[dsl(plural_name = faction_standings, method(update = true))]
#[table(name = faction_standing, public)]
pub struct FactionStanding {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)] // To find all players with standing for a faction
    #[use_wrapper(FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    pub faction_one_id: u32,

    #[index(btree)] // To find all players with standing for a faction
    #[use_wrapper(FactionId)]
    #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
    /// FK to FactionDefinition
    pub faction_two_id: u32,

    /// How the two factions regard each other.
    /// -100 is hated enemies.
    /// 100 is erstwhile allies.
    pub reputation_score: i32,
}

// We are ignoring player-level faction standings at the moment.

// #[dsl(plural_name = player_faction_standings, method(update = true))]
// #[table(name = player_faction_standing, public)]
// pub struct PlayerFactionStanding {
//     #[primary_key]
//     #[auto_inc]
//     #[create_wrapper]
//     id: u64,

//     #[index(btree)] // To find all standings for a player
//     #[use_wrapper(crate::players::PlayerId)]
//     #[foreign_key(path = crate::tables::players, table = player, column = id, on_delete = Delete)]
//     pub player_identity: Identity,

//     #[index(btree)] // To find all players with standing for a faction
//     #[use_wrapper(FactionId)]
//     #[foreign_key(path = crate::tables::factions, table = faction, column = id, on_delete = Error)]
//     /// FK to FactionDefinition
//     pub faction_id: u32,

//     pub reputation_score: i32,
// }

/////////////////////////////////////////////////////////////////////
/// Utilities

/// Gets the faction name for display purposes, with fallback for unknown factions
pub fn get_faction_name<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> String {
    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        faction.get_name().clone()
    } else {
        format!("Unknown Faction #{}", faction_id.value())
    }
}

/// Checks if a faction exists and is joinable by players
pub fn is_faction_joinable<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> bool {
    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        *faction.get_joinable()
    } else {
        false
    }
}

/// Gets the faction tier for a given faction
pub fn get_faction_tier<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Option<FactionTier> {
    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        Some(faction.get_tier().clone())
    } else {
        None
    }
}

/// Gets the reputation score between two factions
/// Returns 0 (neutral) if no standing exists
pub fn get_faction_reputation<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> i32 {
    // Look for existing standing between the factions
    for standing in dsl.get_all_faction_standings() {
        if standing.get_faction_one_id().value() == faction_one_id.value()
            && standing.get_faction_two_id().value() == faction_two_id.value()
        {
            return *standing.get_reputation_score();
        }
    }

    // Default to neutral if no standing exists
    0
}

/// Checks if two factions are hostile to each other (reputation < -50)
pub fn are_factions_hostile<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> bool {
    get_faction_reputation(dsl, faction_one_id, faction_two_id) < -50
}

/// Checks if two factions are allied (reputation > 50)
pub fn are_factions_allied<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_one_id: &FactionId,
    faction_two_id: &FactionId,
) -> bool {
    get_faction_reputation(dsl, faction_one_id, faction_two_id) > 50
}

/// Gets all stations belonging to a specific faction
pub fn get_faction_stations<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Vec<Station> {
    dsl.get_all_stations()
        .filter(|station| station.get_owner_faction_id().value() == faction_id.value())
        .collect()
}

/// Gets all ships belonging to a specific faction
pub fn get_faction_ships<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Vec<Ship> {
    dsl.get_all_ships()
        .filter(|ship| ship.get_faction_id().value() == faction_id.value())
        .collect()
}

/// Counts the total number of stations controlled by a faction
pub fn count_faction_stations<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> usize {
    get_faction_stations(dsl, faction_id).len()
}

/// Counts the total number of ships controlled by a faction
pub fn count_faction_ships<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> usize {
    get_faction_ships(dsl, faction_id).len()
}

/// Gets all sectors where a faction has presence (stations or significant ship activity)
pub fn get_faction_controlled_sectors<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Vec<SectorId> {
    let mut controlled_sectors = Vec::new();

    // Add sectors with faction stations
    for station in get_faction_stations(dsl, faction_id) {
        let sector_id = station.get_sector_id();
        if !controlled_sectors.contains(&sector_id) {
            controlled_sectors.push(sector_id.clone());
        }
    }

    // TODO: Could also add sectors with significant ship presence
    // TODO: Could add sectors with faction-controlled jump gates

    controlled_sectors
}

/// Handles when a faction ship is destroyed - creates reaction timer
pub fn handle_faction_ship_destroyed<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    destroyed_ship: &Ship,
    aggressor_faction_id: Option<&FactionId>,
    destruction_sector_id: &SectorId,
) -> Result<(), String> {
    let faction_id = destroyed_ship.get_faction_id();

    // Only react if it's a faction ship (not player-owned)
    if destroyed_ship.get_player_id().value() == Identity::ONE {
        let _ = dsl.create_faction_ship_reaction_timer(
            spacetimedb::ScheduleAt::Interval(TimeDuration::from_micros(1000)),
            &faction_id,
            aggressor_faction_id.cloned(),
            &destroyed_ship.get_shiptype_id(),
            destruction_sector_id,
            dsl.ctx().timestamp(),
        );

        info!(
            "Faction {} will react to the destruction of their {} in sector #{}",
            get_faction_name(dsl, &faction_id),
            destroyed_ship.get_shiptype_id().value(),
            destruction_sector_id.value()
        );
    }

    Ok(())
}

/// Gets a list of all joinable factions for player selection
pub fn get_joinable_factions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Vec<Faction> {
    dsl.get_all_factions()
        .filter(|faction| *faction.get_joinable())
        .collect()
}

/// Gets faction capital station if it exists
pub fn get_faction_capital_station<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_id: &FactionId,
) -> Option<Station> {
    if let Ok(faction) = dsl.get_faction_by_id(faction_id) {
        if let Some(capital_station_id) = faction.get_capital_station_id() {
            if let Ok(station) = dsl.get_station_by_id(&StationId::new(*capital_station_id)) {
                return Some(station);
            }
        }
    }

    None
}
