//! (#203) Sector-change warp: a ~1s background transition so using a jumpgate
//! reads as travel rather than a teleport.
//!
//! No new render path was needed. `bg_camera` is already anchored to the
//! player's sector — `gameplay.rs` offsets it by `sector.x/y * 100.0` every
//! frame — so a jump *snaps* the background to the destination's slice of the
//! star system. Easing that snap makes `render_star_system` itself slide the
//! planets past: the motion the issue asks for falls out of the renderer we
//! already have, and the warp owns nothing but a decaying offset.
//!
//! Every sector change also gets a blue glow that fades over the same second.
//! Sectors are only reachable through jumpgates, so the glow is the cue that
//! a jump happened at all — the slide alone is too easy to miss when the
//! nearest planet is far away.
//!
//! Inter-system jumps can't ease. The destination has a different `system_id`,
//! so the background swaps to an entirely different set of objects and there is
//! no continuous path between the two camera positions to slide along. Those
//! get the glow with no slide, which covers the swap — still distinguishable
//! from an intra-system jump, which glows *and* slides.
//!
//! The in-sector pass is held for the duration (`hides_in_sector`). The ship
//! is already at the destination gate the frame the jump lands, so drawing it
//! over a still-moving background reads as arriving before the travel
//! finished. Holding it costs a ~1s window where the player is physically in
//! the sector but can't see it — accepted deliberately for the effect.
//!
//! Cosmetic only — nothing here feeds back into server state.

use macroquad::prelude::*;
use spacetimedb_sdk::DbContext;

use crate::server::bindings::SectorTableAccess;

use super::{star_system::render_star_system, GameState};
use crate::gameplay::state::SectorWarp;
use crate::stdb::utils::get_player_ship;

/// Matches the ~1s the issue asks for.
const WARP_DURATION: f64 = 1.0;

/// Sector coordinates are scaled by this to place the star-system background.
/// Same multiplier `gameplay.rs` uses when it anchors `bg_camera` to the
/// sector — if that changes, this has to change with it or the slide will
/// travel the wrong distance.
const SECTOR_BG_SCALE: f32 = 100.0;

/// Peak opacity of the blue glow, at the instant the jump lands. Short of 1.0
/// so the destination stays faintly visible through it rather than the screen
/// going flat blue. Turn this down if the glow reads as too heavy — it's the
/// only knob the effect's intensity has.
const FLASH_PEAK_ALPHA: f32 = 0.85;

/// Render the star-system background, easing through a sector change (#203).
///
/// Replaces a bare `set_camera(bg_camera)` + `render_star_system()` — it owns
/// the whole background pass, warp or no warp, so callers never sequence the
/// two themselves.
pub fn render_background(game_state: &mut GameState) {
    let flash = advance(game_state);

    set_camera(&game_state.bg_camera);
    render_star_system(game_state);

    // Screen space, and deliberately *before* the in-sector pass: the issue
    // scopes this to the background, so ships and stations stay readable
    // through the flash.
    if let Some(alpha) = flash {
        info!("Alpha screen rect!!!");
        set_default_camera();
        draw_rectangle(
            game_state.camera.target.x,
            game_state.camera.target.y,
            screen_width(),
            screen_height(),
            Color::new(0.25, 0.45, 1.0, alpha),
        );
    }
}

/// Whether the in-sector pass should be held this frame (#203).
///
/// Only meaningful *after* `render_background` has run for the frame — that's
/// what starts and expires the warp. Callers render the background first
/// regardless, so this reads the current frame's answer, not the last one's.
pub fn hides_in_sector(game_state: &GameState) -> bool {
    game_state.warp.active.is_some()
}

