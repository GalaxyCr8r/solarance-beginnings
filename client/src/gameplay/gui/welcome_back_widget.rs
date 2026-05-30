//! Welcome-back panel (#100) — the client-side render half of the welcome-back
//! feature whose server-side composition landed in #92.
//!
//! On connect the server composes a single text-only `ServerMessage` tagged
//! with `WELCOME_BACK_CONTEXT` and routes it to the player. This widget detects
//! that message and shows it once, in a cozy centered panel, so a returning
//! player learns at a glance what changed while they were away. Closing the
//! panel marks the message read and suppresses it for the rest of the session.

use egui::{Align2, Color32, Context, RichText};
use spacetimedb_sdk::DbContext;

use crate::{gameplay::server_messages::ServerMessageUtils, server::bindings::DbConnection};

/// Per-session state for the welcome-back panel.
#[derive(Default)]
pub struct State {
    /// Set once the player dismisses the panel. Session-scoped, so a welcome-back
    /// is never re-shown after the player closes it — even if a later frame still
    /// finds the (now-read) message in the cache.
    pub dismissed: bool,
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Draw the welcome-back panel if there is an undismissed welcome-back message.
///
/// Cheap no-op once dismissed, or when no welcome-back message exists yet (the
/// subscription may not have delivered it on the first frame after connect —
/// we simply pick it up on whichever frame it arrives).
pub fn draw(egui_ctx: &Context, ctx: &DbConnection, state: &mut State) {
    if state.dismissed {
        return;
    }

    let Some((message, _recipient)) =
        ServerMessageUtils::get_latest_welcome_back(ctx, &ctx.identity())
    else {
        return;
    };

    let mut close_requested = false;

    egui::Window::new("Welcome Back")
        .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .min_width(360.0)
        .show(egui_ctx, |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Welcome back, pilot")
                    .heading()
                    .color(Color32::from_rgb(120, 190, 255)),
            );
            ui.separator();
            ui.add_space(4.0);

            // The server joins the summary lines with '\n'; egui renders the
            // newlines directly. Keep it text-only and easy to read at a glance.
            ui.label(RichText::new(&message.message).size(15.0));

            ui.add_space(8.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button("Dismiss").clicked() {
                    close_requested = true;
                }
            });
        });

    if close_requested {
        // Mark read so the chat unread indicator clears too; failures here are
        // non-fatal (the session-level `dismissed` flag still hides the panel).
        let _ = ServerMessageUtils::mark_message_as_read(ctx, message.id);
        state.dismissed = true;
    }
}
