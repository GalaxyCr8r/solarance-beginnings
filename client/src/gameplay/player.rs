use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::server::bindings::*;

use crate::stdb::utils::*;

use super::state::GameState;

pub fn control_player_ship(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if game_state.chat_window.has_focus || ctx.try_identity().is_none() {
        return Ok(());
    }

    let forward  = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
    let backward = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);
    let left     = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
    let right    = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

    let new_flags = (forward, backward, left, right);
    if game_state.movement_flags != new_flags {
        game_state.movement_flags = new_flags;
        let _ = ctx.reducers.update_ship_movement_controller(forward, backward, left, right);
    }

    Ok(())
}

/// How wide a cone `[E]` sweeps for a target (#241). Two degrees is tight on
/// purpose: it should feel like aiming, not like a nearest-object search.
const TARGETING_CONE_RADIANS: f32 = 2.0 * std::f32::consts::PI / 180.0;

/// Which object `[E]` should pick.
pub enum TargetMode {
    /// Nearest object by angle to the ship's heading, within the cone.
    /// Nothing in the cone clears the target — the ship is pointed at empty
    /// space, so it should read as empty.
    Heading,
    /// Nearest object by distance, regardless of where the ship points.
    /// Shift-`[E]`; this was the only behavior before #241.
    Nearest,
}

/// Point the targeting computer at something, or clear it.
///
/// Owns the whole intent, including the toggle-off on re-targeting the same
/// object, so the caller is one line and the two modes can't drift apart.
pub fn acquire_target(ctx: &DbConnection, game_state: &mut GameState, mode: TargetMode) {
    let Some(pose) = get_player_pose(ctx) else {
        return; // Docked — nothing to target from.
    };
    let candidates = targetable_objects(ctx, pose.sobj_id);

    match mode {
        TargetMode::Heading => {
            // Deselects when the cone is empty, which is the point: the
            // readout should match where the nose is pointed.
            game_state.current_target_sobj_id = best_in_cone(
                pose.rotation_radians,
                pose.pos,
                &candidates,
                TARGETING_CONE_RADIANS,
            );
        }
        TargetMode::Nearest => {
            let nearest = nearest_by_distance(pose.pos, &candidates);
            // Re-pressing on the same object clears it, as before #241.
            game_state.current_target_sobj_id =
                if nearest.is_some() && nearest == game_state.current_target_sobj_id {
                    None
                } else {
                    nearest
                };
        }
    }

    match game_state.current_target_sobj_id {
        Some(id) => {
            info!("Targeted stellar object: {}", id);
        }
        None => {
            info!("No target in range.");
        }
    }
}

/// Every object in the player's sector except their own ship, with the
/// position the HUD is drawing it at.
fn targetable_objects(ctx: &DbConnection, player_sobj_id: u64) -> Vec<(u64, glam::Vec2)> {
    let Some(player_sobj) = ctx.db.stellar_object().id().find(&player_sobj_id) else {
        return Vec::new();
    };

    ctx.db()
        .stellar_object()
        .iter()
        .filter(|sobj| sobj.id != player_sobj_id && sobj.sector_id == player_sobj.sector_id)
        .filter_map(|sobj| get_transform(ctx, sobj.id).ok().map(|t| (sobj.id, t.to_vec2())))
        .collect()
}

/// Absolute angle between where the ship points and where `target` sits, in
/// radians, wrapped into [0, π] so the far side of the circle isn't "close".
fn angle_off_heading(heading_radians: f32, from: glam::Vec2, target: glam::Vec2) -> f32 {
    let bearing = (target - from).to_angle();
    let delta = (bearing - heading_radians).rem_euclid(std::f32::consts::TAU);
    delta.min(std::f32::consts::TAU - delta)
}

/// The candidate best lined up with the heading, or `None` if the cone is
/// empty. Ties on angle go to the nearer object — two things on the same
/// bearing means you meant the one in front.
fn best_in_cone(
    heading_radians: f32,
    from: glam::Vec2,
    candidates: &[(u64, glam::Vec2)],
    cone_radians: f32,
) -> Option<u64> {
    candidates
        .iter()
        .filter(|(_, pos)| *pos != from)
        .map(|(id, pos)| (*id, angle_off_heading(heading_radians, from, *pos), from.distance(*pos)))
        .filter(|(_, off, _)| *off <= cone_radians)
        .min_by(|a, b| {
            a.1.total_cmp(&b.1).then(a.2.total_cmp(&b.2))
        })
        .map(|(id, _, _)| id)
}

