//! Galaxy Creator UI (issue #34).
//!
//! Two states, switched on whether we hold a live connection with a known
//! identity:
//!   * **Disconnected** → a centered connection dialog (host dropdown + custom
//!     host, database name, owner token).
//!   * **Connected** → admin panels for the three M4 seed operations:
//!       1. Create sector
//!       2. Place construction-site station
//!       3. Connect two sectors with a bidirectional jumpgate
//!
//! Deliberately utilitarian — this ships only to admins/moderators. The right
//! side panel lists current galaxy state so the designer can see the effect of
//! each reducer as the subscription updates stream back in.

use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Identity, Table};

use crate::server::bindings::*;
use crate::stdb::connector;

/// Which well-known host is selected in the dialog (or a custom one).
#[derive(Clone, Copy, PartialEq, Eq)]
enum HostChoice {
    Local,
    Maincloud,
    Custom,
}

/// All six station sizes, in the order shown in the size dropdown.
const STATION_SIZES: [StationSize; 6] = [
    StationSize::Capital,
    StationSize::Large,
    StationSize::Medium,
    StationSize::Small,
    StationSize::Outpost,
    StationSize::Satellite,
];

/// Module keys understood by `admin_place_station`, paired with UI labels.
const STATION_MODULES: [(&str, &str); 6] = [
    ("trading", "Trading port"),
    ("iron_refinery", "Iron refinery"),
    ("ice_refinery", "Ice refinery"),
    ("silicon_refinery", "Silicon refinery"),
    ("solar_array", "Solar array"),
    ("advanced_manufacturing", "Advanced manufacturing"),
];

struct SectorForm {
    system_id: Option<u32>,
    name: String,
    faction_id: Option<u32>,
    security_level: u8,
    sunlight: f32,
    anomalous: f32,
    nebula: f32,
    rare_ore: f32,
    x: f32,
    y: f32,
}

impl Default for SectorForm {
    fn default() -> Self {
        Self {
            system_id: None,
            name: String::new(),
            faction_id: None,
            security_level: 5,
            sunlight: 0.7,
            anomalous: 0.1,
            nebula: 0.1,
            rare_ore: 0.1,
            x: 0.0,
            y: 0.0,
        }
    }
}

struct StationForm {
    sector_id: Option<u64>,
    name: String,
    size: StationSize,
    faction_id: Option<u32>,
    x: f32,
    y: f32,
    /// `true` → place a finished/operational station (`admin_place_station`);
    /// `false` → place an under-construction site (`admin_create_construction_site`).
    finished: bool,
    /// Construction requirements being assembled: (item_id, quantity). Used for
    /// construction sites only.
    requirements: Vec<(u32, u32)>,
    new_req_item: Option<u32>,
    new_req_qty: u32,
    /// Selected module keys for a finished station.
    modules: Vec<String>,
}

impl Default for StationForm {
    fn default() -> Self {
        Self {
            sector_id: None,
            name: String::new(),
            size: StationSize::Small,
            faction_id: None,
            x: 0.0,
            y: 0.0,
            finished: false,
            requirements: Vec::new(),
            new_req_item: None,
            new_req_qty: 100,
            modules: Vec::new(),
        }
    }
}

#[derive(Default)]
struct ConnectForm {
    sector_a: Option<u64>,
    sector_b: Option<u64>,
}

struct AddModuleForm {
    station_id: Option<u64>,
    module_key: String,
}

impl Default for AddModuleForm {
    fn default() -> Self {
        Self {
            station_id: None,
            module_key: STATION_MODULES[0].0.to_string(),
        }
    }
}

/// State for the "send server message" admin panel (#145 increment 2).
struct MessageForm {
    /// `false` → single recipient (`admin_send_direct_server_message`);
    /// `true` → multi-recipient group (`admin_send_direct_server_message_to_group`).
    to_group: bool,
    target: Option<Identity>,
    group_targets: Vec<Identity>,
    severity: MessageSeverity,
    body: String,
}

impl Default for MessageForm {
    fn default() -> Self {
        Self {
            to_group: false,
            target: None,
            group_targets: Vec::new(),
            severity: MessageSeverity::Info,
            body: String::new(),
        }
    }
}

