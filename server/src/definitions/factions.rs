use log::info;

use spacetimedsl::*;

use crate::tables::factions::*;

// Faction IDs
pub const FACTION_FACTIONLESS: u32 = 0;
pub const FACTION_LRAK_COMBINE: u32 = 1;
pub const FACTION_INDEPENDENT_WORLDS_ALLIANCE: u32 = 2;
pub const FACTION_FREE_TRADE_UNION: u32 = 3;
pub const FACTION_REDIAR_FEDERATION: u32 = 4;
pub const FACTION_VANCELLAN: u32 = 5;

pub const FACTION_ALLIANCE_PROCYON: u32 = 10;

// Reputation scores
pub const REPUTATION_HOSTILE: i32 = -75;
pub const REPUTATION_DISLIKED: i32 = -25;
pub const REPUTATION_NEUTRAL: i32 = 0;
pub const REPUTATION_FRIENDLY: i32 = 25;
pub const REPUTATION_ALLIED: i32 = 75;

//////////////////////////////////////////////////////////////
// Init
//////////////////////////////////////////////////////////////

pub fn init<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    factions(dsl)?;
    faction_standings(dsl)?;

    info!("Faction Defs Loaded: {}", dsl.count_of_all_factions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn factions<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    let pc = Some(FactionId::new(FACTION_ALLIANCE_PROCYON));

    // Factionless - neutral faction for players who want no faction affiliation
    dsl.create_faction(CreateFaction {
        id: FACTION_FACTIONLESS,
        parent_id: None,
        name: "Factionless".to_string(),
        short_name: "FX".to_string(),
        description: "Independent operators who have chosen to remain neutral in galactic politics. Factionless individuals trade freely with all factions but receive no protection or special privileges from any government. They must rely on their own skills and resources to survive in the galaxy.".to_string(),
        tier: FactionTier::Galactic,
        joinable: true,
        capital_station_id: None,
    })?;

    // Lrak Combine - disliked by all other factions (Galactic tier, joinable)
    dsl.create_faction(CreateFaction {
        id: FACTION_LRAK_COMBINE,
        parent_id: pc.clone(),
        name: "Lrak Combine".to_string(),
        short_name: "LC".to_string(),
        description: "A militaristic faction known for their aggressive expansion and authoritarian rule dependent on their control of humanity's homeworld. The Lrak Combine seeks to dominate through superior firepower and strict hierarchical control.".to_string(),
        tier: FactionTier::Galactic,
        joinable: true,
        capital_station_id: None,
    })?;

    // Independent Worlds Alliance - disliked by Lrak and FTU, neutral to others (Galactic tier, joinable)
    dsl.create_faction(CreateFaction {
        id: FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        parent_id: pc.clone(),
        name: "Independent Worlds Alliance".to_string(),
        short_name: "IWA".to_string(),
        description: "A loose confederation of independent star systems that value autonomy and self-governance. The IWA formed as a defensive alliance against larger, more aggressive factions.".to_string(),
        tier: FactionTier::Galactic,
        joinable: true,
        capital_station_id: None,
    })?;

    // Free Trade Union - disliked by everybody (Galactic tier, joinable)
    dsl.create_faction(CreateFaction {
        id: FACTION_FREE_TRADE_UNION,
        parent_id: None,
        name: "Free Trade Union".to_string(),
        short_name: "FTU".to_string(),
        description: "A corporate-dominated faction that prioritizes profit above all else. The FTU's ruthless business practices and exploitation of resources has earned them enemies across the galaxy.".to_string(),
        tier: FactionTier::Galactic,
        joinable: true,
        capital_station_id: None,
    })?;

    // Rediar Federation - neutral to IWA, disliked by everyone else (Galactic tier, joinable)
    dsl.create_faction(CreateFaction {
        id: FACTION_REDIAR_FEDERATION,
        parent_id: pc.clone(),
        name: "Rediar Federation".to_string(),
        short_name: "RF".to_string(),
        description: "A technocratic republic that values scientific advancement and technological superiority. The Rediar Federation's elitist attitudes and secretive research programs create tension with other factions.".to_string(),
        tier: FactionTier::Galactic,
        joinable: true,
        capital_station_id: None,
    })?;

    // Vancellan - enemies to everyone (Galactic tier, NOT joinable - antagonistic faction)
    dsl.create_faction(CreateFaction {
        id: FACTION_VANCELLAN,
        parent_id: None,
        name: "Vancellan".to_string(),
        short_name: "VCN".to_string(),
        description: "A mysterious and hostile faction of unknown origin. The Vancellan are xenophobic extremists who view all other factions as threats to be eliminated. Their advanced biotechnology and ruthless tactics make them feared throughout the galaxy.".to_string(),
        tier: FactionTier::Galactic,
        joinable: false,
        capital_station_id: None,
    })?;

    // The alliance formed at Procyon - An affliation of factions who are coordinating the counter-attack against the Vancellans.
    dsl.create_faction(CreateFaction {
        id: FACTION_ALLIANCE_PROCYON,
        parent_id: None,
        name: "Procyon Compact".to_string(),
        short_name: "PC".to_string(),
        description: "The alliance formed at Procyon - An affliation of factions who are coordinating the counter-attack against the Vancellans.".to_string(),
        tier: FactionTier::Alliance,
        joinable: false,
        capital_station_id: None,
    })?;

    Ok(())
}

