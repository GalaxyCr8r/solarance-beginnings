use std::f32::consts::PI;

use glam::Vec2;
use log::info;
use spacetimedb::*;
use spacetimedsl::*;

use crate::tables::{
    items::*, jumpgates::*, server_messages::utility::send_info_message, ships::timers::*,
    stations::*,
};

use super::*;

pub fn get_username(ctx: &ReducerContext, id: Identity) -> String {
    if let Some(player) = ctx.db().player().id().find(id) {
        player.username
    } else {
        if ctx.sender == ctx.identity() {
            "SERVER".to_string()
        } else {
            id.to_abbreviated_hex().to_string()
        }
    }
}