/// Owned snapshot of the galaxy used to populate dropdowns and listings for a
/// single frame, so the egui closure never holds a borrow on the connection's
/// table cache.
#[derive(Default)]
struct GalaxyData {
    systems: Vec<(u32, String)>,
    factions: Vec<(u32, String)>,
    sectors: Vec<(u64, String)>,
    items: Vec<(u32, String)>,
    /// Existing stations as `(id, label)` for the add-module dropdown.
    stations: Vec<(u64, String)>,
    sector_lines: Vec<String>,
    station_lines: Vec<String>,
    gate_lines: Vec<String>,
    /// Read-only live-state snapshot (#145): players and ships (grouped by sector).
    player_lines: Vec<String>,
    ship_lines: Vec<String>,
    /// Players as `(identity, label)` for the message-recipient picker.
    players: Vec<(Identity, String)>,
}

pub struct AdminApp {
    host_choice: HostChoice,
    custom_host: String,
    db_name: String,
    token: String,
    /// Live connection. `Some` while connecting and after; paired with
    /// `connecting_since` to tell "handshaking" from "ready".
    connection: Option<DbConnection>,
    /// `get_time()` when the current connection attempt began; `None` once the
    /// identity is established (i.e. fully connected).
    connecting_since: Option<f64>,
    last_error: Option<String>,

    sector_form: SectorForm,
    station_form: StationForm,
    connect_form: ConnectForm,
    add_module_form: AddModuleForm,
    message_form: MessageForm,
}

impl AdminApp {
    pub fn new() -> Self {
        Self {
            host_choice: HostChoice::Local,
            custom_host: String::new(),
            db_name: connector::DEFAULT_DB_NAME.to_string(),
            token: String::new(),
            connection: None,
            connecting_since: None,
            last_error: None,
            sector_form: SectorForm::default(),
            station_form: StationForm::default(),
            connect_form: ConnectForm::default(),
            add_module_form: AddModuleForm::default(),
            message_form: MessageForm::default(),
        }
    }

    /// Render one frame.
    pub fn draw(&mut self) {
        clear_background(Color::from_hex(0x0b0f1a));

        // Surface any async connection failure and bounce back to the dialog.
        if let Some(err) = connector::take_connect_error() {
            self.last_error = Some(err);
            self.connection = None;
            self.connecting_since = None;
        }

        // Promote "connecting" → "connected" once the identity lands, or time
        // out a stalled attempt.
        if self.connection.is_some() {
            if let Some(started) = self.connecting_since {
                let has_identity = self
                    .connection
                    .as_ref()
                    .map(|c| c.try_identity().is_some())
                    .unwrap_or(false);
                if has_identity {
                    self.connecting_since = None;
                } else if get_time() - started > 12.0 {
                    self.connection = None;
                    self.connecting_since = None;
                    self.last_error =
                        Some("Connection timed out — check the host and token.".to_string());
                }
            }
        }

        let connected = self.connection.is_some() && self.connecting_since.is_none();
        let galaxy = if connected {
            self.connection
                .as_ref()
                .map(gather_galaxy)
                .unwrap_or_default()
        } else {
            GalaxyData::default()
        };

        // Destructure so the egui closure can hold independent borrows of the
        // connection and each form without tripping the borrow checker.
        let AdminApp {
            host_choice,
            custom_host,
            db_name,
            token,
            connection,
            connecting_since,
            last_error,
            sector_form,
            station_form,
            connect_form,
            add_module_form,
            message_form,
        } = self;

        let mut requested_connect = false;
        let mut requested_disconnect = false;

        egui_macroquad::ui(|egui_ctx| {
            if connection.is_some() && connecting_since.is_none() {
                let conn = connection.as_ref().unwrap();
                requested_disconnect = connected_ui(
                    egui_ctx,
                    conn,
                    &galaxy,
                    sector_form,
                    station_form,
                    connect_form,
                    add_module_form,
                    message_form,
                );
            } else {
                requested_connect = connection_dialog(
                    egui_ctx,
                    host_choice,
                    custom_host,
                    db_name,
                    token,
                    connecting_since,
                    last_error,
                );
            }
        });
        egui_macroquad::draw();

        if requested_disconnect {
            self.connection = None;
            self.connecting_since = None;
            connector::log_activity("Disconnected by user.");
        }

        if requested_connect {
            self.begin_connect();
        }
    }

