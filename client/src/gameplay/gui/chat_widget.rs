use egui::{Align2, Color32, Context, RichText, ScrollArea, TextStyle, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Timestamp};

use crate::{gameplay::server_messages::ServerMessageUtils, module_bindings::*, stdb::utils::*};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum GlobalChatMessageType {
    Server,
    Global,
    Sector,
    Alliance,
    Faction,
}

impl Default for GlobalChatMessageType {
    fn default() -> Self {
        GlobalChatMessageType::Server
    }
}

#[derive(Default)]
pub struct State {
    pub global_chat_channel: Vec<GlobalChatMessage>,
    pub sector_chat_channel: Vec<SectorChatMessage>,
    pub text: String,
    pub selected_tab: GlobalChatMessageType,
    pub has_focus: bool,
    pub hidden: bool,
}

fn contents_hidden(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    ui.horizontal(|ui| {
        if ui.button("^").clicked() {
            chat_window.hidden = false;
        }

        // Show server tab with unread indicator (first tab)
        let unread_count = ServerMessageUtils::get_unread_count(ctx, &ctx.identity());
        let server_tab_text = if unread_count > 0 {
            format!(" Server* ")
        } else {
            " Server ".to_string()
        };
        ui.label(RichText::new(server_tab_text).color(
            if chat_window.selected_tab == GlobalChatMessageType::Server {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            },
        ));

        ui.label(RichText::new(" Global ").color(
            if chat_window.selected_tab == GlobalChatMessageType::Global {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            },
        ));
        ui.label(RichText::new(" Sector ").color(
            if chat_window.selected_tab == GlobalChatMessageType::Sector {
                Color32::DARK_GRAY
            } else {
                Color32::BLACK
            },
        ));

        ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
        ui.label(RichText::new(" Faction ").color(Color32::BLACK));
    });

    ui.separator();

    ui.label(RichText::new("...").color(Color32::DARK_GRAY));
    for message in chat_window.global_chat_channel.iter().rev().take(3).rev() {
        ui.label(
            RichText::new(format!(
                "[{}]: {}",
                get_username(ctx, &message.player_id),
                message.message
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

pub fn _draw_widget(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    ui.horizontal(|ui| {
        if !chat_window.hidden && ui.button("v").clicked() {
            chat_window.hidden = true;
        }

        // Server tab with unread indicator (first tab)
        let unread_count = ServerMessageUtils::get_unread_count(ctx, &ctx.identity());
        let server_tab_text = if unread_count > 0 {
            format!("Server*")
        } else {
            "Server".to_string()
        };
        ui.selectable_value(
            &mut chat_window.selected_tab,
            GlobalChatMessageType::Server,
            server_tab_text,
        );

        ui.selectable_value(
            &mut chat_window.selected_tab,
            GlobalChatMessageType::Global,
            "Global",
        );
        ui.selectable_value(
            &mut chat_window.selected_tab,
            GlobalChatMessageType::Sector,
            "Sector",
        );

        ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
        ui.label(RichText::new(" Faction ").color(Color32::BLACK));
    });
    ui.separator();

    match chat_window.selected_tab {
        GlobalChatMessageType::Global => {
            draw_global_chat(ctx, chat_window, ui);
        }
        GlobalChatMessageType::Sector => {
            draw_sector_chat(ctx, chat_window, ui);
        }
        GlobalChatMessageType::Server => {
            draw_server_messages(ctx, ui);
        }
        GlobalChatMessageType::Alliance => todo!(),
        GlobalChatMessageType::Faction => todo!(),
    }

    ui.horizontal(|ui| {
        chat_window.has_focus = false;
        if ui.text_edit_singleline(&mut chat_window.text).has_focus() {
            // Game State -> Can Move Ship False
            // Also need to make that new state boolean set to "True" each loop
            // and make sure this dialog is shown before player input can be processed
            chat_window.has_focus = true;
        }
        if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !chat_window.text.is_empty() {
                send_message(ctx, chat_window);
            }
        }
    });
}

pub fn draw_panel(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    let sector_enabled = ctx
        .db()
        .sobj_player_window()
        .id()
        .find(&ctx.identity())
        .is_some();
    if chat_window.selected_tab == GlobalChatMessageType::Sector && !sector_enabled {
        chat_window.selected_tab = GlobalChatMessageType::Server;
    }

    egui::TopBottomPanel::top("chat_top")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                if !chat_window.hidden && ui.button("v").clicked() {
                    chat_window.hidden = true;
                }

                // Server tab with unread indicator (first tab)
                let unread_count = ServerMessageUtils::get_unread_count(ctx, &ctx.identity());
                let server_tab_text = if unread_count > 0 {
                    format!("Server*")
                } else {
                    "Server".to_string()
                };
                ui.selectable_value(
                    &mut chat_window.selected_tab,
                    GlobalChatMessageType::Server,
                    server_tab_text,
                );

                ui.selectable_value(
                    &mut chat_window.selected_tab,
                    GlobalChatMessageType::Global,
                    "Global",
                );
                if sector_enabled {
                    ui.selectable_value(
                        &mut chat_window.selected_tab,
                        GlobalChatMessageType::Sector,
                        "Sector",
                    );
                } else {
                    ui.label(RichText::new(" Sector ").color(Color32::BLACK));
                }

                ui.label(RichText::new(" Station ").color(Color32::BLACK));
                ui.label(RichText::new(" Faction ").color(Color32::BLACK));
                ui.label(RichText::new(" Guild ").color(Color32::BLACK));
            });
        });

    egui::TopBottomPanel::bottom("chat_bottom")
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                chat_window.has_focus = false;
                if ui.text_edit_singleline(&mut chat_window.text).has_focus() {
                    // Game State -> Can Move Ship False
                    // Also need to make that new state boolean set to "True" each loop
                    // and make sure this dialog is shown before player input can be processed
                    chat_window.has_focus = true;
                }
                if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !chat_window.text.is_empty() {
                        send_message(ctx, chat_window);
                    }
                }
            });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| match chat_window.selected_tab {
        GlobalChatMessageType::Global => {
            draw_global_chat(ctx, chat_window, ui);
        }
        GlobalChatMessageType::Sector => {
            draw_sector_chat(ctx, chat_window, ui);
        }
        GlobalChatMessageType::Server => {
            draw_server_messages(ctx, ui);
        }
        GlobalChatMessageType::Alliance => todo!(),
        GlobalChatMessageType::Faction => todo!(),
    });
}

