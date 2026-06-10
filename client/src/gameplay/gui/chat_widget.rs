//! Chat widget — tabbed access to all six messaging channels (#101).
//!
//! Each tab reads directly from its STDB View / public table — there is no
//! mirrored state in `State`. The views auto-update on insert, so each frame
//! sees fresh rows for free. This is the simpler shape the new schema enables;
//! the old MPSC mirror existed only because the per-faction filter had to
//! happen in Rust callbacks.
//!
//! ## Tabs
//! - **Server** — `ServerChannelMessage` (public, MOTD; read-only).
//! - **Galaxy** — `my_galaxy_chat` view; logged-in players only.
//! - **System** — `my_star_system_chat` view; player's current star system.
//! - **Sector** — `my_sector_chat` view; player's current sector.
//! - **Faction** — `my_faction_chat` view; player's faction.
//! - **DM** — `my_direct_server_messages` view; the async inbox.

use std::cmp::Ordering;

use egui::{Align2, Color32, Context, RichText, ScrollArea, TextStyle, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Table, Timestamp};

use crate::{
    gameplay::direct_server_messages::{render_sender, DirectServerMessageUtils},
    server::bindings::*,
    stdb::utils::*,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd)]
pub enum ChatTab {
    Server,
    Galaxy,
    System,
    Sector,
    Faction,
    DirectMessages,
}

impl Default for ChatTab {
    fn default() -> Self {
        ChatTab::Galaxy
    }
}

#[derive(Default)]
pub struct State {
    pub text: String,
    pub selected_tab: ChatTab,
    pub has_focus: bool,
    pub hidden: bool,
    /// Session-local "Mark all read" timestamp. The DM unread count is taken
    /// against `max(player.last_login, dms_dismissed_at)`. None until the
    /// player clicks the Read button; cleared on next session (login-relative
    /// is still the source of truth across sessions).
    pub dms_dismissed_at: Option<Timestamp>,
}

fn contents_hidden(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    ui.horizontal(|ui| {
        if ui.button("^").clicked() {
            chat_window.hidden = false;
        }

        let unread_count = unread_dm_count(ctx, chat_window);
        let dm_tab_text = if unread_count > 0 {
            format!(" DM* ")
        } else {
            " DM ".to_string()
        };

        for (label, tab) in [
            (" Server ".to_string(), ChatTab::Server),
            (" Galaxy ".to_string(), ChatTab::Galaxy),
            (" System ".to_string(), ChatTab::System),
            (" Sector ".to_string(), ChatTab::Sector),
            (" Faction ".to_string(), ChatTab::Faction),
            (dm_tab_text, ChatTab::DirectMessages),
        ] {
            ui.label(RichText::new(label).color(if chat_window.selected_tab == tab {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            }));
        }
    });

    ui.separator();

    // Always show the last 3 Galaxy messages collapsed — most public stream.
    ui.label(RichText::new("...").color(Color32::DARK_GRAY));
    let mut messages: Vec<GalaxyChannelMessage> = ctx.db().my_galaxy_chat().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    for message in messages.iter().rev().take(3).rev() {
        ui.label(
            RichText::new(format!(
                "[{}]: {}",
                render_sender(ctx, &message.sender),
                message.body
            ))
            .color(Color32::DARK_GRAY),
        );
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    chat_window: &mut State,
) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window::new("Chat Window")
        .min_width(256.0)
        .title_bar(false)
        .resizable(true)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if chat_window.hidden {
                contents_hidden(ui, ctx, chat_window);
            } else {
                draw_panel(ui, ctx, chat_window);
            }
        })
}