    /// Kick off a connection attempt using the current dialog inputs.
    fn begin_connect(&mut self) {
        let host = match self.host_choice {
            HostChoice::Local => connector::LOCAL_HOST.to_string(),
            HostChoice::Maincloud => connector::MAINCLOUD_HOST.to_string(),
            HostChoice::Custom => self.custom_host.trim().to_string(),
        };
        if host.is_empty() {
            self.last_error = Some("Enter a host to connect to.".to_string());
            return;
        }
        let token = {
            let t = self.token.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        };
        self.last_error = None;
        match connector::connect(&host, self.db_name.trim(), token) {
            Ok(conn) => {
                self.connection = Some(conn);
                self.connecting_since = Some(get_time());
            }
            Err(e) => {
                self.last_error = Some(format!("Failed to start connection: {e}"));
            }
        }
    }
}

/// Collect an owned snapshot of the tables we render this frame.
fn gather_galaxy(conn: &DbConnection) -> GalaxyData {
    let db = conn.db();

    let mut systems: Vec<(u32, String)> = db
        .star_system()
        .iter()
        .map(|s| (s.id, s.name.clone()))
        .collect();
    systems.sort_by_key(|(id, _)| *id);

    let mut factions: Vec<(u32, String)> =
        db.faction().iter().map(|f| (f.id, f.name.clone())).collect();
    factions.sort_by_key(|(id, _)| *id);

    let mut sectors: Vec<(u64, String)> = db
        .sector()
        .iter()
        .map(|s| (s.id, format!("{} (#{}, sys {})", s.name, s.id, s.system_id)))
        .collect();
    sectors.sort_by_key(|(id, _)| *id);

    let mut items: Vec<(u32, String)> = db
        .item_definition()
        .iter()
        .map(|i| (i.id, i.name.clone()))
        .collect();
    items.sort_by_key(|(id, _)| *id);

    let mut sector_lines: Vec<String> = db
        .sector()
        .iter()
        .map(|s| {
            format!(
                "#{} \"{}\"  sys {}  fac {}  sec {}",
                s.id, s.name, s.system_id, s.controlling_faction_id, s.security_level
            )
        })
        .collect();
    sector_lines.sort();

    let mut stations: Vec<(u64, String)> = db
        .station()
        .iter()
        .map(|st| (st.id, format!("{} (#{}, {:?})", st.name, st.id, st.size)))
        .collect();
    stations.sort_by_key(|(id, _)| *id);

    let mut station_lines: Vec<String> = db
        .station()
        .iter()
        .map(|st| {
            // Append a build-progress suffix while a matching construction site
            // is still in progress (id is shared with the station).
            let suffix = match db.station_under_construction().id().find(&st.id) {
                Some(uc) if !uc.is_operational => {
                    format!("  [building {:.0}%]", uc.construction_progress_percentage)
                }
                _ => String::new(),
            };
            format!(
                "#{} {:?} \"{}\"  sector {}  fac {}{}",
                st.id, st.size, st.name, st.sector_id, st.owner_faction_id, suffix
            )
        })
        .collect();
    station_lines.sort();

    // Players: identity, username, faction, online status, credits, last login.
    let faction_name = |fid: u32| {
        factions
            .iter()
            .find(|(id, _)| *id == fid)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| format!("fac {fid}"))
    };
    let mut player_lines: Vec<String> = db
        .player()
        .iter()
        .map(|p| {
            let status = if p.logged_in { "online" } else { "offline" };
            let last = match p.last_login {
                Some(t) => format!("  last {}", t.to_micros_since_unix_epoch()),
                None => "  never logged in".to_string(),
            };
            format!(
                "{}  [{}]  {}  {}  {}cr{}",
                p.username,
                p.id.to_abbreviated_hex(),
                status,
                faction_name(p.faction_id.value),
                p.credits,
                last,
            )
        })
        .collect();
    player_lines.sort();

    // Players as (identity, label) for the message-recipient picker — online
    // status in the label so admins can target logged-in players.
    let mut players: Vec<(Identity, String)> = db
        .player()
        .iter()
        .map(|p| {
            let status = if p.logged_in { "online" } else { "offline" };
            (
                p.id,
                format!("{} [{}] {}", p.username, p.id.to_abbreviated_hex(), status),
            )
        })
        .collect();
    players.sort_by(|a, b| a.1.cmp(&b.1));

    // Ships, grouped by sector (sector_id then ship id).
    let mut ships: Vec<(u64, u64, String)> = db
        .ship()
        .iter()
        .map(|s| {
            (
                s.sector_id,
                s.id,
                format!(
                    "sector {}  ship #{}  {:?}  type {}  owner {}",
                    s.sector_id,
                    s.id,
                    s.location,
                    s.shiptype_id,
                    s.player_id.to_abbreviated_hex(),
                ),
            )
        })
        .collect();
    ships.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
    let ship_lines: Vec<String> = ships.into_iter().map(|(_, _, line)| line).collect();

    let mut gate_lines: Vec<String> = db
        .jump_gate()
        .iter()
        .map(|g| format!("#{}  sector {} → {}", g.id, g.current_sector_id, g.target_sector_id))
        .collect();
    gate_lines.sort();

    GalaxyData {
        systems,
        factions,
        sectors,
        items,
        stations,
        sector_lines,
        station_lines,
        gate_lines,
        player_lines,
        ship_lines,
        players,
    }
}

