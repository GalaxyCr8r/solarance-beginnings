//! Welcome-back panel (#100) — the client-side render half of the welcome-back
//! feature whose server-side composition landed in #92.
//!
//! Post-#101 the welcome-back is a plain `DirectServerMessage` with no
//! discriminator. We identify it as the most-recent DM at-or-after the player's
//! `last_login` — the server's composer fires inside `client_connected`, so
//! that row is always the freshest one when this panel checks. Dismissing the
//! panel just hides it for the session; there is no server-side read state to
//! mutate.

use egui::{Align2, Color32, Context, RichText};
use spacetimedb_sdk::DbContext;

use crate::{
    gameplay::direct_server_messages::DirectServerMessageUtils,
    server::bindings::DbConnection,
    stdb::utils::get_current_player,
};

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

    // The player row's `last_login` is the pre-connect timestamp the server
    // composed the welcome-back against. If the row hasn't streamed in yet
    // we wait — without it we can't tell welcome-back from any other DM.
    let Some(player) = get_current_player(ctx) else {
        return;
    };
    let Some(message) =
        DirectServerMessageUtils::get_latest_welcome_back(ctx, player.last_login)
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
            ui.label(RichText::new(&message.body).size(15.0));

            ui.add_space(8.0);
            ui.separator();
            ui.vertical_centered(|ui| {
                if ui.button("Dismiss").clicked() {
                    close_requested = true;
                }
            });
        });

    if close_requested {
        state.dismissed = true;
    }
}
