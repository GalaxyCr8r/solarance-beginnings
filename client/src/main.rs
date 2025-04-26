use std::{ collections::HashMap, f32::consts::PI, sync::mpsc::{ self, Receiver }, time::Duration };

use egui::Align2;
use macroquad::{ math::Vec2, prelude::*, ui::{ self } };
use secs::World;
use macroquad::miniquad::conf::Conf;

mod module_bindings;
use module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };
use stdb_client_helper::get_transform;
mod stdb_client_helper;
mod shader;

struct GameState<'a> {
    paused: bool,
    done: bool,
    ctx: &'a DbConnection,
    textures: HashMap<&'static str, Texture2D>,
}

#[derive(Default)]
struct SolShip {
    sobj_id: u64,
}

#[derive(Default)]
struct Transform {
    x: f32,
    y: f32,
    rotation_radians: f32,
}

// struct Velocity {
//     x: f32,
//     y: f32,
// }

// struct Sprite {
//     //shape: Shape,
//     width: f32,
//     height: f32,
// }

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

fn draw_ship(transform: &StellarObjectTransform) {
    let position = Vec2::new(transform.x, transform.y);
    let forward = Vec2::from_angle(transform.rotation_radians) * 16.0;
    let right = Vec2::from_angle(transform.rotation_radians + PI * 0.75) * 16.0;
    let left = Vec2::from_angle(transform.rotation_radians - PI * 0.75) * 16.0;

    let forward_pos = position + forward * 2.0;

    draw_line(position.x, position.y, forward_pos.x, forward_pos.y, 2.0, RED);
    draw_triangle(position + forward, position + right, position + left, WHITE);

    let string = format!("Sobj{}", transform.sobj_id.to_string());
    draw_text_ex(&string, transform.x, transform.y, TextParams {
        font_size: 16,
        rotation: transform.rotation_radians,
        color: WHITE,
        ..TextParams::default()
    });
}