pub fn draw_panel(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    // Sector tab is only meaningful while the player has an in-sector ship.
    let sector_enabled = ctx
        .db()
        .ship()
        .iter()
        .any(|s| s.player_id == ctx.identity() && s.location == ShipLocation::Sector);
    if chat_window.selected_tab == ChatTab::Sector && !sector_enabled {
        chat_window.selected_tab = ChatTab::Galaxy;
    }

    egui::TopBottomPanel::top("chat_top")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if !chat_window.hidden && ui.button("v").clicked() {
                    chat_window.hidden = true;
                }

                ui.selectable_value(&mut chat_window.selected_tab, ChatTab::Server, "Server");
                ui.selectable_value(&mut chat_window.selected_tab, ChatTab::Galaxy, "Galaxy");
                ui.selectable_value(&mut chat_window.selected_tab, ChatTab::System, "System");
                if sector_enabled {
                    ui.selectable_value(&mut chat_window.selected_tab, ChatTab::Sector, "Sector");
                } else {
                    ui.label(RichText::new(" Sector ").color(Color32::BLACK));
                }
                ui.selectable_value(&mut chat_window.selected_tab, ChatTab::Faction, "Faction");

                let unread_count = unread_dm_count(ctx, chat_window);
                let dm_text = if unread_count > 0 {
                    format!("*DM ({})", unread_count)
                } else {
                    "DM".to_string()
                };
                ui.selectable_value(&mut chat_window.selected_tab, ChatTab::DirectMessages, dm_text);
            });
        });

    egui::TopBottomPanel::bottom("chat_bottom")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                chat_window.has_focus = false;
                // The DM tab + Server tab are read-only (server-composed).
                let read_only = matches!(
                    chat_window.selected_tab,
                    ChatTab::Server | ChatTab::DirectMessages
                );
                if read_only {
                    ui.label(RichText::new("(read-only)").color(Color32::DARK_GRAY));
                } else {
                    if ui.text_edit_singleline(&mut chat_window.text).has_focus() {
                        chat_window.has_focus = true;
                    }
                    if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter))
                    {
                        if !chat_window.text.is_empty() {
                            send_message(ctx, chat_window);
                        }
                    }
                }
            });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| match chat_window.selected_tab {
        ChatTab::Server => draw_server_channel(ctx, ui),
        ChatTab::Galaxy => draw_galaxy_channel(ctx, ui),
        ChatTab::System => draw_system_channel(ctx, ui),
        ChatTab::Sector => draw_sector_channel(ctx, ui),
        ChatTab::Faction => draw_faction_channel(ctx, ui),
        ChatTab::DirectMessages => draw_direct_messages(ctx, chat_window, ui),
    });
}

fn send_message(ctx: &DbConnection, chat_window: &mut State) {
    let body = chat_window.text.clone();
    let result = match chat_window.selected_tab {
        ChatTab::Galaxy => ctx.reducers.send_galaxy_chat(body),
        ChatTab::System => ctx.reducers.send_star_system_chat(body),
        ChatTab::Sector => ctx.reducers.send_sector_chat(body),
        ChatTab::Faction => ctx.reducers.send_faction_chat(body),
        // Read-only tabs are filtered out before send_message is called.
        ChatTab::Server | ChatTab::DirectMessages => return,
    };
    if let Err(error) = result {
        info!("Failed to send message: {}", error);
        // No fallback local-echo: a failed send surfaces in logs only — the
        // reducer `_then` callback can carry it into a future toast.
    } else {
        chat_window.text.clear();
    }
}

fn unread_dm_count(ctx: &DbConnection, chat_window: &State) -> usize {
    let last_login = get_current_player(ctx).and_then(|p| p.last_login);
    DirectServerMessageUtils::get_unread_count(ctx, last_login, chat_window.dms_dismissed_at)
}

fn draw_galaxy_channel(ctx: &DbConnection, ui: &mut Ui) {
    let mut messages: Vec<GalaxyChannelMessage> = ctx.db().my_galaxy_chat().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    draw_scrolling_list(ui, messages.len(), |ui, idx| {
        let message = &messages[idx];
        let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
        (timestamp, format!("[{}]: {}", render_sender(ctx, &message.sender), message.body))
    });
}

fn draw_system_channel(ctx: &DbConnection, ui: &mut Ui) {
    let mut messages: Vec<StarSystemChannelMessage> =
        ctx.db().my_star_system_chat().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    draw_scrolling_list(ui, messages.len(), |ui, idx| {
        let message = &messages[idx];
        let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
        (timestamp, format!("({}): {}", render_sender(ctx, &message.sender), message.body))
    });
}

fn draw_sector_channel(ctx: &DbConnection, ui: &mut Ui) {
    let mut messages: Vec<SectorChannelMessage> = ctx.db().my_sector_chat().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    draw_scrolling_list(ui, messages.len(), |ui, idx| {
        let message = &messages[idx];
        let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
        (timestamp, format!("({}): {}", render_sender(ctx, &message.sender), message.body))
    });
}

fn draw_faction_channel(ctx: &DbConnection, ui: &mut Ui) {
    let mut messages: Vec<FactionChannelMessage> = ctx.db().my_faction_chat().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    draw_scrolling_list(ui, messages.len(), |ui, idx| {
        let message = &messages[idx];
        let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
        (timestamp, format!("{}: {}", render_sender(ctx, &message.sender), message.body))
    });
}

