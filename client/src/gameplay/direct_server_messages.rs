//! Client-side helpers for the **Direct Server Message** inbox (#101).
//!
//! The server exposes DMs through the `my_direct_server_messages` View, which
//! auto-filters to the current player. The client simply reads, sorts, and
//! formats them; there is no read-state to mutate (login-relative).

use crate::server::bindings::*;
use spacetimedb_sdk::{DbContext, Table};

/// Client-side utilities for handling Direct Server Messages.
pub struct DirectServerMessageUtils;

impl DirectServerMessageUtils {
    /// All Direct Server Messages addressed to the current player, newest first.
    pub fn get_messages(ctx: &DbConnection) -> Vec<DirectServerMessage> {
        let mut messages: Vec<DirectServerMessage> = ctx
            .db()
            .my_direct_server_messages()
            .iter()
            .collect();
        messages.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        messages
    }

    /// Effective unread cutoff: the later of `last_login` (server-side
    /// login-relative read state) and `dismissed_at` (session-local "Mark all
    /// read" click). Either or both may be `None`.
    fn effective_cutoff(
        last_login: Option<spacetimedb_sdk::Timestamp>,
        dismissed_at: Option<spacetimedb_sdk::Timestamp>,
    ) -> Option<spacetimedb_sdk::Timestamp> {
        match (last_login, dismissed_at) {
            (Some(a), Some(b)) => Some(if a > b { a } else { b }),
            (Some(t), None) | (None, Some(t)) => Some(t),
            (None, None) => None,
        }
    }

    /// Messages that arrived after the effective unread cutoff. Used by the
    /// chat widget to render an unread highlight; the cutoff is the later of
    /// `last_login` and any session-local "Mark all read" dismissal.
    pub fn get_unread(
        ctx: &DbConnection,
        last_login: Option<spacetimedb_sdk::Timestamp>,
        dismissed_at: Option<spacetimedb_sdk::Timestamp>,
    ) -> Vec<DirectServerMessage> {
        let cutoff = match Self::effective_cutoff(last_login, dismissed_at) {
            Some(t) => t,
            None => return Self::get_messages(ctx),
        };
        Self::get_messages(ctx)
            .into_iter()
            .filter(|m| m.created_at > cutoff)
            .collect()
    }

    /// Count of unread DMs (post effective cutoff).
    pub fn get_unread_count(
        ctx: &DbConnection,
        last_login: Option<spacetimedb_sdk::Timestamp>,
        dismissed_at: Option<spacetimedb_sdk::Timestamp>,
    ) -> usize {
        Self::get_unread(ctx, last_login, dismissed_at).len()
    }

    /// The welcome-back DM — the **earliest** message at-or-after `last_login`.
    ///
    /// Post-#101 the welcome-back carries no discriminator. The composer is
    /// the first thing `client_connected` runs and re-stamps `last_login` in
    /// the same transaction, so the welcome-back row has
    /// `created_at == last_login`. Any other server DM that fires after
    /// connect (e.g. mining resuming on a tick a few seconds later) has a
    /// *later* timestamp, so picking the **earliest** qualifying message
    /// reliably finds the welcome-back even when post-connect chatter exists.
    pub fn get_latest_welcome_back(
        ctx: &DbConnection,
        last_login: Option<spacetimedb_sdk::Timestamp>,
    ) -> Option<DirectServerMessage> {
        let cutoff = last_login?;
        // `get_messages` returns newest-first; reverse to walk oldest-first
        // and take the first one at or after the cutoff.
        Self::get_messages(ctx)
            .into_iter()
            .rev()
            .find(|m| m.created_at >= cutoff)
    }

    /// Format a single DM for inbox rendering.
    pub fn format_for_display(message: &DirectServerMessage) -> String {
        let tag = match message.severity {
            MessageSeverity::Info => "[INFO]",
            MessageSeverity::Warning => "[WARNING]",
            MessageSeverity::Critical => "[CRITICAL]",
        };
        format!("{} {}", tag, message.body)
    }

    /// Color for a DM by severity. Three tiers, three colors.
    pub fn color_for_severity(severity: &MessageSeverity) -> egui::Color32 {
        match severity {
            MessageSeverity::Info => egui::Color32::from_rgb(50, 150, 220),
            MessageSeverity::Warning => egui::Color32::from_rgb(255, 165, 0),
            MessageSeverity::Critical => egui::Color32::from_rgb(220, 50, 50),
        }
    }

    /// "2026-06-08T15:33" — short timestamp used as a per-row gutter label.
    /// Kept here (rather than in a generic utils module) because it is the
    /// shared formatter the chat widget uses across both channel + DM rows.
    pub fn format_timestamp_short(timestamp: &spacetimedb_sdk::Timestamp) -> String {
        match timestamp.to_rfc3339() {
            Ok(datetime) => datetime.as_str()[..16].to_string(),
            Err(_) => "n/a".to_string(),
        }
    }
}

/// Render the sender of a Channel Message for display.
/// `MessageSender::Player(id)` looks up the username via `get_username`;
/// `MessageSender::System` becomes the literal `"SYSTEM"`.
pub fn render_sender(ctx: &DbConnection, sender: &MessageSender) -> String {
    match sender {
        MessageSender::Player(id) => crate::stdb::utils::get_username(ctx, id),
        MessageSender::System => "SYSTEM".to_string(),
    }
}
