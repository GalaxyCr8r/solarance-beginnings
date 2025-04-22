use std::{f32::consts::PI, sync::{
    atomic::{AtomicBool, Ordering}, Arc
}};

use egui::{Align2};
use macroquad::{math::Vec2, prelude::*, rand, ui::{self, root_ui}};
use secs::World;
use macroquad::miniquad::{self, conf::Conf};

mod module_bindings;
use module_bindings::*;
use spacetimedb_sdk::{DbContext, Table};
mod stdb;

struct GameState {
    paused: bool,
}

struct Position {
    x: f32,
    y: f32,
}

struct Velocity {
    x: f32,
    y: f32,
}

struct Sprite {
    //shape: Shape,
    width: f32,
    height: f32,
}

// struct Powerup {
//     active: bool,
//     width: f32,
//     height: f32,
// }

// struct Score {
//     value: i32,
// }

// enum Shape {
//     Square,
//     Circle,
// }

fn move_system(world: &World, game_state: &mut GameState) {
    if game_state.paused {
        return;
    }

    // world.query(|_entity, pos: &mut Position, vel: &mut Velocity| {
    //     vel.x = 0.;
    //     vel.y = 0.;

    //     if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
    //         vel.x = 2.;
    //     }
    //     if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
    //         vel.x = -2.;
    //     }
    //     if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
    //         vel.y = 2.;
    //     }
    //     if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
    //         vel.y = -2.;
    //     }

    //     pos.x += vel.x;
    //     pos.y += vel.y;
    // });
}

fn collision_system(world: &World, _: &mut GameState) {
    // world.query(
    //     |_, player_center: &Position, player: &mut Sprite, player_score: &mut Score| {
    //         world.query(|_, powerup_center: &Position, powerup: &mut Powerup| {
    //             if powerup.active
    //                 && (powerup_center.x - player_center.x).abs()
    //                     < (powerup.width * 0.5) + (player.width * 0.5)
    //                 && (powerup_center.y - player_center.y).abs()
    //                     < (powerup.height * 0.5) + (player.height * 0.5)
    //             {
    //                 powerup.active = false;

    //                 player.shape = match player.shape {
    //                     Shape::Square => Shape::Circle,
    //                     Shape::Circle => Shape::Square,
    //                 };

    //                 player_score.value += 1;
    //                 player.width += 3.;
    //                 player.height += 3.;
    //             }
    //         });
    //     },
    // )
}

fn draw_ship(transform: StellarObjectTransform) {
    let position = vec2(transform.x, transform.y);
    let forward = Vec2::from_angle(transform.rotation_radians) * 16.0;
    let right = Vec2::from_angle(transform.rotation_radians + (PI * 0.75)) * 16.0;
    let left = Vec2::from_angle(transform.rotation_radians - (PI * 0.75)) * 16.0;

    let forward_pos = position+(forward*2.0);
    
    draw_line(position.x, position.y, forward_pos.x, forward_pos.y, 2.0, RED);
    draw_triangle(position+forward, position+right, position+left, BLACK);
    
    let string = format!("Sobj{}", transform.sobj_id.to_string());
    draw_text_ex(&string, transform.x, transform.y, TextParams { 
        font_size: 16,
        rotation: transform.rotation_radians, 
        color: BLACK,
        ..TextParams::default()
     });
}

fn render_system(world: &World, game_state: &mut GameState) {
    if game_state.paused {
        let text = "PAUSED";
        let font_size = 100.;
        let text_width = measure_text(text, None, font_size as u16, 1.).width;
        let (x, y) = ((screen_width() - text_width) / 2., screen_height() / 2.);

        draw_text(text, x, y, font_size, RED);

        return;
    }

    draw_text("Solarance", 0., 64., 150., RED);
    draw_line(0., 0., 32., 150., 3., GOLD);

    // world.query(|_, pos: &Position, sprite: &Sprite| match sprite.shape {
    //     Shape::Square => draw_rectangle(
    //         pos.x - (sprite.width * 0.5),
    //         pos.y - (sprite.width * 0.5),
    //         sprite.width,
    //         sprite.height,
    //         ORANGE,
    //     ),
    //     Shape::Circle => draw_circle(pos.x, pos.y, sprite.width * 0.5, PURPLE),
    // });

    // world.query(|_, powerup: &Powerup, pos: &Position| {
    //     if powerup.active {
    //         draw_rectangle(
    //             pos.x - (powerup.width * 0.5),
    //             pos.y - (powerup.width * 0.5),
    //             powerup.width,
    //             powerup.height,
    //             RED,
    //         );
    //     }
    // });

    // world.query(|_, score: &Score| {
    //     root_ui().label(None, &format!("Player Score: {}", score.value));
    // });
}


fn _window_conf() -> Conf {
    Conf {
        window_title: "egui with macroquad & secs".to_owned(),
        //high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main("secs_macroquad")]
async fn main() {
    let world = World::default();
    //let _egui_demo_windows = egui_demo_lib::DemoWindows::default();
    //let mut current_map_view = MapView::GalacticSystem;

    let ctx = stdb::connect_to_spacetime(None);

    let scheduler = secs::Scheduler::default();
    let mut game_state = GameState { paused: false };

    //scheduler.register(move_system);
    //scheduler.register(collision_system);
    scheduler.register(render_system);

    set_panic_handler(|msg, _backtrace| async move {
        loop {
            clear_background(RED);
            ui::root_ui().label(None, &msg);
            next_frame().await;
         }
    });

    loop {
        clear_background(SKYBLUE);

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("Solarance:Beginnings")
            .resizable(false)
            .collapsible(false)
            .movable(false)
            .anchor(Align2::RIGHT_TOP, egui::Vec2::new(-5.0, 5.0))
            .show(egui_ctx, |ui| {
                ui.heading("Test Header");
                ui.label("Test text");
                
                for object in ctx.db.stellar_object().iter() {
                    ui.horizontal(|ui| {
                        ui.label("Ship!!");
                        match ctx.db.stellar_object_hi_res().sobj_id().find(&object.id) {
                            so_transform => {
                                if so_transform.is_some() {
                                    let position = so_transform.unwrap();
                                    let string = format!("Hi-Rez: {}, {}", position.x.to_string(), position.y.to_string());
                                    ui.label(string);
                                } else {
                                    let lr = ctx.db.stellar_object_low_res().sobj_id().find(&object.id);
                                    if lr.is_none() {
                                        ui.label("Low-rez transform n/a");
                                        return;
                                    }
                                    let position = lr.unwrap();
                                    let string = format!("Lo-Rez: {}, {}", position.x.to_string(), position.y.to_string());
                                    ui.label(string);
                                }
                                return;
                            },
                            _ => {
                                ui.label("Object transform n/a");
                            }
                        }
                    });
                }
            });
        });

        for object in ctx.db.stellar_object().iter() {
            match ctx.db.stellar_object_hi_res().sobj_id().find(&object.id) {
                Some(hirez) => draw_ship(hirez),
                None => {
                    match ctx.db.stellar_object_low_res().sobj_id().find(&object.id) {
                        Some(lorez) => draw_ship(lorez),
                        None => (),
                    }
                },
            }
            
        }

        // run all parallel and sequential systems
        scheduler.run(&world, &mut game_state);
        egui_macroquad::draw();

        next_frame().await;
    }
}