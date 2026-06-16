use std::collections::{HashMap, HashSet};

use egui::*;
use macroquad::prelude::*;
use spacetimedb_sdk::Table;
use spacetimedb_sdk::*;

use crate::{server::bindings::*, stdb::utils::*};

/// Shrink factor applied to the auto-fit scale so sectors don't touch the
/// canvas edges. Developer-tunable — smaller = more padding around the network.
const MAP_FIT_FACTOR: f32 = 0.82;
/// Half-size (px) of a sector marker on the galaxy map.
const MAP_SECTOR_RADIUS: f32 = 8.0;

#[derive(PartialEq)]
enum MapTab {
    /// The current star system: its sectors + orbital objects (the implemented map).
    System,
    /// Galaxy-wide star-system overview — placeholder until post-MVP (#160).
    Galaxy,
}

pub struct State {
    current_tab: MapTab,

    stroke: Stroke,

    /// Accumulated pan offset (screen px) from dragging the galaxy map canvas.
    /// Reset by the "Recenter" button. Zoom is intentionally not supported (#120).
    pan: egui::Vec2,
}

impl State {
    pub fn new() -> Self {
        State {
            current_tab: MapTab::System,

            stroke: Stroke::new(2.0, Color32::from_rgb(25, 200, 100)),

            pan: egui::Vec2::ZERO,
        }
    }