/// Start a warp when the player's sector changed, then apply whichever warp is
/// running to `bg_camera`.
///
/// Returns the blue glow's alpha while a warp is running, `None` otherwise.
fn advance(game_state: &mut GameState) -> Option<f32> {
    let now = get_time();

    // `None` while docked / out-of-play. Leave `last_sector` alone in that
    // case: docking and undocking in one sector must not read as a jump.
    if let Some(sector_id) = get_player_ship(game_state.ctx).map(|ship| ship.sector_id) {
        if let Some(previous) = game_state.warp.last_sector.replace(sector_id) {
            if previous != sector_id {
                game_state.warp.active = begin(game_state, previous, sector_id, now);
            }
        }
    }

    let warp = game_state.warp.active?;
    let progress = ((now - warp.start_time) / WARP_DURATION) as f32;

    if progress >= 1.0 {
        game_state.warp.active = None;
        return None;
    }

    // ponytail: `gameplay.rs` reassigns `bg_camera.target` outright each frame,
    // so this offset can't accumulate — except while docked, where it skips
    // that reassignment entirely. Docking inside the 1s window would drift the
    // background behind the out-of-play panel that covers it. Cache the
    // pre-warp target if that ever becomes visible.
    game_state.bg_camera.target += warp.from_offset * offset_decay(progress);

    Some(flash_alpha(progress))
}

/// Build the warp for a jump from `previous` to `current`.
///
/// `None` when either sector row is missing from the cache — no warp is a far
/// better failure than a slide across a garbage distance.
fn begin(
    game_state: &GameState,
    previous: u64,
    current: u64,
    now: f64,
) -> Option<SectorWarp> {
    let sectors = game_state.ctx.db().sector();
    let (Some(from), Some(to)) = (sectors.id().find(&previous), sectors.id().find(&current))
    else {
        warn!("Sector warp skipped: no cached sector row for jump #{previous} -> #{current}");
        return None;
    };

    let inter_system = from.system_id != to.system_id;

    // One line per jump, naming both systems and the branch taken. Every jump
    // glows; only an intra-system one also slides. When something looks wrong
    // the first question is "which branch did it pick?" — answer it in the log
    // rather than by reading the sector table by hand.
    info!(
        "Sector warp: #{previous} (system {}) -> #{current} (system {}) — {}",
        from.system_id,
        to.system_id,
        if inter_system {
            "inter-system, glow only"
        } else {
            "intra-system, glow + slide"
        }
    );

    Some(SectorWarp {
        start_time: now,
        from_offset: start_offset(
            Vec2::new(from.x, from.y),
            Vec2::new(to.x, to.y),
            inter_system,
        ),
    })
}

/// Background offset a warp starts from: the previous sector's position
/// relative to the new one, so the camera begins where the player *was* and
/// slides to where they are.
///
/// Zero for an inter-system jump — see the module docs.
fn start_offset(from: Vec2, to: Vec2, inter_system: bool) -> Vec2 {
    if inter_system {
        Vec2::ZERO
    } else {
        (from - to) * SECTOR_BG_SCALE
    }
}

/// Opacity of the blue glow at `progress` through the warp.
///
/// Linear to zero so the sector arrives clean instead of the glow popping off
/// on the final frame.
fn flash_alpha(progress: f32) -> f32 {
    (1.0 - progress) * FLASH_PEAK_ALPHA
}

/// How much of the starting offset survives at `progress` through the warp.
///
/// Squared so the slide decelerates into the destination instead of travelling
/// at a constant rate and stopping dead on the last frame.
fn offset_decay(progress: f32) -> f32 {
    (1.0 - progress).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intra_system_slides_from_the_previous_sector() {
        let offset = start_offset(Vec2::new(3.0, 1.0), Vec2::new(1.0, 1.0), false);
        assert_eq!(offset, Vec2::new(2.0 * SECTOR_BG_SCALE, 0.0));
    }

    #[test]
    fn inter_system_does_not_slide() {
        let offset = start_offset(Vec2::new(3.0, 1.0), Vec2::new(1.0, 1.0), true);
        assert_eq!(offset, Vec2::ZERO);
    }

    #[test]
    fn glow_fades_to_nothing_by_the_end() {
        assert_eq!(flash_alpha(0.0), FLASH_PEAK_ALPHA);
        assert_eq!(flash_alpha(1.0), 0.0);
        assert!(flash_alpha(0.25) > flash_alpha(0.75));
    }

    #[test]
    fn offset_decays_to_nothing_by_the_end() {
        assert_eq!(offset_decay(0.0), 1.0);
        assert_eq!(offset_decay(1.0), 0.0);
        // Monotonic, so the background never doubles back mid-slide.
        assert!(offset_decay(0.25) > offset_decay(0.5));
        assert!(offset_decay(0.5) > offset_decay(0.75));
    }
}
