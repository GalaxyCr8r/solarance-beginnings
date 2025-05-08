use std::{ collections::HashMap, f32::consts::PI, sync::mpsc::{self, Sender} };

use egui::{ Align2, Context, ScrollArea, TextStyle };
use macroquad::{ math::Vec2, prelude::*, ui };

use super::module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };
use super::stdb::connector::connect_to_spacetime;

use crate::{shader::*, stdb::utils::*};


struct GameState<'a> {
    paused: bool,
    done: bool,
    ctx: &'a DbConnection,
    textures: HashMap<&'static str, Texture2D>,
    camera: Camera2D
}

// #[derive(Default)]
// struct SolShip {
//     sobj_id: u64,
// }

// #[derive(Default)]
// struct Transform {
//     x: f32,
//     y: f32,
//     rotation_radians: f32,
// }


/// Register all the callbacks our app will use to respond to database events.
pub fn register_callbacks(ctx: &DbConnection, global_chat_channel: Sender<GlobalChat>) {
    ctx.db.stellar_object().on_insert( |_ec, sobj| {
        println!("Stellar Object Inserted: {:?}", sobj);
        // world.spawn((
        //     SolShip {
        //         sobj_id: sobj.id,
        //         ..Default::default()
        //     },
        //     Transform::default(),
        // ));
    });

    ctx.db.global_chat().on_insert(move |_ec, message| {
        print!("{}: {}", message.identity.to_abbreviated_hex().to_string(), message.message);
        let _ = global_chat_channel.send(message.clone());
    });
}

fn draw_ship(transform: &StellarObjectTransform, game_state: &mut GameState) {
    let position = transform.to_vec2();
    let forward = Vec2::from_angle(transform.rotation_radians) * 16.0;

    let forward_pos = position + forward * 2.0;

    draw_line(position.x, position.y, forward_pos.x, forward_pos.y, 2.0, RED);

    let string = format!("Sobj{}", transform.sobj_id.to_string());
    draw_text_ex(&string, position.x, position.y, TextParams {
        font_size: 16,
        rotation: transform.rotation_radians,
        color: WHITE,
        ..TextParams::default()
    });

    let tex = game_state.textures["lc.phalanx"];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: transform.rotation_radians,
            ..DrawTextureParams::default()
        }
    );
}

fn render_system(game_state: &mut GameState) {
    if game_state.paused {
        let text = "PAUSED";
        let font_size = 100.0;
        let text_width = measure_text(text, None, font_size as u16, 1.0).width;
        let (x, y) = ((screen_width() - text_width) / 2.0, screen_height() / 2.0);

        draw_text(text, x, y, font_size, RED);

        return;
    }

    // TODO: Figure out how to get the player ship's position at the beginning so we can offset everything drawn by it.

    let sun = game_state.textures["star"];
    draw_texture(sun, sun.width() * -0.5, sun.height() * -0.5, WHITE);

    for object in game_state.ctx.db.stellar_object().iter() {
        if let Ok(transform) = get_transform(game_state.ctx, object.id) {
            draw_ship(&transform, game_state);
        }
    }

    draw_line(0.0, 0.0, game_state.camera.target.x, game_state.camera.target.y, 3.0, RED);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Debug Gameplay Window
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

fn debug_window(egui_ctx: &Context, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = &game_state.ctx;

    egui::Window
        ::new("Solarance:Beginnings")
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .anchor(Align2::LEFT_TOP, egui::Vec2::new(-5.0, 5.0))
        .show(egui_ctx, |ui| {
            match ctx.db.player().identity().find(&ctx.identity()) {
                Some(player) => {
                    ui.heading(format!("Player: {}", player.username));
                    if let Some(controlled) = player.get_controlled_stellar_object(&ctx) {
                        match get_transform(&ctx, controlled)
                        {
                            Ok(transform) => {
                                ui.label(
                                    format!(
                                        "Ship: {}, {}",
                                        transform.x.to_string(),
                                        transform.y.to_string()
                                    )
                                );
                            }
                            _ => {
                                ui.label("Ship: unknown");
                            }
                        }
                    } else {
                        ui.label("Ship: None");
                    }
                }
                None => {
                    ui.heading("Player: unknown");
                    ui.label(format!("ID: {}", ctx.identity()));
                    if ui.button("Create Player & Ship").clicked() {
                        let _ = ctx.reducers.create_player_controlled_ship(ctx.identity(), "Galaxy".to_string());
                        info!("Creating player and ship");
                    }
                }
            }

            for object in ctx.db.stellar_object().iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("- Ship #{}", object.id));

                    match get_transform(&ctx, object.id) {
                        Ok(transform) => {
                            let string = format!(
                                "Position: {}, {}",
                                transform.x.to_string(),
                                transform.y.to_string()
                            );
                            ui.label(string);
                            return;
                        }
                        _ => {
                            ui.label("Position: n/a");
                        }
                    }
                });
            }

            for player_controlled in ctx.db.player_controlled_stellar_object().iter() {
                ui.label(format!(" - Player Controlled Obj #{} in Sec#{}", player_controlled.controlled_sobj_id, player_controlled.sector_id));
            }

            ui.label(
                format!(
                    "Camera: {}, {}",
                    game_state.camera.target.x.to_string(),
                    game_state.camera.target.y.to_string()
                )
            );

            ui.add_space(8.0);
            if ui.button("  Quit  ").clicked() {
                game_state.done = true;
            }
        })
}