fn send_message(ctx: &DbConnection, chat_window: &mut State) {
    match chat_window.selected_tab {
        GlobalChatMessageType::Global => {
            if let Err(error) = ctx.reducers.send_global_chat(chat_window.text.clone()) {
                info!("Failed to send message: {}", error);
                // TODO Add a message to chat log or do SOMETHING to alert the user it failed.
                chat_window.global_chat_channel.push(GlobalChatMessage {
                    player_id: ctx.identity(),
                    id: 0,
                    message: format!("Failed to send message: {}", chat_window.text.clone()),
                    created_at: Timestamp::now(),
                });
            } else {
                chat_window.text.clear();
            }
        }
        GlobalChatMessageType::Sector => {
            let sector_id = get_player_ship(ctx).unwrap().sector_id;
            if let Err(error) = ctx
                .reducers
                .send_sector_chat(chat_window.text.clone(), sector_id)
            {
                info!("Failed to send message: {}", error);
                // TODO Add a message to chat log or do SOMETHING to alert the user it failed.
                chat_window.sector_chat_channel.push(SectorChatMessage {
                    player_id: ctx.identity(),
                    id: 0,
                    sector_id: sector_id,
                    message: format!("Failed to send message: {}", chat_window.text.clone()),
                    created_at: Timestamp::now(),
                });
            } else {
                chat_window.text.clear();
            }
        }
        GlobalChatMessageType::Server => {
            // Server messages are read-only, so we don't allow sending messages
            // Just clear the text if user tries to send
            chat_window.text.clear();
        }
        GlobalChatMessageType::Alliance => todo!(),
        GlobalChatMessageType::Faction => todo!(),
    }
}

fn draw_global_chat(ctx: &DbConnection, chat_window: &mut State, ui: &mut Ui) {
    let text_style = TextStyle::Body;
    let row_height = ui.text_style_height(&text_style);
    ScrollArea::vertical()
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .max_height(screen_height() / 4.0)
        .show_rows(
            ui,
            row_height,
            chat_window.global_chat_channel.len(),
            |ui, row_range| {
                let mut count = 0;
                for message in &chat_window.global_chat_channel {
                    if row_range.contains(&count) {
                        ui.horizontal(|ui| {
                            // Timestamp
                            let timestamp_text =
                                ServerMessageUtils::format_timestamp_short(&message.created_at);
                            ui.label(
                                RichText::new(format!("[{}]", timestamp_text))
                                    .color(Color32::GRAY)
                                    .size(10.0),
                            );

                            // Message
                            ui.label(format!(
                                "[{}]: {}",
                                get_username(ctx, &message.player_id),
                                message.message
                            ));
                        });
                    }
                    count += 1;
                }
            },
        );
}

