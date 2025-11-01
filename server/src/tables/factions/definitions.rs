use log::info;
use spacetimedb::ReducerContext;
use spacetimedsl::{dsl, Wrapper, DSL};

use super::*;

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

pub fn init(ctx: &ReducerContext) -> Result<(), String> {
    let dsl = dsl(ctx);

    factions(&dsl)?;
    faction_standings(&dsl)?;

    info!("Faction Defs Loaded: {}", dsl.count_of_all_factions());
    Ok(())
}

//////////////////////////////////////////////////////////////
// Utility
//////////////////////////////////////////////////////////////

fn factions(dsl: &DSL) -> Result<(), String> {
    let pc = Some(FactionId::new(FACTION_ALLIANCE_PROCYON));

    // Factionless - neutral faction for players who want no faction affiliation
    dsl.create_faction(
        FACTION_FACTIONLESS,
        None, // This is a standalone faction
        "Factionless",
        "FX",
        "Independent operators who have chosen to remain neutral in galactic politics. Factionless individuals trade freely with all factions but receive no protection or special privileges from any government. They must rely on their own skills and resources to survive in the galaxy.",
        FactionTier::Galactic,
        true, // joinable
        None,
    )?;

    // Lrak Combine - disliked by all other factions (Galactic tier, joinable)
    dsl.create_faction(
        FACTION_LRAK_COMBINE,
        pc.clone(), // This is a standalone faction that has joined the Proycon Compact
        "Lrak Combine",
        "LC",
        "A militaristic faction known for their aggressive expansion and authoritarian rule dependent on their control of humanity's homeworld. The Lrak Combine seeks to dominate through superior firepower and strict hierarchical control.",
        FactionTier::Galactic, // tier
        true, // joinable
        None,
    )?;

    // Independent Worlds Alliance - disliked by Lrak and FTU, neutral to others (Galactic tier, joinable)
    dsl.create_faction(
        FACTION_INDEPENDENT_WORLDS_ALLIANCE,
        pc.clone(), // This is a standalone faction that has joined the Proycon Compact
        "Independent Worlds Alliance",
        "IWA",
        "A loose confederation of independent star systems that value autonomy and self-governance. The IWA formed as a defensive alliance against larger, more aggressive factions.",
        FactionTier::Galactic, // tier
        true, // joinable
        None,
    )?;

    // Free Trade Union - disliked by everybody (Galactic tier, joinable)
    dsl.create_faction(
        FACTION_FREE_TRADE_UNION,
        None, // This is a standalone faction
        "Free Trade Union",
        "FTU",
        "A corporate-dominated faction that prioritizes profit above all else. The FTU's ruthless business practices and exploitation of resources has earned them enemies across the galaxy.",
        FactionTier::Galactic, // tier
        true, // joinable
        None,
    )?;

    // Rediar Federation - neutral to IWA, disliked by everyone else (Galactic tier, joinable)
    dsl.create_faction(
        FACTION_REDIAR_FEDERATION,
        pc.clone(), // This is a standalone faction that has joined the Proycon Compact
        "Rediar Federation",
        "RF",
        "A technocratic republic that values scientific advancement and technological superiority. The Rediar Federation's elitist attitudes and secretive research programs create tension with other factions.",
        FactionTier::Galactic, // tier
        true, // joinable
        None,
    )?;

    // Vancellan - enemies to everyone (Galactic tier, NOT joinable - antagonistic faction)
    dsl.create_faction(
        FACTION_VANCELLAN,
        None, // This is a standalone faction
        "Vancellan",
        "VCN",
        "A mysterious and hostile faction of unknown origin. The Vancellan are xenophobic extremists who view all other factions as threats to be eliminated. Their advanced biotechnology and ruthless tactics make them feared throughout the galaxy.",
        FactionTier::Galactic, // tier
        false, // NOT joinable - main antagonistic force
        None,
    )?;

    // The alliance formed at Procyon - An affliation of factions who are coordinating the counter-attack against the Vancellans.
    dsl.create_faction(
        FACTION_ALLIANCE_PROCYON,
        None, // This is a standalone faction
        "Procyon Compact",
        "PC",
        "The alliance formed at Procyon - An affliation of factions who are coordinating the counter-attack against the Vancellans.",
        FactionTier::Alliance,
        false, // joinable
        None,
    )?;

    Ok(())
}

fn faction_standings(dsl: &DSL) -> Result<(), String> {
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
fn create_mutual_standing(
    dsl: &DSL,
    faction_one: u32,
    faction_two: u32,
    reputation: i32,
) -> Result<(), String> {
    // Create standing from faction_one to faction_two
    dsl.create_faction_standing(
        FactionId { value: faction_one },
        FactionId { value: faction_two },
        reputation,
    )?;

    // Create standing from faction_two to faction_one (mutual)
    dsl.create_faction_standing(
        FactionId { value: faction_two },
        FactionId { value: faction_one },
        reputation,
    )?;

    Ok(())
}
