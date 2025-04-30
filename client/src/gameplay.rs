

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

////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////
/// Main Loop
////////////////////////////////////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////

async fn gameplay(token : Option<String>) {
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