fn faction_standings<T: spacetimedsl::WriteContext>(dsl: &DSL<T>) -> Result<(), String> {
    // Factionless relationships (neutral with everyone except hostile to Vancellan)
    create_mutual_standing(
        dsl,
        FACTION_FACTIONLESS,
        FACTION_LRAK_COMBINE,
        REPUTATION_NEUTRAL,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_FACTIONLESS,
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        REPUTATION_NEUTRAL,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_FACTIONLESS,
        FACTION_FREE_TRADE_UNION,
        REPUTATION_NEUTRAL,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_FACTIONLESS,
        FACTION_REDIAR_FEDERATION,
        REPUTATION_NEUTRAL,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_FACTIONLESS,
        FACTION_VANCELLAN,
        REPUTATION_HOSTILE,
    )?;

    // Lrak Combine relationships (disliked by all)
    create_mutual_standing(
        dsl,
        FACTION_LRAK_COMBINE,
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        REPUTATION_DISLIKED,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_LRAK_COMBINE,
        FACTION_FREE_TRADE_UNION,
        REPUTATION_DISLIKED,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_LRAK_COMBINE,
        FACTION_REDIAR_FEDERATION,
        REPUTATION_DISLIKED,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_LRAK_COMBINE,
        FACTION_VANCELLAN,
        REPUTATION_HOSTILE,
    )?;

    // IWA relationships (disliked by Lrak and FTU, neutral to RF, hostile to Vancellan)
    create_mutual_standing(
        dsl,
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        FACTION_FREE_TRADE_UNION,
        REPUTATION_DISLIKED,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        FACTION_REDIAR_FEDERATION,
        REPUTATION_FRIENDLY,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        FACTION_VANCELLAN,
        REPUTATION_HOSTILE,
    )?;

    // FTU relationships (disliked by everybody, hostile to Vancellan)
    create_mutual_standing(
        dsl,
        FACTION_FREE_TRADE_UNION,
        FACTION_REDIAR_FEDERATION,
        REPUTATION_DISLIKED,
    )?;
    create_mutual_standing(
        dsl,
        FACTION_FREE_TRADE_UNION,
        FACTION_VANCELLAN,
        REPUTATION_HOSTILE,
    )?;

    // RF relationships (disliked by everyone except neutral to IWA, hostile to Vancellan)
    create_mutual_standing(
        dsl,
        FACTION_REDIAR_FEDERATION,
        FACTION_VANCELLAN,
        REPUTATION_HOSTILE,
    )?;

    Ok(())
}

/// Helper function to create mutual faction standings (both directions)
fn create_mutual_standing<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    faction_one: u32,
    faction_two: u32,
    reputation: i32,
) -> Result<(), String> {
    // Create standing from faction_one to faction_two
    dsl.create_faction_standing(CreateFactionStanding {
        faction_one_id: FactionId::new(faction_one),
        faction_two_id: FactionId::new(faction_two),
        reputation_score: reputation,
    })?;

    // Create standing from faction_two to faction_one (mutual)
    dsl.create_faction_standing(CreateFactionStanding {
        faction_one_id: FactionId::new(faction_two),
        faction_two_id: FactionId::new(faction_one),
        reputation_score: reputation,
    })?;

    Ok(())
}
