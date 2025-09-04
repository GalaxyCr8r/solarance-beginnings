use spacetimedb::ReducerContext;
use spacetimedsl::dsl;

use super::{StellarObject, StellarObjectTransformInternal, StellarObjectVelocity};
use crate::types::stellarobjects::GetSobjInternalTransformRowOptionById;

impl StellarObject {
    pub fn distance_squared(
        &self,
        ctx: &ReducerContext,
        target: &StellarObject,
    ) -> Result<f32, String> {
        let dsl = dsl(ctx);

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
