use spacetimedb::{table, Identity, SpacetimeType};
use spacetimedsl::*;

/// What kind of stellar object it is. OBE with the advent of asteroid/ship/station tables?
#[derive(SpacetimeType, Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum StellarObjectKinds {
    Ship,
    Asteroid,
    CargoCrate,
    Station,
    JumpGate,
}

/// An object that exists inside a sector.
#[dsl(plural_name = stellar_objects, method(update = true))]
#[table(name = stellar_object, public)]
pub struct StellarObject {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_velocity)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_internal_transform)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_hi_res_transform)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_low_res_transform)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_turn_left_controller)]
    #[referenced_by(path = crate::tables::stellarobjects, table = sobj_player_window)]
    #[referenced_by(path = crate::tables::asteroids, table = asteroid)]
    #[referenced_by(path = crate::tables::ships, table = ship)]
    #[referenced_by(path = crate::tables::stations, table = station)]
    #[referenced_by(path = crate::tables::jumpgates, table = jump_gate)]
    #[referenced_by(path = crate::tables::items, table = cargo_crate)]
    #[referenced_by(path = crate::tables::ships, table = ship_movement_controller)]
    #[referenced_by(path = crate::tables::npcs, table = npc_ship_controller)]
    id: u64,

    #[index(btree)]
    pub kind: StellarObjectKinds,

    #[index(btree)]
    #[use_wrapper(crate::tables::sectors::SectorId)]
    #[foreign_key(path = crate::tables::sectors, table = sector, column = id, on_delete = Delete)]
    /// FK to SectorLocation
    pub sector_id: u64,
}

/// The current velocity of a stellar object.
#[dsl(plural_name = sobj_velocities, method(update = true))]
#[table(name = sobj_velocity, public)]
#[derive(Default)]
pub struct StellarObjectVelocity {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,

    pub auto_dampen: Option<f32>,
}

/// The current exact transform of a stellar object. Used to populate low/high resolution tables.
#[dsl(plural_name = sobj_internal_transforms, method(update = true))]
#[table(name = sobj_internal_transform)]
#[derive(Default)]
pub struct StellarObjectTransformInternal {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

/// The position of a stellar object that has a high rate of updates
#[dsl(plural_name = sobj_hi_res_transforms, method(update = true))]
#[table(name = sobj_hi_res_transform, public)]
#[derive(Default)]
pub struct StellarObjectTransformHiRes {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_low_res_transforms, method(update = true))]
#[table(name = sobj_low_res_transform, public)]
#[derive(Default)]
pub struct StellarObjectTransformLowRes {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,

    pub x: f32,
    pub y: f32,
    pub rotation_radians: f32,
}

#[dsl(plural_name = sobj_turn_left_controllers, method(update = true))]
#[table(name = sobj_turn_left_controller)]
pub struct StellarObjectControllerTurnLeft {
    #[primary_key]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    id: u64,
}

impl StellarObjectControllerTurnLeft {
    pub fn id(&self) -> u64 {
        self.id
    }
}

#[dsl(plural_name = sobj_player_windows, method(update = true))]
#[table(name = sobj_player_window, public)]
pub struct StellarObjectPlayerWindow {
    #[primary_key]
    #[use_wrapper(crate::players::PlayerId)]
    #[foreign_key(path = crate::players, table = player, column = id, on_delete = Delete)]
    id: Identity,

    #[unique]
    #[use_wrapper(StellarObjectId)]
    #[foreign_key(path = crate::tables::stellarobjects, table = stellar_object, column = id, on_delete = Delete)]
    /// FK to StellarObject
    pub sobj_id: u64,

    pub window: f32, // How big of a rectangular window should be
    pub margin: f32, // How much space can you move within the window before recalculating
    // Top Left
    pub tl_x: f32,
    pub tl_y: f32,
    // Bottom Right
    pub br_x: f32,
    pub br_y: f32,
}

impl StellarObjectPlayerWindow {
    pub fn tl_x(&self) -> f32 {
        self.tl_x
    }
    pub fn tl_y(&self) -> f32 {
        self.tl_y
    }
    pub fn br_x(&self) -> f32 {
        self.br_x
    }
    pub fn br_y(&self) -> f32 {
        self.br_y
    }
    pub fn margin(&self) -> f32 {
        self.margin
    }
    pub fn window(&self) -> f32 {
        self.window
    }
}

//////////////////////////////////////////////////////////////
// Utilities

pub fn same_sector_from_ids<T: spacetimedsl::WriteContext>(
    dsl: &DSL<T>,
    id1: &StellarObjectId,
    id2: &StellarObjectId,
) -> bool {
    if let Ok(sobj1) = dsl.get_stellar_object_by_id(id1) {
        if let Ok(sobj2) = dsl.get_stellar_object_by_id(id2) {
            return sobj1.sector_id == sobj2.sector_id;
        }
    }
    false
}

//////////////////////////////////////////////////////////////
// Impls
//////////////////////////////////////////////////////////////

impl StellarObject {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn distance_squared<T: spacetimedsl::WriteContext>(
        &self,
        dsl: &DSL<T>,
        target: &StellarObject,
    ) -> Result<f32, String> {
        let transform = dsl.get_sobj_internal_transform_by_id(self)?;
        let target_transform = dsl.get_sobj_internal_transform_by_id(target)?;

        Ok(transform
            .to_vec2()
            .distance_squared(target_transform.to_vec2()))
    }

    // Deprecated due to STDSL's fk deletion rules
    // /// Attempts to smartly delete everything related to this stellar object.
    // pub fn delete(
}

impl StellarObjectVelocity {
    // pub fn new(x: f32, y: f32) -> Self {
    //     Self { x, y }
    // }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectVelocity {
        StellarObjectVelocity {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }
}

impl StellarObjectTransformInternal {
    pub fn new(x: f32, y: f32, rotation_radians: f32) -> Self {
        Self {
            id: 0,
            x,
            y,
            rotation_radians,
        }
    }

    pub fn to_vec2(&self) -> glam::Vec2 {
        glam::Vec2 {
            x: self.x,
            y: self.y,
        }
    }

    pub fn from_vec2(&self, vec: glam::Vec2) -> StellarObjectTransformInternal {
        StellarObjectTransformInternal {
            x: vec.x,
            y: vec.y,
            ..*self
        }
    }

    pub fn from_xy(&self, x: f32, y: f32) -> StellarObjectTransformInternal {
        StellarObjectTransformInternal {
            x: x,
            y: y,
            ..*self
        }
    }
}