fn draw_sector_chat(ctx: &DbConnection, chat_window: &mut State, ui: &mut Ui) {
    let text_style = TextStyle::Body;
    let row_height = ui.text_style_height(&text_style);

    ScrollArea::vertical()
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .max_height(screen_height() / 4.0)
        .show_rows(
            ui,
            row_height,
            chat_window.sector_chat_channel.len(),
            |ui, row_range| {
                let mut count = 0;
                for message in &chat_window.sector_chat_channel {
                    if row_range.contains(&count) {
                        ui.horizontal(|ui| {
                            // Timestamp
                            let timestamp_text =
                                ServerMessageUtils::format_timestamp_short(&message.created_at);
                            ui.label(
                                RichText::new(format!("[{}]", timestamp_text))
                                    .color(Color32::GRAY)
                                    .size(10.0),
                            );

                            // Message
                            ui.label(format!(
                                "({}): {}",
                                get_username(ctx, &message.player_id),
                                message.message
                            ));
                        });
                    }
                    count += 1;
                }
            },
        );
}
fn draw_server_messages(ctx: &DbConnection, ui: &mut Ui) {
    let text_style = TextStyle::Body;
    let row_height = ui.text_style_height(&text_style) * 1.5; // Slightly taller for better readability

    // Get server messages for the current player
    let messages = ServerMessageUtils::get_messages_for_player(ctx, &ctx.identity());

    ScrollArea::vertical()
        .auto_shrink([false, true])
        .stick_to_bottom(true)
        .max_height(screen_height() / 4.0)
        .show_rows(ui, row_height, messages.len(), |ui, row_range| {
            let mut count = 0;
            for (message, recipient) in &messages {
                if row_range.contains(&count) {
                    ui.horizontal(|ui| {
                        // Unread indicator
                        if recipient.read_at.is_none() {
                            ui.label(RichText::new("●").color(Color32::from_rgb(255, 215, 0)));
                        // Gold dot for unread
                        } else {
                            ui.label(RichText::new("○").color(Color32::GRAY)); // Gray circle for read
                        }

                        // Timestamp
                        let timestamp_text =
                            ServerMessageUtils::format_timestamp_short(&message.created_at);
                        ui.label(
                            RichText::new(format!("[{}]", timestamp_text))
                                .color(Color32::GRAY)
                                .size(10.0),
                        );

                        // Message type prefix with color
                        let type_prefix = match message.message_type {
                            ServerMessageType::Error => "[ERROR]",
                            ServerMessageType::Info => "[INFO]",
                            ServerMessageType::Warning => "[WARNING]",
                            ServerMessageType::Admin => "[ADMIN]",
                            ServerMessageType::System => "[SYSTEM]",
                        };

                        let type_color =
                            ServerMessageUtils::get_message_color(&message.message_type);
                        let mut type_text = RichText::new(type_prefix).color(type_color);

                        // Make urgent message types bold
                        if ServerMessageUtils::is_urgent_message(&message.message_type) {
                            type_text = type_text.strong();
                        }

                        ui.label(type_text);

                        // Context if available
                        if let Some(context) = &message.sender_context {
                            ui.label(
                                RichText::new(format!("{}:", context))
                                    .color(Color32::DARK_GRAY)
                                    .strong(),
                            );
                        }

                        // Main message content
                        let mut message_text = RichText::new(&message.message);

                        // Style based on read status
                        if recipient.read_at.is_none() {
                            message_text = message_text.strong(); // Unread messages are bold
                        } else {
                            message_text = message_text.color(Color32::GRAY); // Read messages are dimmed
                        }

                        // Make the message clickable to mark as read
                        let response = ui.label(message_text);
                        if response.clicked() && recipient.read_at.is_none() {
                            let _ = ServerMessageUtils::mark_message_as_read(ctx, message.id);
                        }

                        // Group name if available (without showing other recipients)
                        if let Some(group_name) = &message.group_name {
                            ui.label(
                                RichText::new(format!("(Group: {})", group_name))
                                    .color(Color32::from_rgb(100, 150, 200))
                                    .size(10.0),
                            );
                        }
                    });

                    // Add some spacing between messages
                    ui.add_space(2.0);
                }
                count += 1;
            }
        });
}