/// The pre-connection dialog. Returns `true` when the user clicks Connect.
fn connection_dialog(
    egui_ctx: &egui::Context,
    host_choice: &mut HostChoice,
    custom_host: &mut String,
    db_name: &mut String,
    token: &mut String,
    connecting_since: &mut Option<f64>,
    last_error: &mut Option<String>,
) -> bool {
    let connecting = connecting_since.is_some();
    let mut connect_clicked = false;

    egui::CentralPanel::default().show(egui_ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.heading("Solarance — Galaxy Creator");
            ui.label("Privileged admin client. Connect with the module owner's secret token.");
            ui.add_space(16.0);
        });

        egui::Window::new("Connection")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(egui_ctx, |ui| {
                egui::Grid::new("conn_grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Server");
                        ui.horizontal(|ui| {
                            ui.selectable_value(host_choice, HostChoice::Local, "localhost");
                            ui.selectable_value(host_choice, HostChoice::Maincloud, "maincloud");
                            ui.selectable_value(host_choice, HostChoice::Custom, "custom");
                        });
                        ui.end_row();

                        ui.label("Host");
                        match host_choice {
                            HostChoice::Local => {
                                ui.monospace(connector::LOCAL_HOST);
                            }
                            HostChoice::Maincloud => {
                                ui.monospace(connector::MAINCLOUD_HOST);
                            }
                            HostChoice::Custom => {
                                ui.add(
                                    egui::TextEdit::singleline(custom_host)
                                        .hint_text("http://host:3000"),
                                );
                            }
                        }
                        ui.end_row();

                        ui.label("Database");
                        ui.add(egui::TextEdit::singleline(db_name));
                        ui.end_row();

                        ui.label("Owner token");
                        ui.add(
                            egui::TextEdit::singleline(token)
                                .password(true)
                                .hint_text("paste `spacetime login show --token`"),
                        );
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(!connecting, |ui| {
                        if ui.button("Connect").clicked() {
                            connect_clicked = true;
                        }
                    });
                    if connecting {
                        ui.spinner();
                        ui.label("Connecting…");
                    }
                });

                if let Some(err) = last_error {
                    ui.add_space(6.0);
                    ui.colored_label(egui::Color32::LIGHT_RED, err.as_str());
                }

                ui.add_space(6.0);
                ui.collapsing("Tip: getting the owner token", |ui| {
                    ui.label("Run the project task `show-token`, or:");
                    ui.monospace("spacetime login show --token");
                    ui.label(
                        "The token's identity must match the server's `try_server_only` allow-list.",
                    );
                });
            });
    });

    connect_clicked
}