/// Closest candidate by straight-line distance.
fn nearest_by_distance(from: glam::Vec2, candidates: &[(u64, glam::Vec2)]) -> Option<u64> {
    candidates
        .iter()
        .min_by(|a, b| {
            from.distance_squared(a.1)
                .total_cmp(&from.distance_squared(b.1))
        })
        .map(|(id, _)| *id)
}

/// Start mining the current target, or stop an in-progress beam.
///
/// The `[X]` button and the `[X]` hotkey both land here (#218) so the two can't
/// disagree about when mining is possible. Mining state is read back from the
/// server's beam row, never a local flag (#141).
pub fn toggle_mining_beam(ctx: &DbConnection, game_state: &mut GameState) -> Result<(), String> {
    if is_player_mining(ctx) {
        ctx.reducers
            .stop_mining_asteroid()
            .map_err(|e| format!("Failed to stop mining: {e}"))
    } else {
        let target = get_current_target(ctx, &mut game_state.current_target_sobj_id)
            .ok_or("No target selected to mine.")?;
        if target.kind != StellarObjectKinds::Asteroid {
            return Err(format!(
                "Target is a {:?}, not an asteroid — nothing to mine.",
                target.kind
            ));
        }
        ctx.reducers
            .try_mining_asteroid(StellarObjectId { value: target.id })
            .map_err(|e| format!("Failed to start mining asteroid {}: {e}", target.id))
    }
}

/// What the `[C]` key does right now, given where the ship is and what it has
/// targeted. `None` means the key is inert — nothing to dock with or jump to.
///
/// Both the button label and the keypress read this, so the label can never
/// promise an action the key won't take (#218).
pub fn docking_action(ctx: &DbConnection, game_state: &mut GameState) -> Option<DockingAction> {
    let identity = ctx.try_identity()?;
    let ship = ctx.db().ship().iter().find(|s| s.player_id == identity)?;

    match ship.location {
        ShipLocation::Station => Some(DockingAction::Undock(ship)),
        ShipLocation::Sector => {
            let target = get_current_target(ctx, &mut game_state.current_target_sobj_id)?;
            match target.kind {
                StellarObjectKinds::Station => Some(DockingAction::Dock(target.id)),
                StellarObjectKinds::JumpGate => Some(DockingAction::Jump(target.id)),
                _ => None,
            }
        }
        _ => None,
    }
}

/// Where the `[C]` key sends the player. Server-side distance / energy gating
/// still applies — this only routes the intent.
pub enum DockingAction {
    Dock(u64),
    Jump(u64),
    Undock(Ship),
}