fn draw_server_channel(ctx: &DbConnection, ui: &mut Ui) {
    let mut messages: Vec<ServerChannelMessage> = ctx.db().server_channel_message().iter().collect();
    messages.sort_by_key(|m| m.created_at);
    draw_scrolling_list(ui, messages.len(), |ui, idx| {
        let message = &messages[idx];
        let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
        (timestamp, format!("[MOTD] {}", message.body))
    });
}

fn draw_direct_messages(ctx: &DbConnection, chat_window: &mut State, ui: &mut Ui) {
    let last_login = get_current_player(ctx).and_then(|p| p.last_login);

    // Newest-at-bottom to match every other chat tab. `get_messages` returns
    // newest-first; reverse here so the ScrollArea (stick_to_bottom) keeps
    // the latest visible.
    let mut messages = DirectServerMessageUtils::get_messages(ctx);
    messages.reverse();

    // "Read" header: clears the unread highlight + count for this session.
    // Session-local — the next login resumes login-relative tracking.
    let unread_count = DirectServerMessageUtils::get_unread_count(
        ctx,
        last_login,
        chat_window.dms_dismissed_at,
    );
    ui.horizontal(|ui| {
        if unread_count > 0 {
            ui.label(
                RichText::new(format!("{} unread", unread_count))
                    .color(Color32::from_rgb(255, 215, 0)),
            );
            if ui.button("Read").clicked() {
                chat_window.dms_dismissed_at = Some(Timestamp::now());
            }
        } else {
            ui.label(RichText::new("No unread").color(Color32::DARK_GRAY));
        }
    });
    ui.separator();

    let text_style = TextStyle::Body;
    let row_height = ui.text_style_height(&text_style) * 1.5;

    let dismissed_at = chat_window.dms_dismissed_at;
    ScrollArea::vertical()
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .show_rows(ui, row_height, messages.len(), |ui, row_range| {
            let mut last_timestamp = String::new();
            for idx in row_range {
                let message = &messages[idx];
                let timestamp = DirectServerMessageUtils::format_timestamp_short(&message.created_at);
                if timestamp.cmp(&last_timestamp) != Ordering::Equal {
                    ui.label(
                        RichText::new(format!("[{}]", timestamp))
                            .color(Color32::GRAY)
                            .size(10.0),
                    );
                    last_timestamp = timestamp;
                }
                let cutoff = match (last_login, dismissed_at) {
                    (Some(a), Some(b)) => Some(if a > b { a } else { b }),
                    (Some(t), None) | (None, Some(t)) => Some(t),
                    (None, None) => None,
                };
                let is_unread = cutoff.map(|c| message.created_at > c).unwrap_or(true);
                let severity_color =
                    DirectServerMessageUtils::color_for_severity(&message.severity);
                let prefix = match message.severity {
                    MessageSeverity::Info => "[INFO]",
                    MessageSeverity::Warning => "[WARNING]",
                    MessageSeverity::Critical => "[CRITICAL]",
                };
                ui.horizontal(|ui| {
                    if is_unread {
                        ui.label(RichText::new("●").color(Color32::from_rgb(255, 215, 0)));
                    }
                    let mut prefix_text = RichText::new(prefix).color(severity_color);
                    if matches!(message.severity, MessageSeverity::Critical) {
                        prefix_text = prefix_text.strong();
                    }
                    ui.label(prefix_text);
                    let mut body_text = RichText::new(&message.body);
                    if is_unread {
                        body_text = body_text.strong();
                    } else {
                        body_text = body_text.color(Color32::GRAY);
                    }
                    ui.label(body_text);
                });
                ui.add_space(2.0);
            }
        });
}

/// Shared scroll-list scaffolding with grouped timestamp gutters.
fn draw_scrolling_list<F>(ui: &mut Ui, total: usize, mut row: F)
where
    F: FnMut(&mut Ui, usize) -> (String, String),
{
    let text_style = TextStyle::Body;
    let row_height = ui.text_style_height(&text_style);
    ScrollArea::vertical()
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .show_rows(ui, row_height, total, |ui, row_range| {
            let mut last_timestamp = String::new();
            for idx in row_range {
                let (timestamp, text) = row(ui, idx);
                if timestamp.cmp(&last_timestamp) != Ordering::Equal {
                    ui.label(
                        RichText::new(format!("[{}]", timestamp))
                            .color(Color32::GRAY)
                            .size(10.0),
                    );
                    last_timestamp = timestamp;
                }
                ui.label(text);
            }
        });
}