/// The connected admin UI. Returns `true` if the user asked to disconnect.
fn connected_ui(
    egui_ctx: &egui::Context,
    conn: &DbConnection,
    galaxy: &GalaxyData,
    sector_form: &mut SectorForm,
    station_form: &mut StationForm,
    connect_form: &mut ConnectForm,
    add_module_form: &mut AddModuleForm,
    message_form: &mut MessageForm,
) -> bool {
    let mut disconnect = false;

    egui::TopBottomPanel::top("conn_bar").show(egui_ctx, |ui| {
        ui.horizontal(|ui| {
            let id = conn
                .try_identity()
                .map(|i| i.to_abbreviated_hex().to_string())
                .unwrap_or_else(|| "?".to_string());
            ui.strong("Galaxy Creator");
            ui.separator();
            ui.label(format!("identity {id}"));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Disconnect").clicked() {
                    disconnect = true;
                }
            });
        });
    });

    egui::SidePanel::right("galaxy_panel")
        .resizable(true)
        .default_width(360.0)
        .show(egui_ctx, |ui| {
            ui.heading("Current galaxy");
            ui.label(format!(
                "{} systems · {} sectors · {} stations · {} gates · {} players · {} ships",
                galaxy.systems.len(),
                galaxy.sectors.len(),
                galaxy.station_lines.len(),
                galaxy.gate_lines.len(),
                galaxy.player_lines.len(),
                galaxy.ship_lines.len(),
            ));
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                section_list(ui, "Players", &galaxy.player_lines);
                section_list(ui, "Ships", &galaxy.ship_lines);
                section_list(ui, "Sectors", &galaxy.sector_lines);
                section_list(ui, "Stations", &galaxy.station_lines);
                section_list(ui, "Jumpgates", &galaxy.gate_lines);
                ui.add_space(8.0);
                ui.separator();
                ui.collapsing("Activity log", |ui| {
                    let log = connector::activity_log_snapshot();
                    for line in log.iter().rev().take(40) {
                        ui.monospace(line);
                    }
                });
            });
        });

    egui::CentralPanel::default().show(egui_ctx, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // ①, ②, etc. aren't shown properly in egui
            egui::CollapsingHeader::new("1: Create sector")
                .show(ui, |ui| sector_panel(ui, conn, sector_form, galaxy));

            egui::CollapsingHeader::new("2: Place station")
                .show(ui, |ui| station_panel(ui, conn, station_form, galaxy));

            egui::CollapsingHeader::new("3: Connect sectors with a jumpgate")
                .show(ui, |ui| connect_panel(ui, conn, connect_form, galaxy));

            egui::CollapsingHeader::new("4: Add module to existing station")
                .show(ui, |ui| add_module_panel(ui, conn, add_module_form, galaxy));

            egui::CollapsingHeader::new("5: Send server message")
                .show(ui, |ui| message_panel(ui, conn, message_form, galaxy));
        });
    });

    disconnect
}

fn section_list(ui: &mut egui::Ui, title: &str, lines: &[String]) {
    ui.collapsing(format!("{title} ({})", lines.len()), |ui| {
        if lines.is_empty() {
            ui.weak("(none)");
        }
        for line in lines {
            ui.monospace(line);
        }
    });
}

/// Label for a `(id, name)` selection, or a placeholder when nothing is chosen.
fn selected_label<T: Copy + PartialEq + std::fmt::Display>(
    selected: Option<T>,
    options: &[(T, String)],
) -> String {
    match selected {
        Some(id) => options
            .iter()
            .find(|(oid, _)| *oid == id)
            .map(|(_, name)| name.clone())
            .unwrap_or_else(|| format!("#{id}")),
        None => "— select —".to_string(),
    }
}

fn u32_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    selected: &mut Option<u32>,
    options: &[(u32, String)],
) {
    ui.label(label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_label(*selected, options))
        .show_ui(ui, |ui| {
            for (id, name) in options {
                ui.selectable_value(selected, Some(*id), format!("{name} (#{id})"));
            }
        });
    ui.end_row();
}

fn u64_combo(
    ui: &mut egui::Ui,
    id_salt: &str,
    label: &str,
    selected: &mut Option<u64>,
    options: &[(u64, String)],
) {
    ui.label(label);
    egui::ComboBox::from_id_salt(id_salt)
        .selected_text(selected_label(*selected, options))
        .show_ui(ui, |ui| {
            for (id, name) in options {
                ui.selectable_value(selected, Some(*id), name);
            }
        });
    ui.end_row();
}

