use egui::{Align2, Color32, Context, RichText, ScrollArea, TextStyle, Ui};
use macroquad::prelude::*;
use spacetimedb_sdk::{DbContext, Timestamp};

use crate::{module_bindings::*, stdb::utils::*};

#[derive(Clone, Debug, PartialEq, PartialOrd)]
enum GlobalChatMessageType {
    Global,
    Sector,
    Alliance,
    Faction
}

impl Default for GlobalChatMessageType {
    fn default() -> Self {
        GlobalChatMessageType::Global
    }
}

#[derive(Default)]
pub struct State {
    pub global_chat_channel: Vec<GlobalChatMessage>,
    pub sector_chat_channel: Vec<SectorChatMessage>,
    pub text: String,
    pub selected_tab: GlobalChatMessageType,
    pub has_focus: bool,
    pub hidden: bool
}

fn contents_hidden(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut State) {
    ui.horizontal(|ui| {
        if ui.button("^").clicked() {
            chat_window.hidden = false;
        }

        ui.label(RichText::new(" Global ").color(
            if chat_window.selected_tab == GlobalChatMessageType::Global {Color32::DARK_GRAY} else {Color32::BLACK}));
        ui.label(RichText::new(" Sector ").color(
            if chat_window.selected_tab == GlobalChatMessageType::Sector {Color32::DARK_GRAY} else {Color32::BLACK}));
        ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
        ui.label(RichText::new(" Faction ").color(Color32::BLACK));
    });

    ui.separator();

    ui.label(RichText::new("...").color(Color32::DARK_GRAY));
    for message in chat_window.global_chat_channel.iter().rev().take(3).rev() {
        ui.label(RichText::new(format!("[{}]: {}", get_username(ctx, &message.player_id), message.message)).color(Color32::DARK_GRAY));
    }
}

pub fn draw(egui_ctx: &Context, ctx: &DbConnection, chat_window: &mut State) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Chat Window")
        .min_width(256.0)
        .title_bar(false)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if chat_window.hidden {
                contents_hidden(ui, ctx, chat_window);
                return
            }

            ui.horizontal(|ui| {
                if !chat_window.hidden && ui.button("v").clicked() {
                    chat_window.hidden = true;
                }

                ui.selectable_value(&mut chat_window.selected_tab, GlobalChatMessageType::Global, "Global");
                ui.selectable_value(&mut chat_window.selected_tab, GlobalChatMessageType::Sector, "Sector");
                ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
                ui.label(RichText::new(" Faction ").color(Color32::BLACK));
            });
            ui.separator();

            match chat_window.selected_tab {
                GlobalChatMessageType::Global => {
                    draw_global_chat(ctx, chat_window, ui);
                },
                GlobalChatMessageType::Sector => {
                    draw_sector_chat(ctx, chat_window, ui);
                },
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
                if ui.button("Send").clicked() || ui.input(|i| i.key_pressed(egui::Key::Enter))
                {
                    if !chat_window.text.is_empty() {
                        send_message(ctx, chat_window);
                    }
                }
            });
        })
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
            },
        GlobalChatMessageType::Sector => {
                let sector_id = get_player_ship(ctx).unwrap().sector_id;
                if let Err(error) = ctx.reducers.send_sector_chat(chat_window.text.clone(), sector_id) {
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
            },
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
        .max_height(screen_height()/4.0)
        .show_rows(
        ui,
        row_height,
        chat_window.global_chat_channel.len(),
        |ui, row_range| {
            let mut count = 0;
            for message in &chat_window.global_chat_channel {
                if row_range.contains(&count) {
                    ui.label(format!("[{}]: {}", get_username(ctx, &message.player_id), message.message));
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
        .max_height(screen_height()/4.0)
        .show_rows(
        ui,
        row_height,
        chat_window.sector_chat_channel.len(),
        |ui, row_range| {
            let mut count = 0;
            for message in &chat_window.sector_chat_channel {
                if row_range.contains(&count) {
                    ui.label(format!("({}): {}", get_username(ctx, &message.player_id), message.message));
                }
                count += 1;
            }
        },
    );
}