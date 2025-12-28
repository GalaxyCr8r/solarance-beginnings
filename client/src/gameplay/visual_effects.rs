//! Visual Effects System
//!
//! This module handles client-side visual effects for combat actions.
//! It listens to VisualEffect database entries created by the server
//! and renders appropriate visual feedback for different effect types.
//!
//! Features:
//! - Weapon fire effects (laser beams)
//! - Missile fire effects (projectile trails)
//! - Explosion effects (expanding circles)
//! - Configurable effect durations
//! - Automatic cleanup of expired effects

use macroquad::prelude::*;
use spacetimedb_sdk::Table;

use super::state::{FiringEffect, GameState};
use crate::module_bindings::{
    self, visual_effect_table::VisualEffectTableAccess, VisualEffectType,
};

/// Configuration for visual effects
pub struct VisualEffectConfig {
    pub weapon_fire_duration: f64,
    pub missile_fire_duration: f64,
    pub explosion_duration: f64,
}

impl Default for VisualEffectConfig {
    fn default() -> Self {
        Self {
            weapon_fire_duration: 0.3,  // Quick laser flash
            missile_fire_duration: 1.0, // Longer missile trail
            explosion_duration: 0.8,    // Medium explosion
        }
    }
}

/// Update visual effects based on database changes and time
pub fn update_visual_effects(game_state: &mut GameState) {
    let current_time = get_time();

    // Handle new visual effects from the database
    for visual_effect in game_state.ctx.db.visual_effect().iter() {
        // Check if we already have this effect
        if !game_state.firing_effects.contains_key(&visual_effect.id) {
            // Determine duration based on effect type
            let config = VisualEffectConfig::default();
            let duration = match visual_effect.effect_type {
                VisualEffectType::WeaponFire => config.weapon_fire_duration,
                VisualEffectType::MissileFire => config.missile_fire_duration,
                VisualEffectType::Explosion => config.explosion_duration,
            };

            // Create new firing effect
            let firing_effect = FiringEffect {
                start_time: current_time,
                duration,
                source_pos: visual_effect.source,
                target_pos: visual_effect.target,
                effect_type: visual_effect.effect_type.clone(),
            };

            game_state
                .firing_effects
                .insert(visual_effect.id, firing_effect);
        }
    }

    // Remove expired effects
    let mut expired_effects = Vec::new();
    for (effect_id, effect) in &game_state.firing_effects {
        if current_time - effect.start_time > effect.duration {
            expired_effects.push(*effect_id);
        }
    }

    for effect_id in expired_effects {
        game_state.firing_effects.remove(&effect_id);
    }
}

/// Render all active visual effects
pub fn render_visual_effects(game_state: &GameState) {
    let current_time = get_time();

    for effect in game_state.firing_effects.values() {
        let elapsed = current_time - effect.start_time;
        let progress = (elapsed / effect.duration) as f32;

        if progress <= 1.0 {
            match effect.effect_type {
                VisualEffectType::WeaponFire => {
                    render_weapon_fire_effect(effect, progress);
                }
                VisualEffectType::MissileFire => {
                    render_missile_fire_effect(effect, progress);
                }
                VisualEffectType::Explosion => {
                    render_explosion_effect(effect, progress);
                }
            }
        }
    }
}

/// Render weapon fire effect (laser beam)
fn render_weapon_fire_effect(effect: &FiringEffect, progress: f32) {
    let alpha = (1.0 - progress * progress) * 255.0; // Fade out with quadratic curve
    let thickness = 3.0 * (1.0 - progress);

    // Draw main beam
    draw_line(
        effect.source_pos.x,
        effect.source_pos.y,
        effect.target_pos.x,
        effect.target_pos.y,
        thickness,
        Color::from_rgba(255, 100, 100, alpha as u8),
    );

    // Draw inner bright core
    draw_line(
        effect.source_pos.x,
        effect.source_pos.y,
        effect.target_pos.x,
        effect.target_pos.y,
        thickness * 0.3,
        Color::from_rgba(255, 255, 255, (alpha * 0.8) as u8),
    );
}

/// Render missile fire effect (projectile trail)
fn render_missile_fire_effect(effect: &FiringEffect, progress: f32) {
    let alpha = (1.0 - progress) * 255.0;

    // Calculate current missile position
    let missile_pos = module_bindings::Vec2 {
        x: effect.source_pos.x + (effect.target_pos.x - effect.source_pos.x) * progress,
        y: effect.source_pos.y + (effect.target_pos.y - effect.source_pos.y) * progress,
    };

    // Draw missile trail
    let trail_length = 20.0;
    let direction_x = effect.target_pos.x - effect.source_pos.x;
    let direction_y = effect.target_pos.y - effect.source_pos.y;
    let length = (direction_x * direction_x + direction_y * direction_y).sqrt();
    let normalized_x = direction_x / length;
    let normalized_y = direction_y / length;
    let trail_start = module_bindings::Vec2 {
        x: missile_pos.x - normalized_x * trail_length,
        y: missile_pos.y - normalized_y * trail_length,
    };

    draw_line(
        trail_start.x,
        trail_start.y,
        missile_pos.x,
        missile_pos.y,
        2.0,
        Color::from_rgba(255, 150, 0, alpha as u8),
    );

    // Draw missile dot
    draw_circle(
        missile_pos.x,
        missile_pos.y,
        2.0,
        Color::from_rgba(255, 255, 0, alpha as u8),
    );
}

/// Render explosion effect (expanding circle)
fn render_explosion_effect(effect: &FiringEffect, progress: f32) {
    let alpha = (1.0 - progress) * 255.0;
    let radius = progress * 30.0; // Expand to 30 pixel radius

    // Draw outer explosion ring
    draw_circle_lines(
        effect.target_pos.x,
        effect.target_pos.y,
        radius,
        3.0,
        Color::from_rgba(255, 100, 0, alpha as u8),
    );

    // Draw inner bright flash (only for first half of animation)
    if progress < 0.5 {
        let flash_alpha = (0.5 - progress) * 2.0 * 255.0;
        draw_circle(
            effect.target_pos.x,
            effect.target_pos.y,
            radius * 0.5,
            Color::from_rgba(255, 255, 255, flash_alpha as u8),
        );
    }
}