fn sector_panel(
    ui: &mut egui::Ui,
    conn: &DbConnection,
    form: &mut SectorForm,
    galaxy: &GalaxyData,
) {
    egui::Grid::new("sector_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            u32_combo(ui, "sector_system", "Star system", &mut form.system_id, &galaxy.systems);

            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut form.name).hint_text("e.g. Theta Sector"));
            ui.end_row();

            u32_combo(
                ui,
                "sector_faction",
                "Controlling faction",
                &mut form.faction_id,
                &galaxy.factions,
            );

            ui.label("Security level (0–10)");
            ui.add(egui::DragValue::new(&mut form.security_level).range(0..=10));
            ui.end_row();

            ui.label("Sunlight");
            ui.add(egui::DragValue::new(&mut form.sunlight).speed(0.01).range(0.0..=1.0));
            ui.end_row();

            ui.label("Anomalous");
            ui.add(egui::DragValue::new(&mut form.anomalous).speed(0.01).range(0.0..=1.0));
            ui.end_row();

            ui.label("Nebula");
            ui.add(egui::DragValue::new(&mut form.nebula).speed(0.01).range(0.0..=1.0));
            ui.end_row();

            ui.label("Rare ore");
            ui.add(egui::DragValue::new(&mut form.rare_ore).speed(0.01).range(0.0..=1.0));
            ui.end_row();

            ui.label("System position (x, y)");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut form.x).speed(0.1).prefix("x "));
                ui.add(egui::DragValue::new(&mut form.y).speed(0.1).prefix("y "));
            });
            ui.end_row();
        });

    let valid = form.system_id.is_some() && form.faction_id.is_some() && !form.name.trim().is_empty();
    ui.add_space(4.0);
    ui.add_enabled_ui(valid, |ui| {
        if ui.button("Create sector").clicked() {
            let name = form.name.trim().to_string();
            let label = format!("create_sector \"{name}\"");
            let res = conn.reducers.admin_create_sector_then(
                form.system_id.unwrap(),
                name,
                form.faction_id.unwrap(),
                form.security_level,
                form.sunlight,
                form.anomalous,
                form.nebula,
                form.rare_ore,
                form.x,
                form.y,
                move |_ctx, result| log_reducer_result(label, result),
            );
            log_send_error(res);
        }
    });
}

fn station_panel(
    ui: &mut egui::Ui,
    conn: &DbConnection,
    form: &mut StationForm,
    galaxy: &GalaxyData,
) {
    ui.horizontal(|ui| {
        ui.label("Mode:");
        ui.selectable_value(&mut form.finished, false, "Construction site");
        ui.selectable_value(&mut form.finished, true, "Finished station");
    });
    ui.weak(if form.finished {
        "Places an operational station directly, optionally fitted with modules."
    } else {
        "Places an under-construction site that players contribute resources to."
    });
    ui.add_space(4.0);

    egui::Grid::new("station_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            u64_combo(ui, "station_sector", "Sector", &mut form.sector_id, &galaxy.sectors);

            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut form.name).hint_text("e.g. Theta Outpost"));
            ui.end_row();

            ui.label("Size");
            egui::ComboBox::from_id_salt("station_size")
                .selected_text(format!("{:?}", form.size))
                .show_ui(ui, |ui| {
                    for size in STATION_SIZES {
                        ui.selectable_value(&mut form.size, size.clone(), format!("{size:?}"));
                    }
                });
            ui.end_row();

            u32_combo(
                ui,
                "station_faction",
                "Owner faction",
                &mut form.faction_id,
                &galaxy.factions,
            );

            ui.label("World position (x, y)");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut form.x).speed(1.0).prefix("x "));
                ui.add(egui::DragValue::new(&mut form.y).speed(1.0).prefix("y "));
            });
            ui.end_row();
        });

    ui.add_space(4.0);
    if form.finished {
        station_modules_editor(ui, form);
    } else {
        station_requirements_editor(ui, form, galaxy);
    }

    let valid =
        form.sector_id.is_some() && form.faction_id.is_some() && !form.name.trim().is_empty();
    ui.add_space(4.0);
    ui.add_enabled_ui(valid, |ui| {
        let button_text = if form.finished {
            "Place finished station"
        } else {
            "Place construction site"
        };
        if ui.button(button_text).clicked() {
            let name = form.name.trim().to_string();
            if form.finished {
                let label = format!("place_station \"{name}\"");
                let modules = form.modules.clone();
                let res = conn.reducers.admin_place_station_then(
                    form.sector_id.unwrap(),
                    name,
                    form.size.clone(),
                    form.faction_id.unwrap(),
                    form.x,
                    form.y,
                    modules,
                    move |_ctx, result| log_reducer_result(label, result),
                );
                log_send_error(res);
            } else {
                let label = format!("construction_site \"{name}\"");
                let requirements = form
                    .requirements
                    .iter()
                    .map(|(item_id, quantity)| ResourceAmount {
                        resource_item_id: *item_id,
                        quantity: *quantity,
                    })
                    .collect();
                let res = conn.reducers.admin_create_construction_site_then(
                    form.sector_id.unwrap(),
                    name,
                    form.size.clone(),
                    form.faction_id.unwrap(),
                    form.x,
                    form.y,
                    requirements,
                    move |_ctx, result| log_reducer_result(label, result),
                );
                log_send_error(res);
            }
        }
    });
}