impl DockingAction {
    /// Verb for the button face, so `[C]`'s label follows the binding.
    pub fn verb(&self) -> &'static str {
        match self {
            DockingAction::Dock(_) => "Dock",
            DockingAction::Jump(_) => "Jump",
            DockingAction::Undock(_) => "Undock",
        }
    }

    pub fn perform(self, ctx: &DbConnection) -> Result<(), String> {
        match self {
            DockingAction::Dock(sobj_id) => ctx
                .reducers
                .dock_ship(sobj_id)
                .map_err(|e| format!("Failed to dock at station sobj {sobj_id}: {e}")),
            DockingAction::Jump(sobj_id) => ctx
                .reducers
                .use_jumpgate(sobj_id)
                .map_err(|e| format!("Failed to use jumpgate sobj {sobj_id}: {e}")),
            DockingAction::Undock(ship) => {
                let ship_id = ship.id;
                ctx.reducers
                    .undock_ship(ship)
                    .map_err(|e| format!("Failed to undock ship {ship_id}: {e}"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{angle_off_heading, best_in_cone, nearest_by_distance, TARGETING_CONE_RADIANS};
    use macroquad::math::Vec2;

    const EAST: f32 = 0.0;
    const NORTH: f32 = std::f32::consts::FRAC_PI_2;

    fn degrees(d: f32) -> f32 {
        d * std::f32::consts::PI / 180.0
    }

    /// Bearing and heading are both absolute angles, so the difference has to
    /// wrap — otherwise a target 1° clockwise of due east reads as 359° off
    /// and never enters the cone.
    #[test]
    fn angle_off_heading_wraps_the_short_way() {
        let origin = Vec2::ZERO;
        assert!(angle_off_heading(EAST, origin, Vec2::new(100.0, 0.0)).abs() < 1e-5);

        // Just clockwise of due east: 1° off, not 359°.
        let just_below = Vec2::from_angle(degrees(-1.0)) * 100.0;
        let off = angle_off_heading(EAST, origin, just_below);
        assert!(
            (off - degrees(1.0)).abs() < 1e-4,
            "expected ~1°, got {}°",
            off * 180.0 / std::f32::consts::PI
        );

        // Directly behind is the furthest anything can be: π, never more.
        let behind = angle_off_heading(EAST, origin, Vec2::new(-100.0, 0.0));
        assert!((behind - std::f32::consts::PI).abs() < 1e-4);
    }

    #[test]
    fn the_cone_is_two_degrees_wide_on_each_side() {
        let origin = Vec2::ZERO;
        let inside = (1u64, Vec2::from_angle(degrees(1.5)) * 500.0);
        let outside = (2u64, Vec2::from_angle(degrees(2.5)) * 500.0);

        let picked = best_in_cone(EAST, origin, &[inside, outside], TARGETING_CONE_RADIANS);
        assert_eq!(picked, Some(1), "2.5° off should be outside the cone");
    }

    /// The deselect behavior #241 asks for: nose pointed at nothing means no
    /// target, not "whatever happens to be nearest".
    #[test]
    fn an_empty_cone_selects_nothing() {
        let far_off_axis = (1u64, Vec2::new(0.0, 500.0)); // due north, 90° off
        assert_eq!(
            best_in_cone(EAST, Vec2::ZERO, &[far_off_axis], TARGETING_CONE_RADIANS),
            None
        );
    }

    /// Angle wins over distance — that's what "closest to your direction"
    /// means. A distant object dead ahead beats a near one off to the side.
    #[test]
    fn best_in_cone_prefers_alignment_over_proximity() {
        let dead_ahead_far = (1u64, Vec2::new(9000.0, 0.0));
        let off_axis_near = (2u64, Vec2::from_angle(degrees(1.9)) * 100.0);
        assert_eq!(
            best_in_cone(
                EAST,
                Vec2::ZERO,
                &[off_axis_near, dead_ahead_far],
                TARGETING_CONE_RADIANS
            ),
            Some(1)
        );
    }

    /// Two things on the same bearing: you meant the one in front.
    #[test]
    fn equal_angles_break_toward_the_nearer_object() {
        let near = (1u64, Vec2::new(200.0, 0.0));
        let far = (2u64, Vec2::new(8000.0, 0.0));
        assert_eq!(
            best_in_cone(EAST, Vec2::ZERO, &[far, near], TARGETING_CONE_RADIANS),
            Some(1)
        );
    }

    /// The cone follows the ship, not the world — same candidates, different
    /// heading, different pick.
    #[test]
    fn the_cone_rotates_with_the_ship() {
        let east_target = (1u64, Vec2::new(500.0, 0.0));
        let north_target = (2u64, Vec2::new(0.0, 500.0));
        let candidates = [east_target, north_target];

        assert_eq!(
            best_in_cone(EAST, Vec2::ZERO, &candidates, TARGETING_CONE_RADIANS),
            Some(1)
        );
        assert_eq!(
            best_in_cone(NORTH, Vec2::ZERO, &candidates, TARGETING_CONE_RADIANS),
            Some(2)
        );
    }

    /// Shift-[E]'s behavior: ignores heading entirely.
    #[test]
    fn nearest_by_distance_ignores_heading() {
        let behind_and_near = (1u64, Vec2::new(-100.0, 0.0));
        let ahead_and_far = (2u64, Vec2::new(5000.0, 0.0));
        assert_eq!(
            nearest_by_distance(Vec2::ZERO, &[ahead_and_far, behind_and_near]),
            Some(1)
        );
        assert_eq!(nearest_by_distance(Vec2::ZERO, &[]), None);
    }
}
