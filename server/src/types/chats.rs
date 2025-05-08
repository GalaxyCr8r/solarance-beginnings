use log::info;
use spacetimedb::{table, Identity, ReducerContext, Table, Timestamp};

use crate::types::players::get_username;

#[table(name = global_chat, public)]
pub struct GlobalChat {
    #[primary_key]
    #[auto_inc]
    pub id: u64,

    pub identity: Identity, // FK to Player

    pub message: String,

    created_at: Timestamp,
}

//// Impls ///

impl GlobalChat {
    //
}

//// Reducers ///

#[spacetimedb::reducer]
pub fn send_global_chat(ctx: &ReducerContext, chat_message: String) {
    // If ctx.sender is a valid, unbanned, unmuted player
    info!("Message received: [{}]: {}", get_username(ctx, ctx.sender), chat_message);

    ctx.db.global_chat().insert(GlobalChat {
        id: 0,
        identity: ctx.sender,
        message: chat_message,
        created_at: ctx.timestamp
    });
}