/// Checkbox list of modules to fit onto a finished station.
fn station_modules_editor(ui: &mut egui::Ui, form: &mut StationForm) {
    ui.label("Modules");
    for (key, label) in STATION_MODULES {
        let mut checked = form.modules.iter().any(|m| m == key);
        if ui.checkbox(&mut checked, label).changed() {
            if checked {
                form.modules.push(key.to_string());
            } else {
                form.modules.retain(|m| m != key);
            }
        }
    }
}

/// Editable list of construction resource requirements for a construction site.
fn station_requirements_editor(ui: &mut egui::Ui, form: &mut StationForm, galaxy: &GalaxyData) {
    ui.label("Construction requirements");
    for i in (0..form.requirements.len()).rev() {
        let (item_id, qty) = form.requirements[i];
        let item_name = galaxy
            .items
            .iter()
            .find(|(id, _)| *id == item_id)
            .map(|(_, n)| n.clone())
            .unwrap_or_else(|| format!("item #{item_id}"));
        ui.horizontal(|ui| {
            ui.monospace(format!("{qty} × {item_name}"));
            if ui.small_button("✕").clicked() {
                form.requirements.remove(i);
            }
        });
    }
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("req_item")
            .selected_text(selected_label(form.new_req_item, &galaxy.items))
            .show_ui(ui, |ui| {
                for (id, name) in &galaxy.items {
                    ui.selectable_value(&mut form.new_req_item, Some(*id), format!("{name} (#{id})"));
                }
            });
        ui.add(egui::DragValue::new(&mut form.new_req_qty).speed(1.0).range(1..=u32::MAX));
        ui.add_enabled_ui(form.new_req_item.is_some(), |ui| {
            if ui.button("Add").clicked() {
                if let Some(item) = form.new_req_item {
                    form.requirements.push((item, form.new_req_qty.max(1)));
                }
            }
        });
    });
}

fn connect_panel(
    ui: &mut egui::Ui,
    conn: &DbConnection,
    form: &mut ConnectForm,
    galaxy: &GalaxyData,
) {
    egui::Grid::new("connect_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            u64_combo(ui, "connect_a", "Sector A", &mut form.sector_a, &galaxy.sectors);
            u64_combo(ui, "connect_b", "Sector B", &mut form.sector_b, &galaxy.sectors);
        });

    let valid = match (form.sector_a, form.sector_b) {
        (Some(a), Some(b)) => a != b,
        _ => false,
    };
    ui.add_space(4.0);
    ui.add_enabled_ui(valid, |ui| {
        if ui.button("Connect with jumpgate").clicked() {
            let (a, b) = (form.sector_a.unwrap(), form.sector_b.unwrap());
            let label = format!("connect_sectors {a} <-> {b}");
            let res = conn.reducers.admin_connect_sectors_then(
                a,
                b,
                move |_ctx, result| log_reducer_result(label, result),
            );
            log_send_error(res);
        }
    });
}

fn add_module_panel(
    ui: &mut egui::Ui,
    conn: &DbConnection,
    form: &mut AddModuleForm,
    galaxy: &GalaxyData,
) {
    ui.weak("Fit a module onto an existing station — e.g. a construction site that completed empty.");
    egui::Grid::new("add_module_grid")
        .num_columns(2)
        .spacing([12.0, 6.0])
        .show(ui, |ui| {
            u64_combo(
                ui,
                "add_module_station",
                "Station",
                &mut form.station_id,
                &galaxy.stations,
            );

            ui.label("Module");
            let current_label = STATION_MODULES
                .iter()
                .find(|(key, _)| *key == form.module_key)
                .map(|(_, label)| *label)
                .unwrap_or("— select —");
            egui::ComboBox::from_id_salt("add_module_kind")
                .selected_text(current_label)
                .show_ui(ui, |ui| {
                    for (key, label) in STATION_MODULES {
                        ui.selectable_value(&mut form.module_key, key.to_string(), label);
                    }
                });
            ui.end_row();
        });

    ui.add_space(4.0);
    ui.add_enabled_ui(form.station_id.is_some(), |ui| {
        if ui.button("Add module").clicked() {
            let station_id = form.station_id.unwrap();
            let module_key = form.module_key.clone();
            let label = format!("add_module {module_key} → station {station_id}");
            let res = conn.reducers.admin_add_station_module_then(
                station_id,
                module_key,
                move |_ctx, result| log_reducer_result(label, result),
            );
            log_send_error(res);
        }
    });
}