fn render_system(_world: &World, game_state: &mut GameState) {
    if game_state.paused {
        let text = "PAUSED";
        let font_size = 100.0;
        let text_width = measure_text(text, None, font_size as u16, 1.0).width;
        let (x, y) = ((screen_width() - text_width) / 2.0, screen_height() / 2.0);

        draw_text(text, x, y, font_size, RED);

        return;
    }

    for object in game_state.ctx.db.stellar_object().iter() {
        match game_state.ctx.db.stellar_object_hi_res().sobj_id().find(&object.id) {
            Some(hirez) => {
                draw_ship(&hirez);
                let tex = game_state.textures["lc/phalanx"];
                draw_texture_ex(
                    tex,
                    hirez.x - tex.width() * 0.5,
                    hirez.y - tex.height() * 0.5,
                    WHITE,
                    DrawTextureParams {
                        rotation: hirez.rotation_radians,
                        ..DrawTextureParams::default()
                    }
                );
            }
            None => {
                match game_state.ctx.db.stellar_object_low_res().sobj_id().find(&object.id) {
                    Some(lorez) => draw_ship(&lorez),
                    None => (),
                }
            }
        }
    }

    draw_text("Solarance", 0.0, 64.0, 150.0, RED);
    draw_line(0.0, 0.0, 32.0, 150.0, 3.0, GOLD);

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

fn debug_window(game_state: &mut GameState) {
    let ctx = &game_state.ctx;

    egui_macroquad::ui(|egui_ctx| {
        egui::Window
            ::new("Solarance:Beginnings")
            .resizable(false)
            .collapsible(false)
            .movable(false)
            .anchor(Align2::RIGHT_TOP, egui::Vec2::new(-5.0, 5.0))
            .show(egui_ctx, |ui| {
                match ctx.db.player().identity().find(&ctx.identity()) {
                    Some(player) => {
                        ui.heading(format!("Player: {}", player.username));
                        if player.controlled_entity_id.is_some() {
                            match get_transform(&ctx, player.controlled_entity_id.unwrap())
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
                            let _ = ctx.reducers.create_player_controlled_ship(ctx.identity());
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

                ui.add_space(8.0);
                if ui.button("  Quit  ").clicked() {
                    game_state.done = true;
                }
            });
    });
}

// fn on_stellar_object_inserted(_event: &EventContext, sobj: &StellarObject) {
//     println!("Stellar Object Inserted: {:?}", sobj);
//     unsafe {
//         let world_lock = WORLD.lock();
//         if world_lock.is_err() {
//             println!("Failed to get world lock");
//             return;
//         }
//         let world = world_lock.unwrap();

//         world.spawn((
//             StellarObject {
//                 id: sobj.id,
//                 kind: sobj.kind,
//                 sector_id: 0
//             },
//             StellarObjectTransform {
//                 sobj_id: sobj.id,
//                 x: 0.0, y: 0.0,
//                 rotation_radians: 0.0
//             }
//         ));
//     }
// }

/// Register all the callbacks our app will use to respond to database events.
fn register_callbacks(world: &World, ctx: &DbConnection) -> Receiver<StellarObject> {
    let (transmitter, receiver) = mpsc::channel();

    ctx.db.stellar_object().on_insert(move |_ec, row| {
        match transmitter.send(row.clone()) {
            Err(error) => println!("ERROR : {:?}", error),
            _ => (),
        }
    });

    return receiver;

    // When a new user joins, print a notification.
    // ctx.db.user().on_insert(on_user_inserted);

    // // When a user's status changes, print a notification.
    // ctx.db.user().on_update(on_user_updated);

    // // When a new message is received, print it.
    // ctx.db.message().on_insert(on_message_inserted);

    // // When we fail to set our name, print a warning.
    // ctx.reducers.on_set_name(on_name_set);

    // // When we fail to send a message, print a warning.
    // ctx.reducers.on_send_message(on_message_sent);
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
fn _window_conf() -> Conf {
    Conf {
        window_title: "egui with macroquad & secs".to_owned(),
        //high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main("secs_macroquad")]
async fn main() {
    // DB Connection & ECS World
    let world = World::default();
    let ctx = stdb_client_helper::connect_to_spacetime(None);

    let scheduler = secs::Scheduler::default();
    let mut game_state = GameState {
        paused: false,
        done: false,
        ctx: &ctx,
        textures: HashMap::new(),
    };

    let receiver = register_callbacks(&world, &ctx);

    scheduler.register(render_system);

    // Load asset textures
    set_pc_assets_folder("assets");
    let ship_texture: Texture2D =
        load_texture("ships/lc/phalanx.png").await.expect("Couldn't load file");
    ship_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D =
        load_texture("ships/bullet02.png").await.expect("Couldn't load file");
    bullet_texture.set_filter(FilterMode::Linear);

    build_textures_atlas();
    game_state.textures.insert("lc/phalanx", ship_texture);
    game_state.textures.insert("bullet", bullet_texture);

    // Load starfield shader
    let sf_shader = shader::load_starfield_shader();
    let render_target = render_target(320, 150);
    render_target.texture.set_filter(FilterMode::Nearest);

    // Setup Panic Handler
    set_panic_handler(|msg, _backtrace| async move {
        loop {
            clear_background(RED);
            ui::root_ui().label(None, &msg);
            next_frame().await;
        }
    });

    let mut tmp_angle = 0.0;
    loop {
        clear_background(WHITE);
        //clear_background(BLACK);
        shader::apply_shader_to_screen(
            render_target,
            sf_shader,
            Vec2::from_angle(tmp_angle) * 0.01337
        );
        tmp_angle += 0.01337;

        // run all parallel and sequential systems
        scheduler.run(&world, &mut game_state);

        egui_macroquad::draw();

        debug_window(&mut game_state);

        next_frame().await;

        let _ = control_player_ship(&ctx);

        match receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(sobj) => {
                println!("Stellar Object Inserted: {:?}", sobj);
                world.spawn((
                    SolShip {
                        sobj_id: sobj.id,
                        ..Default::default()
                    },
                    Transform::default(),
                ));
            }
            Err(err) =>
                match err {
                    mpsc::RecvTimeoutError::Timeout => (),
                    mpsc::RecvTimeoutError::Disconnected => {
                        println!("ERROR : {:?}", err);
                    }
                }
        }

        if game_state.done {
            let _ = ctx.disconnect();
            break;
        }
    }
}

fn control_player_ship(ctx: &DbConnection) -> Result<(), String> {
    let player = ctx.db.player().identity().find(&ctx.identity()).ok_or("Could not find player.")?;
    let controlled_entity_id = player.controlled_entity_id.ok_or(
        "Player doesn't control a stellar object yet!"
    )?;
    let mut velocity = ctx.db
        .stellar_object_velocity()
        .sobj_id()
        .find(&controlled_entity_id)
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
