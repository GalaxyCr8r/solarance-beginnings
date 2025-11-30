use super::*;

#[dsl(plural_name = hospital_modules)]
#[table(name = hospital_module, public)]
pub struct Hospital {
    #[primary_key]
    #[use_wrapper(path = StationModuleId)]
    /// FK to StationModule
    id: u64,

    pub medical_bay_capacity: u16, // Max players/NPCs that can be treated simultaneously
    pub healing_effectiveness_modifier: f32, // Base 1.0, affected by upgrades/staff
    /// If morale boost is sector-wide and distinct from station morale.
    pub sector_morale_boost_value: Option<i16>,
    // TODO: Add medical equipment and medical supplies requirements for this station to work at peak efficiency.
}