fn message_panel(
    ui: &mut egui::Ui,
    conn: &DbConnection,
    form: &mut MessageForm,
    galaxy: &GalaxyData,
) {
    ui.weak("Send a direct server message to a player or a group. Solves the Identity/enum-entry problem the CLI can't express (#145).");

    ui.horizontal(|ui| {
        ui.label("To:");
        ui.selectable_value(&mut form.to_group, false, "Single player");
        ui.selectable_value(&mut form.to_group, true, "Group");
    });

    ui.horizontal(|ui| {
        ui.label("Severity:");
        for sev in [
            MessageSeverity::Info,
            MessageSeverity::Warning,
            MessageSeverity::Critical,
        ] {
            ui.selectable_value(&mut form.severity, sev, format!("{sev:?}"));
        }
    });

    if form.to_group {
        ui.label("Recipients:");
        egui::ScrollArea::vertical()
            .max_height(140.0)
            .show(ui, |ui| {
                if galaxy.players.is_empty() {
                    ui.weak("(no players)");
                }
                for (id, label) in &galaxy.players {
                    let mut checked = form.group_targets.contains(id);
                    if ui.checkbox(&mut checked, label).changed() {
                        if checked {
                            form.group_targets.push(*id);
                        } else {
                            form.group_targets.retain(|x| x != id);
                        }
                    }
                }
            });
    } else {
        ui.horizontal(|ui| {
            ui.label("Recipient:");
            let selected_text = form
                .target
                .and_then(|id| galaxy.players.iter().find(|(pid, _)| *pid == id))
                .map(|(_, l)| l.clone())
                .unwrap_or_else(|| "— select —".to_string());
            egui::ComboBox::from_id_salt("dm_target")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    for (id, label) in &galaxy.players {
                        ui.selectable_value(&mut form.target, Some(*id), label.clone());
                    }
                });
        });
    }

    ui.label("Message:");
    ui.text_edit_multiline(&mut form.body);

    let ready = !form.body.trim().is_empty()
        && if form.to_group {
            !form.group_targets.is_empty()
        } else {
            form.target.is_some()
        };

    ui.add_space(4.0);
    ui.add_enabled_ui(ready, |ui| {
        if ui.button("Send message").clicked() {
            let severity = form.severity;
            let body = form.body.clone();
            if form.to_group {
                let ids = form.group_targets.clone();
                let label = format!("send {severity:?} to {} players", ids.len());
                let res = conn.reducers.admin_send_direct_server_message_to_group_then(
                    ids,
                    severity,
                    body,
                    move |_ctx, result| log_reducer_result(label, result),
                );
                log_send_error(res);
            } else if let Some(target) = form.target {
                let label = format!("send {severity:?} to {}", target.to_abbreviated_hex());
                let res = conn.reducers.admin_send_direct_server_message_then(
                    target,
                    severity,
                    body,
                    move |_ctx, result| log_reducer_result(label, result),
                );
                log_send_error(res);
            }
        }
    });
}

/// Record a reducer result into the activity log. Used from the one-shot
/// `_then` callbacks. `Ok(Ok(()))` = committed; `Ok(Err(msg))` = the server
/// reducer returned an error (e.g. `try_server_only` rejection); `Err(_)` = the
/// SDK failed to deliver/await the call. Generic over the SDK's internal error
/// type so we never have to name it.
fn log_reducer_result<E: std::fmt::Display>(
    label: String,
    result: Result<Result<(), String>, E>,
) {
    match result {
        Ok(Ok(())) => connector::log_activity(format!("✓ {label}")),
        Ok(Err(msg)) => connector::log_activity(format!("✗ {label}: {msg}")),
        Err(internal) => connector::log_activity(format!("✗ {label}: SDK error {internal}")),
    }
}

/// Log the synchronous "could the request even be sent" error, if any.
fn log_send_error(res: spacetimedb_sdk::Result<()>) {
    if let Err(e) = res {
        connector::log_activity(format!("✗ failed to send request: {e}"));
    }
}