    /// Tab bar + dispatch. The dialog hosts a "System Map" (the current star
    /// system) and a "Galaxy Map" (post-MVP placeholder, #160).
    fn draw_galaxy_map(&mut self, ui: &mut egui::Ui, ctx: &DbConnection) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, MapTab::System, "System Map");
            ui.selectable_value(&mut self.current_tab, MapTab::Galaxy, "Galaxy Map");
        });
        ui.separator();

        match self.current_tab {
            MapTab::System => self.draw_system_map(ui, ctx),
            MapTab::Galaxy => {
                ui.vertical_centered(|ui| {
                    ui.add_space(40.0);
                    ui.weak("Galaxy-wide star-system overview — coming after MVP (#160).");
                });
            }
        }
    }

    /// The current star system: sector dots, jumpgate edges, faded orbital
    /// backdrop, with pan + auto-fit.
    fn draw_system_map(&mut self, ui: &mut egui::Ui, ctx: &DbConnection) {
        let current_sector = if let Some(player_obj) = get_player_ship(ctx) {
            if let Some(sector) = ctx.db().sector().id().find(&player_obj.sector_id) {
                sector
            } else {
                return;
            }
        } else {
            return;
        };
        let system_name = ctx
            .db()
            .star_system()
            .id()
            .find(&current_sector.system_id)
            .map(|s| s.name)
            .unwrap_or_else(|| format!("#{}", current_sector.system_id));
        ui.horizontal(|ui| {
            ui.label("System:");
            ui.strong(&system_name);
            ui.separator();
            ui.label("Sector:");
            ui.strong(&current_sector.name);
            ui.separator();
            if ui.button("Recenter").clicked() {
                self.pan = egui::Vec2::ZERO;
            }
            ui.weak("drag to pan");
        });

        ui.separator();

        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                Sense::click_and_drag(),
            );

            // Pan the whole map by dragging anywhere on the canvas.
            self.pan += response.drag_delta();

            // Collect sectors once; bail if the world hasn't loaded yet.
            let sectors: Vec<Sector> = ctx.db().sector().iter().collect();
            if sectors.is_empty() {
                return;
            }

            // Auto-fit: derive a uniform world→screen scale from the sector
            // bounding box so the whole network fits the available canvas at
            // any window size. Resizing re-fits automatically (dest = rect).
            let mut min_w = glam::Vec2::splat(f32::INFINITY);
            let mut max_w = glam::Vec2::splat(f32::NEG_INFINITY);
            for s in &sectors {
                min_w = min_w.min(glam::vec2(s.x, s.y));
                max_w = max_w.max(glam::vec2(s.x, s.y));
            }
            let span = (max_w - min_w).max(glam::Vec2::splat(1.0));
            let center_world = (min_w + max_w) * 0.5;
            let avail = response.rect.size();
            let scale = (avail.x / span.x).min(avail.y / span.y) * MAP_FIT_FACTOR;
            let screen_center = response.rect.center();
            let pan = self.pan;

            // Uniform world→screen mapping (keeps proportions; non-distorting).
            let to_screen = |wx: f32, wy: f32| -> Pos2 {
                pos2(
                    screen_center.x + (wx - center_world.x) * scale + pan.x,
                    screen_center.y + (wy - center_world.y) * scale + pan.y,
                )
            };

            let mut backdrop = Vec::new();
            let mut edges = Vec::new();
            let mut markers = Vec::new();

            // --- Orbital backdrop (faded), current system only ---------------
            // Kept behind the sector network; opacity is reduced so the dots
            // and jumpgate edges stay the primary visual layer.
            for object in ctx.db().star_system_object().iter() {
                if object.system_id != current_sector.system_id {
                    continue;
                }
                let stroke = match object.kind {
                    StarSystemObjectKind::Star => {
                        Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 0, 70))
                    }
                    StarSystemObjectKind::Planet => {
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(173, 216, 230, 70))
                    }
                    StarSystemObjectKind::Moon => {
                        Stroke::new(1.0, Color32::from_rgba_unmultiplied(128, 128, 128, 70))
                    }
                    StarSystemObjectKind::AsteroidBelt => Stroke::new(
                        object.rotation_or_width_km,
                        Color32::from_rgba_unmultiplied(115, 52, 32, 16),
                    ),
                    StarSystemObjectKind::NebulaBelt => Stroke::new(
                        object.rotation_or_width_km,
                        Color32::from_rgba_unmultiplied(181, 69, 255, 16),
                    ),
                };

                match object.kind {
                    StarSystemObjectKind::Star
                    | StarSystemObjectKind::Planet
                    | StarSystemObjectKind::Moon => {
                        let p =
                            glam::Vec2::from_angle(object.rotation_or_width_km) * object.orbit_au;
                        let radius = match object.kind {
                            StarSystemObjectKind::Star => MAP_SECTOR_RADIUS * 1.5,
                            StarSystemObjectKind::Planet => MAP_SECTOR_RADIUS * 0.85,
                            StarSystemObjectKind::Moon => MAP_SECTOR_RADIUS * 0.5,
                            _ => unreachable!(),
                        };
                        backdrop.push(Shape::circle_stroke(to_screen(p.x, p.y), radius, stroke));
                    }
                    StarSystemObjectKind::AsteroidBelt | StarSystemObjectKind::NebulaBelt => {
                        // Belts are rings centered on the system origin.
                        backdrop.push(Shape::circle_stroke(
                            to_screen(0.0, 0.0),
                            object.orbit_au * scale,
                            stroke,
                        ));
                    }
                }
            }

            // --- Jumpgate edges ---------------------------------------------
            // One line per connected sector pair. Gates are bidirectional
            // (`connect_sectors_with_warpgates` makes two rows), so dedup on the
            // unordered (a, b) key to avoid stacking two edges per pair.
            let positions: HashMap<u64, (f32, f32)> =
                sectors.iter().map(|s| (s.id, (s.x, s.y))).collect();
            let edge_stroke = Stroke::new(1.5, Color32::from_rgb(90, 160, 150));
            let mut seen: HashSet<(u64, u64)> = HashSet::new();
            for gate in ctx.db().jump_gate().iter() {
                let (a, b) = (gate.current_sector_id, gate.target_sector_id);
                let key = if a <= b { (a, b) } else { (b, a) };
                if !seen.insert(key) {
                    continue;
                }
                if let (Some(&(ax, ay)), Some(&(bx, by))) =
                    (positions.get(&a), positions.get(&b))
                {
                    edges.push(Shape::line_segment(
                        [to_screen(ax, ay), to_screen(bx, by)],
                        edge_stroke,
                    ));
                }
            }

            // --- Sector markers ---------------------------------------------
            // Build the dot shapes now; labels are drawn last so they sit on
            // top of every other layer.
            let hover = response.hover_pos();
            let mut sector_screens: Vec<(&Sector, Pos2)> = Vec::with_capacity(sectors.len());
            for sector in &sectors {
                let center = to_screen(sector.x, sector.y);
                sector_screens.push((sector, center));

                let stroke = if current_sector.id == sector.id {
                    self.stroke // preserved green highlight for the current sector
                } else {
                    Stroke::new(1.5, Color32::from_gray(180))
                };
                let rect = egui::Rect::from_center_size(
                    center,
                    egui::Vec2::splat(2.0 * MAP_SECTOR_RADIUS),
                );
                markers.push(Shape::rect_stroke(
                    rect,
                    CornerRadius {
                        nw: 0,
                        ne: 4,
                        sw: 4,
                        se: 4,
                    },
                    stroke,
                    StrokeKind::Middle,
                ));
            }

            // Paint in z-order: backdrop, edges, sector dots.
            painter.extend(backdrop);
            painter.extend(edges);
            painter.extend(markers);

            // Labels on top. Names are always-on; coordinates appear on hover.
            for (sector, center) in &sector_screens {
                painter.text(
                    pos2(center.x, center.y - MAP_SECTOR_RADIUS - 2.0),
                    Align2::CENTER_BOTTOM,
                    &sector.name,
                    FontId::monospace(8.0),
                    Color32::WHITE,
                );
                if let Some(h) = hover {
                    if (h - *center).length() <= MAP_SECTOR_RADIUS + 4.0 {
                        painter.text(
                            pos2(center.x, center.y + MAP_SECTOR_RADIUS + 2.0),
                            Align2::CENTER_TOP,
                            format!("({:.0}, {:.0})", sector.x, sector.y),
                            FontId::monospace(7.0),
                            Color32::LIGHT_GRAY,
                        );
                    }
                }
            }
        });
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Galactic Map")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(true)
        .show(egui_ctx, |ui| {
            state.draw_galaxy_map(ui, ctx);
        })
}
