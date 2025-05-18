use egui::{Align2, Color32, Context, RichText, ScrollArea, TextStyle, Ui};
use macroquad::prelude::*;

use crate::{module_bindings::*, stdb::utils::get_username};

#[derive(Default)]
pub struct ChatWindowState {
    pub global_chat_channel: Vec<GlobalChatMessage>,
    pub text: String,
    pub selected_tab: u8,
    pub has_focus: bool,
    pub hidden: bool
}

fn contents_hidden(ui: &mut Ui, ctx: &DbConnection, chat_window: &mut ChatWindowState) {
    ui.horizontal(|ui| {
        if ui.button("^").clicked() {
            chat_window.hidden = false;
        }

        ui.label(RichText::new(" Global ").color(if chat_window.selected_tab == 0 {Color32::DARK_GRAY} else {Color32::BLACK}));
        ui.label(RichText::new(" Sector ").color(Color32::BLACK));
        ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
        ui.label(RichText::new(" Faction ").color(Color32::BLACK));
    });

    ui.separator();

    ui.label(RichText::new("...").color(Color32::DARK_GRAY));
    for message in chat_window.global_chat_channel.iter().rev().take(3).rev() {
        ui.label(RichText::new(format!("[{}]: {}", get_username(ctx, &message.identity), message.message)).color(Color32::DARK_GRAY));
    }
}

pub fn chat_window(egui_ctx: &Context, ctx: &DbConnection, chat_window: &mut ChatWindowState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Chat Window")
        .min_width(256.0)
        .title_bar(false)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_BOTTOM, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            if chat_window.hidden {
                contents_hidden(ui, ctx, chat_window);
                return
            }

            ui.horizontal(|ui| {
                if !chat_window.hidden && ui.button("v").clicked() {
                    chat_window.hidden = true;
                }

                ui.selectable_value(&mut chat_window.selected_tab, 0, "Global");
                ui.label(RichText::new(" Sector ").color(Color32::BLACK));
                ui.label(RichText::new(" Alliance ").color(Color32::BLACK));
                ui.label(RichText::new(" Faction ").color(Color32::BLACK));
                // ui.selectable_value(&mut chat_window.selected_tab, 1, "Sector");
                // ui.selectable_value(&mut chat_window.selected_tab, 2, "Alliance");
                // ui.selectable_value(&mut chat_window.selected_tab, 3, "Faction");
            });
            ui.separator();

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
                            ui.label(format!("[{}]: {}", get_username(ctx, &message.identity), message.message));
                        }
                        count += 1;
                    }
                },
            );

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
                        if let Err(error) = ctx.reducers.send_global_chat(chat_window.text.clone()) {
                            info!("Failed to send message: {}", error);
                            // TODO Add a message to chat log or do SOMETHING to alert the user it failed.
                        } else {
                            chat_window.text.clear();
                        }
                    }
                }
            });
        })
}