use std::{ collections::HashMap, f32::consts::PI, sync::mpsc, thread::{ self, JoinHandle }, time::Duration };

use egui::{ Align2, Color32, RichText };
use macroquad::{ math::Vec2, prelude::*, time, ui };
use secs::World;
use macroquad::miniquad::conf::Conf;

mod module_bindings;
use module_bindings::*;
use spacetimedb_sdk::{ DbContext, Table };
mod stdb_client_helper;
mod shader;
pub mod oidc_auth_helper;
use dotenv::dotenv;
use stdb_client_helper::{get_transform, register_callbacks};

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

fn draw_ship(transform: &StellarObjectTransform) { // TODO: Refactor this out of main.rs
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

fn render_system(_world: &World, game_state: &mut GameState) { // TODO: Refactor this out of main.rs
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

#[macroquad::main("Solarance:Beginnings")]
async fn main() { // TODO: Refactor most of this stuff out of main.rs
    dotenv().ok();

    let mut client_token_thread: Option<JoinHandle<Result<String, String>>> = None;

    let mut id_token: Option<String> = None;
    let mut error_message: Option<String> = None;

    let mut break_the_loop = false;

    set_pc_assets_folder("assets");
    let rings: Vec<Texture2D> = vec![
        load_texture("Ring1.png").await.expect("Couldn't load file"),
        load_texture("Ring2.png").await.expect("Couldn't load file"),
        load_texture("Ring3.png").await.expect("Couldn't load file")
    ];

    loop { 
        if client_token_thread.as_ref().is_some_and(|thread| { thread.is_finished() }) {
            let thread = client_token_thread.take().unwrap();
            if thread.is_finished() {
                match thread.join() {
                    Ok(token_result) =>
                        match token_result {
                            Ok(token) => {
                                id_token = Some(token.to_string());
                            }
                            Err(error) => {
                                error_message = Some(error.to_string());
                            }
                        }
                    Err(_join_error) => {
                        error_message = Some("Unknown join error of login thread!".to_string());
                    }
                }
            }
        }

        clear_background(BLACK);

        for i in 0..3 {
            let rot = match i {
                0 => { f32::cos((time::get_time() as f32) * 0.5) }
                1 => { f32::sin(time::get_time() as f32) }
                _ => { f32::cos((-time::get_time() as f32) * 0.25) }
            };
            draw_texture_ex(
                rings[i],
                screen_width() / 2.0 - rings[i].width() / 4.0,
                screen_height() / 2.0 - rings[i].height() / 4.0,
                Color {
                    a: 0.5,
                    ..Color::from_hex(0xbedaff)
                },
                DrawTextureParams {
                    rotation: rot * PI,
                    dest_size: Some(Vec2::new(rings[i].width() / 2.0, rings[i].height() / 2.0)),
                    ..Default::default()
                }
            );
        }

        egui_macroquad::ui(|egui_ctx| {
            egui::Window
                ::new("Solarance:Beginnings")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .anchor(Align2::CENTER_CENTER, egui::Vec2::new(0.0, 0.0))
                .show(egui_ctx, |ui| {
                    if !client_token_thread.as_ref().is_none() {
                        ui.label("Waiting on handshake...");
                    }
                    else {
                        if error_message.is_some() {
                            ui.label(
                                RichText::new(
                                    format!("ERROR: {}", error_message.as_ref().unwrap().to_string())
                                ).color(Color32::RED)
                            );
                        }
    
                        if !id_token.is_some() && ui.button(RichText::new("\n      Login      \n").size(32.0)).clicked() {
                            info!("CLICKED!");
                            client_token_thread = Some(
                                thread::spawn(|| { oidc_auth_helper::get_client_token() })
                            );
                        }
                        if id_token.is_some() && ui.button(RichText::new("\n      PLAY!      \n").size(32.0)).clicked() {
                            info!("CLICKED!");
                            break_the_loop = true;
                        }
                    }
                });
        });

        egui_macroquad::draw();
        next_frame().await;

        if break_the_loop {
            break;
        }
    }

    gameplay(id_token).await;
}

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Debug Gameplay Window
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

fn debug_window(game_state: &mut GameState) { // TODO: Refactor this out of main.rs
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

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

pub async fn gameplay(token : Option<String>) { // TODO: Refactor this out of main.rs
    // DB Connection & ECS World
    let world = World::default();
    let ctx = stdb_client_helper::connect_to_spacetime(token);

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
    info!("Loading textures...");
    set_pc_assets_folder("assets");
    let sun_texture: Texture2D =
        load_texture("stars/star.png").await.expect("Couldn't load file");
    sun_texture.set_filter(FilterMode::Nearest);
    let ship_texture: Texture2D =
        load_texture("ships/lc/phalanx.png").await.expect("Couldn't load file");
    ship_texture.set_filter(FilterMode::Nearest);
    let station_texture: Texture2D =
        load_texture("ships/lc/generic_station.png").await.expect("Couldn't load file");
        station_texture.set_filter(FilterMode::Nearest);
    let bullet_texture: Texture2D =
        load_texture("ships/bullet02.png").await.expect("Couldn't load file");
    bullet_texture.set_filter(FilterMode::Linear);

    build_textures_atlas();
    game_state.textures.insert("star", sun_texture);
    game_state.textures.insert("lc/phalanx", ship_texture);
    game_state.textures.insert("lc/station", station_texture);
    game_state.textures.insert("bullet", bullet_texture);

    // Load starfield shader
    info!("Loading shader...");
    let sf_shader = shader::load_starfield_shader();
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

        debug_window(&mut game_state);

        egui_macroquad::draw();
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