fn chat_window(egui_ctx: &Context, ctx: &DbConnection, global_chat_channel: &[GlobalChat], text: &mut String) -> Option<egui::InnerResponse<Option<()>>> {
    //let mut chat_text = String::new();
    let mut selected_tab = 0; // TODO Add this to a new `chat_window_state` struct

    egui::Window
        ::new("Chat Window")
        .min_width(256.0)
        .title_bar(false)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut selected_tab, 0, "Global");
                ui.selectable_value(&mut selected_tab, 1, "Sector");
                ui.selectable_value(&mut selected_tab, 2, "Alliance");
                ui.selectable_value(&mut selected_tab,3, "Faction");
            });
            ui.separator();
            let text_style = TextStyle::Body;
            let row_height = ui.text_style_height(&text_style);
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .stick_to_bottom(true)
                .show_rows(
                ui,
                row_height,
                global_chat_channel.len(),
                |ui, _row_range| {
                    for message in global_chat_channel {
                        ui.label(format!("[{}]: {}", get_username(ctx, &message.identity), message.message));
                    }
                },
            );

            ui.text_edit_singleline(text);
        })
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn gameplay(textures : HashMap<&'static str, Texture2D>, token : Option<String>) {
    // DB Connection & ECS World
    let ctx = connect_to_spacetime(token);

    let (global_chat_transmitter, global_chat_receiver) = mpsc::channel::<GlobalChat>();
    let mut global_chat_channel = Vec::<GlobalChat>::new();

    let mut game_state = GameState {
        paused: false,
        done: false,
        ctx: &ctx,
        textures: textures,
        camera: Camera2D::from_display_rect(Rect { x: 0.0, y: 0.0, w: screen_width(), h: screen_height() })
        // camera: Camera2D{
        //     rotation: 0.0,
        //     zoom: Vec2 { x: 1.0, y: 1.0 },
        //     target: Vec2::ZERO,
        //     offset: Vec2::ZERO,
        //     render_target: None,
        //     viewport: None,
        // }
    };
    let mut text: String = String::new();

    let _receiver = register_callbacks(&ctx, global_chat_transmitter);

    // Load starfield shader
    info!("Loading shader...");
    let sf_shader = load_starfield_shader();
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Linear);

    // Setup Panic Handler
    set_panic_handler(|msg, _backtrace| async move {
        loop {
            clear_background(RED);
            ui::root_ui().label(None, &msg);
            next_frame().await;
        }
    });

    set_camera(&game_state.camera);

    loop {
        clear_background(WHITE);

        game_state.camera.target = get_player_transform_vec2(&ctx, Vec2::ZERO);// - Vec2 { x: screen_width()/4.0, y: screen_height()/4.0 };
        set_camera(&game_state.camera);
        
        apply_shader_to_screen(
            render_target,
            sf_shader,
            game_state.camera.target,
            game_state.camera.target * 0.0001337
        );

        // run all parallel and sequential systems
        render_system(&mut game_state);

        egui_macroquad::ui(|egui_ctx| {  
            chat_window(&egui_ctx, &ctx, &global_chat_channel, &mut text);
            debug_window(&egui_ctx, &mut game_state);
        });

        egui_macroquad::draw();
        next_frame().await;

        let _ = control_player_ship(&ctx);

        if let Ok(message) = global_chat_receiver.try_recv() {
            global_chat_channel.push(message);
        }

        if game_state.done {
            let _ = ctx.disconnect();
            break;
        }
    }
}


fn control_player_ship(ctx: &DbConnection) -> Result<(), String> {
    let controlled_entity_id = get_player_sobj_id(ctx);
    if controlled_entity_id.is_none() {
        return Err("Player doesn't control a stellar object yet!".to_string());
    }
    let mut velocity = ctx.db
        .stellar_object_velocity()
        .sobj_id()
        .find(&controlled_entity_id.unwrap())
        .ok_or("Player's controlled object doesn't have a velocity table entry!")?;

    let vel = velocity.to_vec2();
    let mut changed = false;
    if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
        velocity.rotation_radians += PI * 0.01337;
        changed = true;
    }
    if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
        velocity.rotation_radians -= PI * 0.01337;
        changed = true;
    }
    if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
        velocity = velocity.from_vec2(vel * 0.75);
        changed = true;
    }
    if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
        info!("Orig. Velocity: {}, {}", velocity.x, velocity.y);
        let transform = get_transform(&ctx, velocity.sobj_id)?;
        velocity = velocity.from_vec2(Vec2::from_angle(transform.rotation_radians) * 200.0);
        changed = true;
        info!("Updated Velocity: {}, {}", velocity.x, velocity.y);
    }

    if !changed {
        return Ok(());
    }

    ctx.reducers
        .update_stellar_object_velocity(velocity)
        .or_else(|err| Err(err.to_string()))
}
