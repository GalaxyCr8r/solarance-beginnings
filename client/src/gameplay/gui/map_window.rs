use egui::*;
use macroquad::prelude::*;
use spacetimedb_sdk::Table;
use spacetimedb_sdk::*;

use crate::{module_bindings::*, stdb::utils::*};

#[derive(PartialEq)]
enum CurrentTab {
    Ship,
    Cargo,
}

pub struct State {
    current_tab: CurrentTab,

    stroke: Stroke,
}

impl State {
    pub fn new() -> Self {
        State {
            current_tab: CurrentTab::Ship,

            stroke: Stroke::new(2.0, Color32::from_rgb(25, 200, 100)),
        }
    }

    fn draw_galaxy_map(&self, ui: &mut egui::Ui, ctx: &DbConnection) {
        let current_sector = if let Some(player_obj) = get_player_ship(ctx) {
            if let Some(sector) = ctx.db().sector().id().find(&player_obj.sector_id) {
                sector
            } else {
                return;
            }
        } else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Current Sector:");
            ui.label(&current_sector.name);
        });

        ui.separator();

        // TODO Canvas of the galaxy
        Frame::canvas(ui.style()).show(ui, |ui| {
            let (response, painter) = ui.allocate_painter(
                egui::Vec2::new(ui.available_width(), ui.available_height()),
                Sense::hover(),
            );

            let to_screen = emath::RectTransform::from_to(
                egui::Rect::from_min_size(Pos2::ZERO, response.rect.size()),
                response.rect,
            );

            let mut shapes = Vec::new();
            let offset = pos2(response.rect.width() / 2., response.rect.height() / 2.);
            let zoom_px = 1.0;
            let sector_radius = 8.0;

            for (i, object) in ctx.db().star_system_object().iter().enumerate() {
                if object.system_id != current_sector.system_id {
                    continue;
                }

                let stroke = match object.kind {
                    StarSystemObjectKind::Star => Stroke {
                        width: 3.,
                        color: Color32::YELLOW,
                    },
                    StarSystemObjectKind::Planet => Stroke {
                        width: 1.,
                        color: Color32::LIGHT_BLUE,
                    },
                    StarSystemObjectKind::Moon => Stroke {
                        width: 1.,
                        color: Color32::GRAY,
                    },
                    StarSystemObjectKind::AsteroidBelt => Stroke {
                        width: object.rotation_or_width_km,
                        color: Color32::from_rgba_unmultiplied(115, 52, 32, 16),
                    },
                    StarSystemObjectKind::NebulaBelt => Stroke {
                        width: object.rotation_or_width_km,
                        color: Color32::from_rgba_unmultiplied(181, 69, 255, 16),
                    },
                };

                match object.kind {
                    StarSystemObjectKind::Star
                    | StarSystemObjectKind::Planet
                    | StarSystemObjectKind::Moon => {
                        let point_in_system =
                            glam::Vec2::from_angle(object.rotation_or_width_km) * object.orbit_au;

                        let mut point = pos2(
                            point_in_system.x * zoom_px + offset.x,
                            point_in_system.y * zoom_px + offset.y,
                        );
                        point = to_screen.from().clamp(point);
                        let point_in_screen = to_screen.transform_pos(point);

                        let radius = match object.kind {
                            StarSystemObjectKind::Star => sector_radius * 1.5,
                            StarSystemObjectKind::Planet => sector_radius * 0.85,
                            StarSystemObjectKind::Moon => sector_radius * 0.5,
                            _ => unreachable!(),
                        };

                        shapes.push(Shape::circle_stroke(point_in_screen, radius, stroke));
                    }
                    StarSystemObjectKind::AsteroidBelt | StarSystemObjectKind::NebulaBelt => {
                        let mut point = pos2(zoom_px + offset.x, zoom_px + offset.y);
                        point = to_screen.from().clamp(point);
                        let point_in_screen = to_screen.transform_pos(point);

                        shapes.push(Shape::circle_stroke(
                            point_in_screen,
                            object.orbit_au,
                            stroke,
                        ));
                    }
                }
            }

            for (i, sector) in ctx.db().sector().iter().enumerate() {
                let size = egui::Vec2::splat(2.0 * sector_radius);

                let mut point = pos2(sector.x * zoom_px + offset.x, sector.y * zoom_px + offset.y);
                let point_in_screen = to_screen.transform_pos(point);

                let point_rect = egui::Rect::from_center_size(point_in_screen, size);
                let point_id = response.id.with(i);
                let point_response = ui.interact(point_rect, point_id, Sense::click());

                point += point_response.drag_delta();
                point = to_screen.from().clamp(point);

                let point_in_screen = to_screen.transform_pos(point);
                let stroke = if current_sector.id == sector.id {
                    self.stroke
                } else {
                    ui.style().interact(&point_response).fg_stroke
                };

                if point_response.hovered() {
                    painter.text(
                        to_screen.transform_pos(point),
                        Align2::CENTER_BOTTOM,
                        format!("{}", sector.name),
                        FontId::monospace(16.0),
                        Color32::WHITE,
                    );
                    painter.text(
                        to_screen.transform_pos(point),
                        Align2::CENTER_TOP,
                        format!("({},{})", sector.x, sector.y),
                        FontId::monospace(8.0),
                        Color32::WHITE,
                    );
                }

                let min = Pos2::new(
                    point_in_screen.x - sector_radius,
                    point_in_screen.y - sector_radius,
                );
                let max = Pos2::new(
                    point_in_screen.x + sector_radius,
                    point_in_screen.y + sector_radius,
                );
                shapes.push(Shape::rect_stroke(
                    egui::Rect { min, max },
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
            painter.extend(shapes);
        });

        // egui::Frame::group(ui.style())
        //   .inner_margin(0.0)
        //   .show(ui, |ui| {
        //     //
        //     let mut reset_view = false;
        //     let mut inner_rect = egui::Rect::NAN;
        //     egui::
        //   });
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Galatic Maps")
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
